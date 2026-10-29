import { describe, expect, it } from 'vitest'
import { eventPath, gatewayBaseUrl, slotBaseUrl } from '~/services/api/urls'

describe('API URL builder', () => {
  it('builds development monolith URLs', () => {
    const environment = { mode: 'development' as const }
    expect(gatewayBaseUrl(environment)).toBe('http://localhost:8080')
    expect(slotBaseUrl(environment, 'sr-09')).toBe('http://localhost:8080/slots/sr-09')
  })

  it('builds production gateway and first-level slot hosts', () => {
    const environment = { mode: 'production' as const }
    expect(gatewayBaseUrl(environment)).toBe('https://api-meet.izbi.beer')
    expect(slotBaseUrl(environment, 'nd-23')).toBe('https://nd-23-meet.izbi.beer')
  })

  it('honors a gateway override without leaking it into production slot hosts', () => {
    expect(gatewayBaseUrl({ mode: 'development', gatewayBase: 'http://api.test/' })).toBe('http://api.test')
    expect(slotBaseUrl({ mode: 'production', gatewayBase: 'https://gateway.test' }, 'pn-00'))
      .toBe('https://pn-00-meet.izbi.beer')
  })

  it('rejects malformed slot identifiers and encodes event IDs', () => {
    expect(() => slotBaseUrl({ mode: 'development' }, 'pn-99')).toThrow('Invalid slot identifier')
    expect(eventPath('event/unsafe')).toBe('/events/event%2Funsafe')
  })
})
