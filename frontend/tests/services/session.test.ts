import { afterEach, describe, expect, it, vi } from 'vitest'
import { forgetEventSession, readEventSession, tokenExpiryStatus, writeEventSession } from '~/services/session'

function tokenWithExpiry(exp: number): string {
  const payload = btoa(JSON.stringify({ exp })).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '')
  return `header.${payload}.signature`
}

afterEach(() => {
  sessionStorage.clear()
  vi.restoreAllMocks()
})

describe('event session adapter', () => {
  it('isolates sessions by event and forgets only the requested event', () => {
    const session = { userId: 'u_1', nick: 'Iza', token: 'secret' }
    expect(writeEventSession('e_one', session)).toBe(true)
    expect(readEventSession('e_one')).toEqual(session)
    expect(readEventSession('e_two')).toBeNull()

    forgetEventSession('e_one')
    expect(readEventSession('e_one')).toBeNull()
  })

  it('degrades safely when session storage is unavailable', () => {
    vi.spyOn(sessionStorage, 'setItem').mockImplementation(() => { throw new DOMException('blocked') })
    expect(writeEventSession('e_one', { userId: 'u_1', nick: 'Iza', token: 'secret' })).toBe(false)
  })
})

describe('JWT expiry hint', () => {
  it('classifies expired, soon-expiring and valid tokens without treating decoding as validation', () => {
    expect(tokenExpiryStatus(tokenWithExpiry(999), 1_000)).toEqual({ status: 'expired', expiresAt: 999 })
    expect(tokenExpiryStatus(tokenWithExpiry(1_100), 1_000, 200)).toEqual({ status: 'expiring', expiresAt: 1_100 })
    expect(tokenExpiryStatus(tokenWithExpiry(2_000), 1_000, 200)).toEqual({ status: 'valid', expiresAt: 2_000 })
    expect(tokenExpiryStatus('not-a-token', 1_000)).toEqual({ status: 'unknown' })
  })
})
