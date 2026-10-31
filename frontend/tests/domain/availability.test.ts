import { describe, expect, it } from 'vitest'
import {
  availabilityLevel,
  discoverParticipants,
  evaluateSelectedOverlap,
  filterAvailability,
  summarizeAvailability,
} from '~/domain/availability'
import type { AvailabilityResponse } from '~/types/api/contracts'

const responses: AvailabilityResponse[] = [
  {
    slot: 'wt-10',
    entries: [
      { user_id: 'u_2', nick: 'Żaneta', state: 'yes', updated_at: 2 },
      { user_id: 'u_1', nick: 'Ala', state: 'maybe', updated_at: 2 },
    ],
  },
  {
    slot: 'pn-09',
    entries: [
      { user_id: 'u_1', nick: 'Ala', state: 'yes', updated_at: 1 },
      { user_id: 'u_2', nick: 'Żaneta', state: 'yes', updated_at: 1 },
    ],
  },
  {
    slot: 'pn-10',
    entries: [{ user_id: 'u_1', nick: 'Ala', state: 'yes', updated_at: 3 }],
  },
]

describe('availability aggregation', () => {
  it('discovers unique participants from successful slot entries', () => {
    expect(discoverParticipants(responses)).toEqual([
      { userId: 'u_1', nick: 'Ala' },
      { userId: 'u_2', nick: 'Żaneta' },
    ])
  })

  it('summarizes states and keeps chronological slot order', () => {
    expect(summarizeAvailability(responses).map(summary => ({
      slot: summary.slot,
      yes: summary.yesCount,
      maybe: summary.maybeCount,
    }))).toEqual([
      { slot: 'pn-09', yes: 2, maybe: 0 },
      { slot: 'pn-10', yes: 1, maybe: 0 },
      { slot: 'wt-10', yes: 1, maybe: 1 },
    ])
  })

  it('maps weighted availability to four color-depth levels', () => {
    const summaries = summarizeAvailability(responses)
    expect(availabilityLevel(summaries.find(slot => slot.slot === 'pn-09'), 2)).toBe(4)
    expect(availabilityLevel(summaries.find(slot => slot.slot === 'wt-10'), 2)).toBe(3)
    expect(availabilityLevel(undefined, 2)).toBe(0)
  })

  it('filters the heatmap to one or many selected participants', () => {
    const filtered = filterAvailability(summarizeAvailability(responses), ['u_2'])
    expect(filtered.map(summary => ({ slot: summary.slot, yes: summary.yesCount, maybe: summary.maybeCount }))).toEqual([
      { slot: 'pn-09', yes: 1, maybe: 0 },
      { slot: 'pn-10', yes: 0, maybe: 0 },
      { slot: 'wt-10', yes: 1, maybe: 0 },
    ])
  })

  it('keeps multiple selected participants while excluding everyone else', () => {
    const summaries = summarizeAvailability([{
      slot: 'pn-09',
      entries: [
        { user_id: 'u_1', nick: 'Ala', state: 'yes', updated_at: 1 },
        { user_id: 'u_2', nick: 'Żaneta', state: 'yes', updated_at: 1 },
        { user_id: 'u_3', nick: 'Ela', state: 'maybe', updated_at: 1 },
      ],
    }])

    expect(filterAvailability(summaries, ['u_1', 'u_3'])).toEqual([{
      slot: 'pn-09',
      entries: [summaries[0]!.entries[0], summaries[0]!.entries[2]],
      yesCount: 1,
      maybeCount: 1,
    }])
  })
})

describe('selected participant overlap', () => {
  it('is eligible only when at least two people including the current user are selected', () => {
    const summaries = summarizeAvailability(responses)
    const options = { summaries, allSlotsKnown: true, hasPendingWrites: false }

    expect(evaluateSelectedOverlap({
      ...options,
      participantIds: ['u_1'],
      currentUserId: 'u_1',
    })).toEqual({ status: 'insufficient' })
    expect(evaluateSelectedOverlap({
      ...options,
      participantIds: ['u_2', 'u_3'],
      currentUserId: 'u_1',
    })).toEqual({ status: 'insufficient' })
  })

  it('reports overlap only when every selected participant has an entry in one slot', () => {
    expect(evaluateSelectedOverlap({
      summaries: summarizeAvailability(responses),
      participantIds: ['u_1', 'u_2'],
      currentUserId: 'u_1',
      allSlotsKnown: true,
      hasPendingWrites: false,
    })).toEqual({ status: 'overlap' })
  })

  it('reports no overlap when selected schedules are separated', () => {
    const separated = summarizeAvailability([
      { slot: 'pn-09', entries: [{ user_id: 'u_1', nick: 'Ala', state: 'yes', updated_at: 1 }] },
      { slot: 'wt-10', entries: [{ user_id: 'u_2', nick: 'Żaneta', state: 'maybe', updated_at: 2 }] },
    ])
    const result = evaluateSelectedOverlap({
      summaries: separated,
      participantIds: ['u_1', 'u_2'],
      currentUserId: 'u_1',
      allSlotsKnown: true,
      hasPendingWrites: false,
    })

    expect(result).toEqual({ status: 'no-overlap' })
  })

  it('does not claim no overlap with partial data, pending writes, or missing own entries', () => {
    const summaries = summarizeAvailability(responses)
    const base = { summaries, participantIds: ['u_1', 'u_2'], currentUserId: 'u_1' }
    expect(evaluateSelectedOverlap({ ...base, allSlotsKnown: false, hasPendingWrites: false })).toEqual({ status: 'pending' })
    expect(evaluateSelectedOverlap({ ...base, allSlotsKnown: true, hasPendingWrites: true })).toEqual({ status: 'pending' })
    expect(evaluateSelectedOverlap({
      summaries,
      participantIds: ['u_1', 'u_missing'],
      currentUserId: 'u_1',
      allSlotsKnown: true,
      hasPendingWrites: false,
    })).toEqual({ status: 'insufficient' })
  })
})
