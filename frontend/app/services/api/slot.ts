import { isAvailabilityEntry, isAvailabilityResponse } from './guards'
import { requestJson, requestNoContent, type ApiResult } from './result'
import { eventPath, slotBaseUrl, type ApiEnvironment } from './urls'
import type {
  AvailabilityEntry,
  AvailabilityResponse,
  AvailabilityState,
  SlotId,
} from '~/types/api/contracts'

export interface SlotClient {
  getAvailability(slot: SlotId, eventId: string, signal?: AbortSignal): Promise<ApiResult<AvailabilityResponse>>
  setMyAvailability(
    slot: SlotId,
    eventId: string,
    token: string,
    state: AvailabilityState,
  ): Promise<ApiResult<AvailabilityEntry>>
  clearMyAvailability(slot: SlotId, eventId: string, token: string): Promise<ApiResult<undefined>>
}

function authorization(token: string): HeadersInit {
  return { Authorization: `Bearer ${token}` }
}

export function createSlotClient(environment: ApiEnvironment): SlotClient {
  const eventUrl = (slot: SlotId, eventId: string) => `${slotBaseUrl(environment, slot)}${eventPath(eventId)}`

  return {
    // Public reads intentionally receive no token and no Authorization header.
    getAvailability: (slot, eventId, signal) => requestJson(
      eventUrl(slot, eventId),
      { method: 'GET', signal },
      (value): value is AvailabilityResponse => isAvailabilityResponse(value) && value.slot === slot,
    ),
    setMyAvailability: (slot, eventId, token, state) => requestJson(
      `${eventUrl(slot, eventId)}/me`,
      { method: 'PUT', headers: authorization(token), body: JSON.stringify({ state }) },
      isAvailabilityEntry,
    ),
    clearMyAvailability: (slot, eventId, token) => requestNoContent(
      `${eventUrl(slot, eventId)}/me`,
      { method: 'DELETE', headers: authorization(token) },
    ),
  }
}
