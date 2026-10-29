import { afterEach, describe, expect, it, vi } from 'vitest'
import { requestJson } from '~/services/api/result'

const isGreeting = (value: unknown): value is { greeting: string } => (
  typeof value === 'object' && value !== null && 'greeting' in value
  && typeof value.greeting === 'string'
)

afterEach(() => {
  vi.useRealTimers()
  vi.unstubAllGlobals()
})

describe('API result normalization', () => {
  it('returns validated JSON on success', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(
      JSON.stringify({ greeting: 'hej' }),
      { status: 200, headers: { 'Content-Type': 'application/json' } },
    )))

    await expect(requestJson('https://example.test', {}, isGreeting)).resolves.toEqual({
      ok: true,
      data: { greeting: 'hej' },
    })
  })

  it('preserves documented API error codes', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(
      JSON.stringify({ error: { code: 'event_not_found', message: 'missing' } }),
      { status: 404, headers: { 'Content-Type': 'application/json' } },
    )))

    await expect(requestJson('https://example.test', {}, isGreeting)).resolves.toEqual({
      ok: false,
      error: { kind: 'api', status: 404, code: 'event_not_found', message: 'missing' },
    })
  })

  it('distinguishes network and invalid response failures', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('offline')))
    const networkResult = await requestJson('https://example.test', {}, isGreeting)
    expect(networkResult).toMatchObject({ ok: false, error: { kind: 'network' } })

    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('{}', { status: 200 })))
    const invalidResult = await requestJson('https://example.test', {}, isGreeting)
    expect(invalidResult).toMatchObject({ ok: false, error: { kind: 'invalid-response', status: 200 } })
  })

  it('aborts requests that exceed the shared transport timeout', async () => {
    vi.useFakeTimers()
    vi.stubGlobal('fetch', vi.fn().mockImplementation((_url: string, init: RequestInit) => (
      new Promise((_resolve, reject) => {
        init.signal?.addEventListener('abort', () => reject(new DOMException('aborted', 'AbortError')))
      })
    )))

    const resultPromise = requestJson('https://example.test', {}, isGreeting)
    await vi.advanceTimersByTimeAsync(15_000)

    await expect(resultPromise).resolves.toMatchObject({ ok: false, error: { kind: 'network' } })
  })
})
