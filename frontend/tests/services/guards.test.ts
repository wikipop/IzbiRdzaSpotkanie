import { describe, expect, it } from 'vitest'
import { isAvailabilityResponse, isEventResponse } from '~/services/api/guards'

describe('API response guards', () => {
  it('accepts a verified event response shape', () => {
    expect(isEventResponse({ event_id: 'e_test', name: 'Test', slots: ['pn-09'] })).toBe(true)
  })

  it('rejects event responses containing invalid slots', () => {
    expect(isEventResponse({ event_id: 'e_test', name: 'Test', slots: ['pn-24'] })).toBe(false)
  })

  it('validates nested availability entries', () => {
    expect(isAvailabilityResponse({
      slot: 'wt-10',
      entries: [{ user_id: 'u_test', nick: 'Iza', state: 'maybe', updated_at: 1_700_000_000 }],
    })).toBe(true)
    expect(isAvailabilityResponse({
      slot: 'wt-10',
      entries: [{ user_id: 'u_test', nick: 'Iza', state: 'absent', updated_at: 1_700_000_000 }],
    })).toBe(false)
  })
})
