import { apiErrorCodes, type ApiErrorCode, type ApiErrorEnvelope } from '~/types/api/contracts'

export type ApiFailure =
  | { kind: 'api'; status: number; code: ApiErrorCode; message: string }
  | { kind: 'network'; message: string }
  | { kind: 'invalid-response'; status: number; message: string }

export type ApiResult<T> =
  | { ok: true; data: T }
  | { ok: false; error: ApiFailure }

const requestTimeoutMs = 15_000

async function fetchWithTimeout(url: string, init: RequestInit): Promise<Response> {
  const controller = new AbortController()
  const externalSignal = init.signal
  const abortFromExternal = () => controller.abort(externalSignal?.reason)
  if (externalSignal?.aborted) abortFromExternal()
  else externalSignal?.addEventListener('abort', abortFromExternal, { once: true })
  const timeout = setTimeout(() => controller.abort(new DOMException('Request timed out', 'TimeoutError')), requestTimeoutMs)

  try {
    return await fetch(url, { ...init, signal: controller.signal })
  } finally {
    clearTimeout(timeout)
    externalSignal?.removeEventListener('abort', abortFromExternal)
  }
}

function isApiErrorEnvelope(value: unknown): value is ApiErrorEnvelope {
  if (!value || typeof value !== 'object' || !('error' in value)) return false
  const error = value.error
  if (!error || typeof error !== 'object') return false
  const record = error as Record<string, unknown>
  return typeof record.message === 'string'
    && typeof record.code === 'string'
    && apiErrorCodes.includes(record.code as ApiErrorCode)
}

export async function requestJson<T>(
  url: string,
  init: RequestInit = {},
  isResponse: (value: unknown) => value is T,
): Promise<ApiResult<T>> {
  let response: Response

  try {
    response = await fetchWithTimeout(url, {
      ...init,
      headers: {
        Accept: 'application/json',
        ...(init.body ? { 'Content-Type': 'application/json' } : {}),
        ...init.headers,
      },
    })
  } catch {
    return { ok: false, error: { kind: 'network', message: 'Nie udało się połączyć z serwerem.' } }
  }

  const body: unknown = await response.json().catch(() => null)

  if (!response.ok) {
    if (isApiErrorEnvelope(body)) {
      return {
        ok: false,
        error: { kind: 'api', status: response.status, code: body.error.code, message: body.error.message },
      }
    }
    return {
      ok: false,
      error: { kind: 'invalid-response', status: response.status, message: 'Serwer zwrócił nieznany błąd.' },
    }
  }

  if (!isResponse(body)) {
    return {
      ok: false,
      error: { kind: 'invalid-response', status: response.status, message: 'Serwer zwrócił niepoprawne dane.' },
    }
  }

  return { ok: true, data: body }
}

export async function requestNoContent(url: string, init: RequestInit): Promise<ApiResult<undefined>> {
  let response: Response

  try {
    response = await fetchWithTimeout(url, {
      ...init,
      headers: { Accept: 'application/json', ...init.headers },
    })
  } catch {
    return { ok: false, error: { kind: 'network', message: 'Nie udało się połączyć z serwerem.' } }
  }

  if (response.ok) return { ok: true, data: undefined }

  const body: unknown = await response.json().catch(() => null)
  if (isApiErrorEnvelope(body)) {
    return {
      ok: false,
      error: { kind: 'api', status: response.status, code: body.error.code, message: body.error.message },
    }
  }

  return {
    ok: false,
    error: { kind: 'invalid-response', status: response.status, message: 'Serwer zwrócił nieznany błąd.' },
  }
}
