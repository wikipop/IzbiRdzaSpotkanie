export interface EventSession {
  userId: string
  nick: string
  token: string
  garnekDismissed?: boolean
}

export type TokenExpiryStatus =
  | { status: 'unknown' }
  | { status: 'valid' | 'expiring' | 'expired'; expiresAt: number }

const sessionPrefix = 'izbi-rdza:event-session:'

function isEventSession(value: unknown): value is EventSession {
  if (!value || typeof value !== 'object') return false
  const record = value as Record<string, unknown>
  return typeof record.userId === 'string'
    && typeof record.nick === 'string'
    && typeof record.token === 'string'
    && (record.garnekDismissed === undefined || typeof record.garnekDismissed === 'boolean')
}

export function readEventSession(eventId: string): EventSession | null {
  if (!import.meta.client) return null

  try {
    const raw = sessionStorage.getItem(`${sessionPrefix}${eventId}`)
    if (!raw) return null
    const parsed: unknown = JSON.parse(raw)
    if (isEventSession(parsed)) return parsed
    sessionStorage.removeItem(`${sessionPrefix}${eventId}`)
  } catch {
    // Storage can be unavailable in privacy modes; the page falls back to memory-only state.
  }
  return null
}

export function writeEventSession(eventId: string, session: EventSession): boolean {
  if (!import.meta.client) return false
  try {
    sessionStorage.setItem(`${sessionPrefix}${eventId}`, JSON.stringify(session))
    return true
  } catch {
    return false
  }
}

export function forgetEventSession(eventId: string): void {
  if (!import.meta.client) return
  try {
    sessionStorage.removeItem(`${sessionPrefix}${eventId}`)
  } catch {
    // In-memory state is still cleared by the caller.
  }
}

export function tokenExpiryStatus(
  token: string,
  nowSeconds = Math.floor(Date.now() / 1000),
  warningWindowSeconds = 24 * 60 * 60,
): TokenExpiryStatus {
  try {
    const payloadPart = token.split('.')[1]
    if (!payloadPart) return { status: 'unknown' }
    const normalized = payloadPart.replaceAll('-', '+').replaceAll('_', '/')
    const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, '=')
    const payload: unknown = JSON.parse(atob(padded))
    if (!payload || typeof payload !== 'object' || !('exp' in payload)) return { status: 'unknown' }
    const expiresAt = (payload as Record<string, unknown>).exp
    if (typeof expiresAt !== 'number' || !Number.isFinite(expiresAt)) return { status: 'unknown' }
    if (expiresAt <= nowSeconds) return { status: 'expired', expiresAt }
    if (expiresAt - nowSeconds <= warningWindowSeconds) return { status: 'expiring', expiresAt }
    return { status: 'valid', expiresAt }
  } catch {
    return { status: 'unknown' }
  }
}
