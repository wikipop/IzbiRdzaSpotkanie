import { createGatewayClient } from '~/services/api/gateway'
import { createSlotClient } from '~/services/api/slot'
import type { ApiEnvironment, ApiMode } from '~/services/api/urls'

export function useApi() {
  const config = useRuntimeConfig()
  const configuredMode = config.public.apiMode
  if (configuredMode !== 'development' && configuredMode !== 'production') {
    throw new Error(`Niepoprawny NUXT_PUBLIC_API_MODE: ${configuredMode}`)
  }
  const mode: ApiMode = configuredMode
  const environment: ApiEnvironment = {
    mode,
    gatewayBase: config.public.gatewayBase || undefined,
  }

  return {
    gateway: createGatewayClient(environment),
    slots: createSlotClient(environment),
  }
}
