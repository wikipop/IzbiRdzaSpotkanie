import { describe, expect, it } from 'vitest'
import { compareSlots, formatSlot, isSlotId, slotId, slotIndex, slotsForRange } from '~/domain/slots'

describe('slot domain', () => {
  it.each(['pn-00', 'sr-09', 'nd-23'])('accepts valid slot %s', (slot) => {
    expect(isSlotId(slot)).toBe(true)
  })

  it.each(['PN-09', 'sr-9', 'sr-24', 'xx-10', 'sr-0a'])('rejects invalid slot %s', (slot) => {
    expect(isSlotId(slot)).toBe(false)
  })

  it('uses the backend Monday-to-Sunday order', () => {
    expect(slotIndex('pn-00')).toBe(0)
    expect(slotIndex('sr-09')).toBe(57)
    expect(slotIndex('nd-23')).toBe(167)
    expect(['nd-01', 'pn-20', 'wt-00'].sort(compareSlots)).toEqual(['pn-20', 'wt-00', 'nd-01'])
  })

  it('formats an hourly range in Polish', () => {
    expect(formatSlot('sr-09')).toBe('Środa, 09:00–10:00')
    expect(formatSlot('nd-23')).toBe('Niedziela, 23:00–24:00')
  })

  it('builds normalized explicit slots from an exclusive range', () => {
    expect(slotId('sr', 9)).toBe('sr-09')
    expect(slotsForRange(['wt', 'pn'], 9, 11)).toEqual(['pn-09', 'pn-10', 'wt-09', 'wt-10'])
    expect(slotsForRange(['pn'], 10, 10)).toEqual([])
  })
})
