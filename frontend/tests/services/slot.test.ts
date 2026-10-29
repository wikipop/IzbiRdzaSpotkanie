import { afterEach, describe, expect, it, vi } from 'vitest'
import { createSlotClient } from '~/services/api/slot'

afterEach(() => vi.unstubAllGlobals())

describe('slot client authorization boundaries', () => {
  it('never adds Authorization to public reads', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(
      JSON.stringify({ slot: 'pn-09', entries: [] }),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    ))
    vi.stubGlobal('fetch', fetchMock)

    const client = createSlotClient({ mode: 'development' })
    await client.getAvailability('pn-09', 'e_test')

    expect(fetchMock).toHaveBeenCalledOnce()
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(new Headers(init.headers).has('Authorization')).toBe(false)
  })

  it('rejects a valid availability body returned for a different slot', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(
      JSON.stringify({ slot: 'wt-10', entries: [] }),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    )))

    const result = await createSlotClient({ mode: 'development' }).getAvailability('pn-09', 'e_test')

    expect(result).toEqual({
      ok: false,
      error: { kind: 'invalid-response', status: 200, message: 'Serwer zwrócił niepoprawne dane.' },
    })
  })

  it('adds the bearer token only to authenticated writes', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(
      JSON.stringify({ user_id: 'u_test', nick: 'Iza', state: 'yes', updated_at: 1 }),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    ))
    vi.stubGlobal('fetch', fetchMock)

    const client = createSlotClient({ mode: 'development' })
    await client.setMyAvailability('pn-09', 'e_test', 'secret-token', 'yes')

    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(new Headers(init.headers).get('Authorization')).toBe('Bearer secret-token')
  })
})
