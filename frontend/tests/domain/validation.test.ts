import { describe, expect, it } from 'vitest'
import { unicodeLength, validateCreateEvent, validateCreateEventSlots, validateJoin } from '~/domain/validation'

describe('create event validation', () => {
  it('counts Unicode code points rather than UTF-16 units', () => {
    expect(unicodeLength('🍵')).toBe(1)
  })

  it('accepts a valid exclusive hour range', () => {
    expect(validateCreateEvent({ name: 'Herbatka', days: ['pn'], from: 9, to: 10 })).toEqual({})
  })

  it('rejects empty names, days and invalid hour ranges', () => {
    expect(validateCreateEvent({ name: '   ', days: [], from: 10, to: 10 })).toEqual({
      name: 'Wpisz nazwę spotkania.',
      days: 'Wybierz co najmniej jeden dzień.',
      hours: 'Wybierz poprawny zakres godzin. Godzina końcowa nie należy do zakresu.',
    })
  })

  it('accepts 00–24 and rejects non-integer hours', () => {
    expect(validateCreateEvent({ name: 'Cały dzień', days: ['sb'], from: 0, to: 24 })).toEqual({})
    expect(validateCreateEvent({ name: 'Błąd', days: ['sb'], from: 9.5, to: 11 })).toHaveProperty('hours')
  })

  it('validates explicit slots without allowing an empty event', () => {
    expect(validateCreateEventSlots('Noc filmowa', ['pt-20', 'sb-12'])).toEqual({})
    expect(validateCreateEventSlots('Noc filmowa', [])).toEqual({
      slots: 'Wybierz co najmniej jedną godzinę.',
    })
  })
})

describe('join validation', () => {
  it('matches nick and password Unicode limits', () => {
    expect(validateJoin('🍵'.repeat(32), '🔐'.repeat(128))).toEqual({})
    expect(validateJoin('🍵'.repeat(33), '🔐'.repeat(129))).toEqual({
      nick: 'Pseudonim może mieć maksymalnie 32 znaki.',
      password: 'Hasło może mieć maksymalnie 128 znaków.',
    })
  })

  it('rejects empty and control-character nicknames', () => {
    expect(validateJoin('   ', '')).toHaveProperty('nick')
    expect(validateJoin('Iza\nTest', '')).toHaveProperty('nick')
  })
})
