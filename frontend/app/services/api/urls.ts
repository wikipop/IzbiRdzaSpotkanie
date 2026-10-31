import { isSlotId } from '~/domain/slots'
import type { SlotId } from '~/types/api/contracts'

export type ApiMode = 'development' | 'production'

export interface ApiEnvironment {
  mode: ApiMode
  gatewayBase?: string
  slotBaseTemplate?: string
}

const defaultSlotBaseTemplate = 'https://{slot}-meet.izbi.beer'

const defaultGatewayBase: Record<ApiMode, string> = {
  development: 'http://localhost:8080',
  production: 'https://api-meet.izbi.beer',
}

function withoutTrailingSlash(value: string): string {
  return value.replace(/\/+$/, '')
}

export function gatewayBaseUrl(environment: ApiEnvironment): string {
  return withoutTrailingSlash(environment.gatewayBase || defaultGatewayBase[environment.mode])
}

export function slotBaseUrl(environment: ApiEnvironment, slot: SlotId): string {
  if (!isSlotId(slot)) throw new Error(`Invalid slot identifier: ${slot}`)

  if (environment.mode === 'development') {
    return `${gatewayBaseUrl(environment)}/slots/${slot}`
  }

  return withoutTrailingSlash((environment.slotBaseTemplate || defaultSlotBaseTemplate).replaceAll('{slot}', slot))
}

export function eventPath(eventId: string): string {
  return `/events/${encodeURIComponent(eventId)}`
}
