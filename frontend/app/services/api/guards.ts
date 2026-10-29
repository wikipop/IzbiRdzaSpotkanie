import { isSlotId } from '~/domain/slots'
import type {
  AvailabilityEntry,
  AvailabilityResponse,
  CreateEventResponse,
  EventResponse,
  JoinResponse,
} from '~/types/api/contracts'

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

export function isCreateEventResponse(value: unknown): value is CreateEventResponse {
  return isRecord(value) && typeof value.event_id === 'string'
}

export function isEventResponse(value: unknown): value is EventResponse {
  return isRecord(value)
    && typeof value.event_id === 'string'
    && typeof value.name === 'string'
    && Array.isArray(value.slots)
    && value.slots.every(isSlotId)
}

export function isJoinResponse(value: unknown): value is JoinResponse {
  return isRecord(value) && typeof value.user_id === 'string' && typeof value.token === 'string'
}

export function isAvailabilityEntry(value: unknown): value is AvailabilityEntry {
  return isRecord(value)
    && typeof value.user_id === 'string'
    && typeof value.nick === 'string'
    && (value.state === 'yes' || value.state === 'maybe')
    && typeof value.updated_at === 'number'
    && Number.isInteger(value.updated_at)
}

export function isAvailabilityResponse(value: unknown): value is AvailabilityResponse {
  return isRecord(value)
    && isSlotId(value.slot)
    && Array.isArray(value.entries)
    && value.entries.every(isAvailabilityEntry)
}
