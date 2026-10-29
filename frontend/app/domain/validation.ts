import type { SlotId, Weekday } from '~/types/api/contracts'

export interface CreateEventFormValue {
  name: string
  days: Weekday[]
  from: number
  to: number
}

export interface ValidationErrors {
  name?: string
  days?: string
  hours?: string
  slots?: string
}

export interface JoinValidationErrors {
  nick?: string
  password?: string
}

export function unicodeLength(value: string): number {
  return Array.from(value).length
}

export function validateCreateEvent(value: CreateEventFormValue): ValidationErrors {
  const errors: ValidationErrors = {}
  const nameLength = unicodeLength(value.name.trim())

  if (nameLength === 0) errors.name = 'Wpisz nazwę spotkania.'
  else if (nameLength > 100) errors.name = 'Nazwa może mieć maksymalnie 100 znaków.'

  if (value.days.length === 0) errors.days = 'Wybierz co najmniej jeden dzień.'

  if (!Number.isInteger(value.from) || !Number.isInteger(value.to)
    || value.from < 0 || value.to > 24 || value.from >= value.to) {
    errors.hours = 'Wybierz poprawny zakres godzin. Godzina końcowa nie należy do zakresu.'
  }

  return errors
}

export function validateCreateEventSlots(name: string, slots: SlotId[]): ValidationErrors {
  const errors: ValidationErrors = {}
  const nameLength = unicodeLength(name.trim())

  if (nameLength === 0) errors.name = 'Wpisz nazwę spotkania.'
  else if (nameLength > 100) errors.name = 'Nazwa może mieć maksymalnie 100 znaków.'
  if (slots.length === 0) errors.slots = 'Wybierz co najmniej jedną godzinę.'

  return errors
}

export function validateJoin(nick: string, password: string): JoinValidationErrors {
  const errors: JoinValidationErrors = {}
  const trimmedNick = nick.trim()
  const nickLength = unicodeLength(trimmedNick)

  if (nickLength === 0) errors.nick = 'Wpisz pseudonim.'
  else if (nickLength > 32) errors.nick = 'Pseudonim może mieć maksymalnie 32 znaki.'
  else if (/\p{Cc}/u.test(trimmedNick)) errors.nick = 'Pseudonim nie może zawierać znaków sterujących.'

  if (unicodeLength(password) > 128) errors.password = 'Hasło może mieć maksymalnie 128 znaków.'

  return errors
}
