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

export type OverlapEvaluation =
  | { status: 'pending' | 'insufficient' | 'overlap' }
  | { status: 'no-overlap'; signature: string }

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

  const participantIds = [...new Set(options.participantIds)].sort()
  if (participantIds.length < 2 || !options.currentUserId || !participantIds.includes(options.currentUserId)) {
    return { status: 'insufficient' }
  }

  const hasAnyEntry = participantIds.every(userId => options.summaries.some(summary => (
    summary.entries.some(entry => entry.user_id === userId)
  )))
  if (!hasAnyEntry) return { status: 'insufficient' }

  const scored = options.summaries.map((summary) => {
    const selectedEntries = summary.entries.filter(entry => participantIds.includes(entry.user_id))
    return {
      availableCount: new Set(selectedEntries.map(entry => entry.user_id)).size,
    }
  })
  if (scored.some(slot => slot.availableCount === participantIds.length)) return { status: 'overlap' }

  const signature = participantIds.map(userId => (
    `${userId}:${options.summaries.flatMap(summary => (
      summary.entries.find(entry => entry.user_id === userId)
        ? [`${summary.slot}=${summary.entries.find(entry => entry.user_id === userId)?.state}`]
        : []
    )).join(',')}`
  )).join('|')

  return { status: 'no-overlap', signature }
}
