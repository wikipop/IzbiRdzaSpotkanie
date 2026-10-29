export const weekdays = ['pn', 'wt', 'sr', 'cz', 'pt', 'sb', 'nd'] as const

export type Weekday = (typeof weekdays)[number]
export type AvailabilityState = 'yes' | 'maybe'
export type SlotId = `${Weekday}-${string}`

export type CreateEventRequest =
  | { name: string; slots: SlotId[] }
  | { name: string; days: Weekday[]; hours: { from: number; to: number } }

export interface CreateEventResponse {
  event_id: string
}

export interface EventResponse {
  event_id: string
  name: string
  slots: SlotId[]
}

export interface JoinRequest {
  nick: string
  password?: string
}

export interface JoinResponse {
  user_id: string
  token: string
}

export interface AvailabilityEntry {
  user_id: string
  nick: string
  state: AvailabilityState
  updated_at: number
}

export interface AvailabilityResponse {
  slot: SlotId
  entries: AvailabilityEntry[]
}

export const apiErrorCodes = [
  'invalid_body',
  'invalid_event_name',
  'invalid_slot',
  'invalid_nick',
  'invalid_password',
  'invalid_event_id',
  'missing_token',
  'invalid_token',
  'invalid_credentials',
  'event_mismatch',
  'slot_out_of_range',
  'event_not_found',
  'internal',
] as const

export type ApiErrorCode = (typeof apiErrorCodes)[number]

export interface ApiErrorEnvelope {
  error: {
    code: ApiErrorCode
    message: string
  }
}
