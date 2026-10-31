import { flushPromises } from '@vue/test-utils'
import { mockNuxtImport } from '@nuxt/test-utils/runtime'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { useEventSchedule } from '~/composables/useEventSchedule'
import type { ApiResult } from '~/services/api/result'
import type { AvailabilityEntry, AvailabilityResponse } from '~/types/api/contracts'

const { useApiMock } = vi.hoisted(() => ({ useApiMock: vi.fn() }))
mockNuxtImport('useApi', () => useApiMock)

function entry(state: 'yes' | 'maybe'): AvailabilityEntry {
  return { user_id: 'u_1', nick: 'Iza', state, updated_at: 1 }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((next) => { resolve = next })
  return { promise, resolve }
}

function setupSchedule(
  setMyAvailability: ReturnType<typeof vi.fn>,
  initialEntries: AvailabilityEntry[] = [],
) {
  const slots = {
    getAvailability: vi.fn().mockResolvedValue({
      ok: true,
      data: { slot: 'pn-09', entries: initialEntries },
    }),
    setMyAvailability,
    clearMyAvailability: vi.fn(),
  }
  useApiMock.mockReturnValue({ slots })

  const eventId = ref('e_test')
  const event = ref({ event_id: 'e_test', name: 'Test', slots: ['pn-09'] as const })
  return { schedule: useEventSchedule(eventId, event), slots }
}

afterEach(() => useApiMock.mockReset())

describe('useEventSchedule writes', () => {
  it('serializes and coalesces rapid changes for one slot', async () => {
    const first = deferred<ApiResult<AvailabilityEntry>>()
    const setMyAvailability = vi.fn()
      .mockReturnValueOnce(first.promise)
      .mockResolvedValueOnce({ ok: true, data: entry('maybe') })
    const { schedule } = setupSchedule(setMyAvailability)
    await schedule.refreshAll()

    schedule.setDesiredAvailability('pn-09', { userId: 'u_1', token: 'token' }, 'yes')
    schedule.setDesiredAvailability('pn-09', { userId: 'u_1', token: 'token' }, 'maybe')

    expect(setMyAvailability).toHaveBeenCalledTimes(1)
    first.resolve({ ok: true, data: entry('yes') })
    await flushPromises()

    expect(setMyAvailability).toHaveBeenCalledTimes(2)
    expect(setMyAvailability.mock.calls[1]?.[3]).toBe('maybe')
    expect(schedule.summaries.value[0]?.entries[0]?.state).toBe('maybe')
    expect(schedule.writeStates.value['pn-09']).toBeUndefined()
  })

  it('retains the newest intent when an earlier in-flight write fails', async () => {
    const first = deferred<ApiResult<AvailabilityEntry>>()
    const setMyAvailability = vi.fn().mockReturnValue(first.promise)
    const { schedule } = setupSchedule(setMyAvailability)
    await schedule.refreshAll()

    schedule.setDesiredAvailability('pn-09', { userId: 'u_1', token: 'token' }, 'yes')
    schedule.setDesiredAvailability('pn-09', { userId: 'u_1', token: 'token' }, 'maybe')
    first.resolve({ ok: false, error: { kind: 'network', message: 'offline' } })
    await flushPromises()

    expect(schedule.writeStates.value['pn-09']).toMatchObject({ status: 'failed', desired: 'maybe' })
    expect(setMyAvailability).toHaveBeenCalledTimes(1)
  })

  it('keeps failed intent and retries it with a replacement session', async () => {
    const setMyAvailability = vi.fn()
      .mockResolvedValueOnce({
        ok: false,
        error: { kind: 'api', status: 401, code: 'invalid_token', message: 'expired' },
      })
      .mockResolvedValueOnce({ ok: true, data: entry('yes') })
    const { schedule } = setupSchedule(setMyAvailability)
    await schedule.refreshAll()

    schedule.setDesiredAvailability('pn-09', { userId: 'u_1', token: 'old' }, 'yes')
    await flushPromises()

    expect(schedule.reauthRequired.value).toBe(true)
    expect(schedule.writeStates.value['pn-09']).toMatchObject({ status: 'failed', desired: 'yes' })

    schedule.retryFailedWrites({ userId: 'u_1', token: 'new' })
    await flushPromises()

    expect(setMyAvailability.mock.calls[1]?.[2]).toBe('new')
    expect(schedule.reauthRequired.value).toBe(false)
    expect(schedule.writeStates.value['pn-09']).toBeUndefined()
  })

  it('retries a write rejected for a token replaced while it was in flight', async () => {
    const stale = deferred<ApiResult<AvailabilityEntry>>()
    const setMyAvailability = vi.fn()
      .mockReturnValueOnce(stale.promise)
      .mockResolvedValueOnce({ ok: true, data: entry('yes') })
    const { schedule } = setupSchedule(setMyAvailability)
    await schedule.refreshAll()

    schedule.setDesiredAvailability('pn-09', { userId: 'u_1', token: 'old' }, 'yes')
    schedule.retryFailedWrites({ userId: 'u_1', token: 'new' })
    stale.resolve({ ok: false, error: { kind: 'api', status: 401, code: 'invalid_token', message: 'expired' } })
    await flushPromises()

    expect(setMyAvailability.mock.calls[1]?.[2]).toBe('new')
    expect(schedule.reauthRequired.value).toBe(false)
    expect(schedule.writeStates.value['pn-09']).toBeUndefined()
  })

  it('never retries another participant’s failed intent with a replacement identity', async () => {
    const setMyAvailability = vi.fn().mockResolvedValue({
      ok: false,
      error: { kind: 'network', message: 'offline' },
    })
    const { schedule } = setupSchedule(setMyAvailability)
    await schedule.refreshAll()

    schedule.setDesiredAvailability('pn-09', { userId: 'u_1', token: 'old' }, 'yes')
    await flushPromises()
    schedule.retryFailedWrites({ userId: 'u_2', token: 'new' })
    await flushPromises()

    expect(setMyAvailability).toHaveBeenCalledTimes(1)
    expect(schedule.writeStates.value['pn-09']).toBeUndefined()
  })

  it('deletes the current entry when the desired state is absent', async () => {
    const { schedule, slots } = setupSchedule(vi.fn(), [entry('maybe')])
    slots.clearMyAvailability.mockResolvedValue({ ok: true, data: undefined })
    await schedule.refreshAll()

    schedule.setDesiredAvailability('pn-09', { userId: 'u_1', token: 'token' }, null)
    await flushPromises()

    expect(slots.clearMyAvailability).toHaveBeenCalledWith('pn-09', 'e_test', 'token')
    expect(schedule.summaries.value[0]?.entries).toEqual([])
    expect(schedule.writeStates.value['pn-09']).toBeUndefined()
  })

  it('does not let an overlapping stale refresh erase a confirmed write', async () => {
    const pendingWrite = deferred<ApiResult<AvailabilityEntry>>()
    const setMyAvailability = vi.fn().mockReturnValue(pendingWrite.promise)
    const { schedule, slots } = setupSchedule(setMyAvailability)
    await schedule.refreshAll()

    const staleRead = deferred<ApiResult<AvailabilityResponse>>()
    slots.getAvailability.mockReturnValueOnce(staleRead.promise)
    schedule.setDesiredAvailability('pn-09', { userId: 'u_1', token: 'token' }, 'yes')
    const refresh = schedule.refreshAll()
    pendingWrite.resolve({ ok: true, data: entry('yes') })
    await flushPromises()

    staleRead.resolve({ ok: true, data: { slot: 'pn-09', entries: [] } })
    await refresh

    expect(schedule.summaries.value[0]?.entries).toEqual([entry('yes')])
    expect(schedule.writeStates.value['pn-09']).toBeUndefined()
  })
})
