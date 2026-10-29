import { weekdays, type SlotId, type Weekday } from '~/types/api/contracts'

export const weekdayLabels: Record<Weekday, string> = {
  pn: 'Poniedziałek',
  wt: 'Wtorek',
  sr: 'Środa',
  cz: 'Czwartek',
  pt: 'Piątek',
  sb: 'Sobota',
  nd: 'Niedziela',
}

const slotPattern = /^(pn|wt|sr|cz|pt|sb|nd)-(\d{2})$/

export function isWeekday(value: unknown): value is Weekday {
  return typeof value === 'string' && weekdays.includes(value as Weekday)
}

export function isSlotId(value: unknown): value is SlotId {
  if (typeof value !== 'string') return false
  const match = slotPattern.exec(value)
  return match !== null && Number(match[2]) <= 23
}

export function slotIndex(slot: SlotId): number {
  const [day, hour] = slot.split('-') as [Weekday, string]
  return weekdays.indexOf(day) * 24 + Number(hour)
}

export function compareSlots(left: SlotId, right: SlotId): number {
  return slotIndex(left) - slotIndex(right)
}

export function formatSlot(slot: SlotId): string {
  const [day, hour] = slot.split('-') as [Weekday, string]
  return `${weekdayLabels[day]}, ${hour}:00–${String(Number(hour) + 1).padStart(2, '0')}:00`
}

export function slotId(day: Weekday, hour: number): SlotId {
  if (!Number.isInteger(hour) || hour < 0 || hour > 23) throw new Error('Invalid slot hour')
  return `${day}-${String(hour).padStart(2, '0')}`
}

export function slotsForRange(days: Weekday[], from: number, to: number): SlotId[] {
  if (!Number.isInteger(from) || !Number.isInteger(to) || from < 0 || to > 24 || from >= to) return []
  return weekdays.flatMap(day => (
    days.includes(day)
      ? Array.from({ length: to - from }, (_, offset) => slotId(day, from + offset))
      : []
  ))
}
