import { compareSlots } from './slots'
import type { AvailabilityEntry, AvailabilityResponse, SlotId } from '~/types/api/contracts'

export interface ParticipantSummary {
  userId: string
  nick: string
}

export interface SlotSummary {
  slot: SlotId
  entries: AvailabilityEntry[]
  yesCount: number
  maybeCount: number
}

export type AvailabilityLevel = 0 | 1 | 2 | 3 | 4

export interface OverlapEvaluation {
  status: 'pending' | 'insufficient' | 'overlap' | 'no-overlap'
}

export function availabilityLevel(summary: SlotSummary | undefined, participantCount: number): AvailabilityLevel {
  if (!summary || participantCount <= 0) return 0
  const weightedAvailability = summary.yesCount + summary.maybeCount * 0.5
  if (weightedAvailability === 0) return 0
  return Math.min(4, Math.max(1, Math.ceil((weightedAvailability / participantCount) * 4))) as AvailabilityLevel
}

export function summarizeAvailability(responses: AvailabilityResponse[]): SlotSummary[] {
  return responses.map(response => ({
    slot: response.slot,
    entries: response.entries,
    yesCount: response.entries.filter(entry => entry.state === 'yes').length,
    maybeCount: response.entries.filter(entry => entry.state === 'maybe').length,
  })).sort((left, right) => compareSlots(left.slot, right.slot))
}

export function discoverParticipants(responses: AvailabilityResponse[]): ParticipantSummary[] {
  const participants = new Map<string, ParticipantSummary>()

  for (const response of responses) {
    for (const entry of response.entries) {
      if (!participants.has(entry.user_id)) {
        participants.set(entry.user_id, { userId: entry.user_id, nick: entry.nick })
      }
    }
  }

  return [...participants.values()].sort((left, right) => (
    left.nick.localeCompare(right.nick, 'pl', { sensitivity: 'base' })
  ))
}

export function filterAvailability(summaries: SlotSummary[], participantIds: string[]): SlotSummary[] {
  if (participantIds.length === 0) return summaries
  const selected = new Set(participantIds)

  return summaries.map((summary) => {
    const entries = summary.entries.filter(entry => selected.has(entry.user_id))
    return {
      ...summary,
      entries,
      yesCount: entries.filter(entry => entry.state === 'yes').length,
      maybeCount: entries.filter(entry => entry.state === 'maybe').length,
    }
  })
}

export function evaluateSelectedOverlap(options: {
  summaries: SlotSummary[]
  participantIds: string[]
  currentUserId?: string
  allSlotsKnown: boolean
  hasPendingWrites: boolean
}): OverlapEvaluation {
  if (!options.allSlotsKnown || options.hasPendingWrites) return { status: 'pending' }

  const participantIds = new Set(options.participantIds)
  if (participantIds.size < 2 || !options.currentUserId || !participantIds.has(options.currentUserId)) {
    return { status: 'insufficient' }
  }

  const withAnyEntry = new Set<string>()
  let hasOverlap = false
  for (const summary of options.summaries) {
    const available = new Set<string>()
    for (const entry of summary.entries) {
      if (!participantIds.has(entry.user_id)) continue
      available.add(entry.user_id)
      withAnyEntry.add(entry.user_id)
    }
    if (available.size === participantIds.size) hasOverlap = true
  }
  if (withAnyEntry.size < participantIds.size) return { status: 'insufficient' }

  return { status: hasOverlap ? 'overlap' : 'no-overlap' }
}
