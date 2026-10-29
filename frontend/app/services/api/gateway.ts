import { isCreateEventResponse, isEventResponse, isJoinResponse } from './guards'
import { requestJson, type ApiResult } from './result'
import { eventPath, gatewayBaseUrl, type ApiEnvironment } from './urls'
import type {
  CreateEventRequest,
  CreateEventResponse,
  EventResponse,
  JoinRequest,
  JoinResponse,
} from '~/types/api/contracts'

export interface GatewayClient {
  createEvent(body: CreateEventRequest): Promise<ApiResult<CreateEventResponse>>
  getEvent(eventId: string): Promise<ApiResult<EventResponse>>
  joinEvent(eventId: string, body: JoinRequest): Promise<ApiResult<JoinResponse>>
}

export function createGatewayClient(environment: ApiEnvironment): GatewayClient {
  const baseUrl = gatewayBaseUrl(environment)

  return {
    createEvent: body => requestJson(
      `${baseUrl}/events`,
      { method: 'POST', body: JSON.stringify(body) },
      isCreateEventResponse,
    ),
    getEvent: eventId => requestJson(
      `${baseUrl}${eventPath(eventId)}`,
      { method: 'GET' },
      isEventResponse,
    ),
    joinEvent: (eventId, body) => requestJson(
      `${baseUrl}${eventPath(eventId)}/join`,
      { method: 'POST', body: JSON.stringify(body) },
      isJoinResponse,
    ),
  }
}
