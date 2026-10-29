<script setup lang="ts">
import { validateCreateEvent, validateCreateEventSlots, type ValidationErrors } from '~/domain/validation'
import { slotId, slotsForRange, weekdayLabels } from '~/domain/slots'
import type { ApiFailure } from '~/services/api/result'
import { weekdays, type SlotId, type Weekday } from '~/types/api/contracts'

useHead({
  title: 'Izbi Rdza Spotkanie — znajdź wspólną godzinę',
  meta: [{ name: 'description', content: 'Zaplanuj spotkanie i znajdź godzinę pasującą całej grupie.' }],
})

const { gateway } = useApi()
const name = ref('')
const selectedDays = ref<Weekday[]>(['pn', 'wt', 'sr', 'cz', 'pt'])
const from = ref(9)
const to = ref(17)
const createMode = ref<'range' | 'slots'>('range')
const selectedSlots = ref<SlotId[]>([])
const explicitActiveSlot = ref<SlotId>('pn-00')
const errors = ref<ValidationErrors>({})
const submitError = ref('')
const pending = ref(false)

const hourOptions = Array.from({ length: 25 }, (_, hour) => ({
  value: hour,
  label: `${String(hour).padStart(2, '0')}:00`,
}))

function toggleDay(day: Weekday): void {
  selectedDays.value = selectedDays.value.includes(day)
    ? selectedDays.value.filter(value => value !== day)
    : [...selectedDays.value, day]
}

function setCreateMode(mode: 'range' | 'slots'): void {
  createMode.value = mode
  errors.value = {}
  if (mode === 'slots' && selectedSlots.value.length === 0) {
    selectedSlots.value = slotsForRange(selectedDays.value, Number(from.value), Number(to.value))
  }
}

function toggleSlot(slot: SlotId): void {
  selectedSlots.value = selectedSlots.value.includes(slot)
    ? selectedSlots.value.filter(value => value !== slot)
    : [...selectedSlots.value, slot]
  errors.value.slots = undefined
}

function toggleWholeDay(day: Weekday): void {
  const daySlots = Array.from({ length: 24 }, (_, hour) => slotId(day, hour))
  const allSelected = daySlots.every(slot => selectedSlots.value.includes(slot))
  selectedSlots.value = allSelected
    ? selectedSlots.value.filter(slot => !daySlots.includes(slot))
    : [...new Set([...selectedSlots.value, ...daySlots])]
  errors.value.slots = undefined
}

function explicitTabIndex(slot: SlotId): 0 | -1 {
  return slot === explicitActiveSlot.value ? 0 : -1
}

async function focusExplicitSlot(slot: SlotId): Promise<void> {
  explicitActiveSlot.value = slot
  await nextTick()
  document.getElementById(`slot-${slot}`)?.focus()
}

function handleExplicitSlotKeydown(event: KeyboardEvent, day: Weekday, hour: number): void {
  const dayIndex = weekdays.indexOf(day)
  let nextDay = dayIndex
  let nextHour = hour

  if (event.key === 'ArrowLeft') nextHour = Math.max(0, hour - 1)
  else if (event.key === 'ArrowRight') nextHour = Math.min(23, hour + 1)
  else if (event.key === 'ArrowUp') nextDay = Math.max(0, dayIndex - 1)
  else if (event.key === 'ArrowDown') nextDay = Math.min(weekdays.length - 1, dayIndex + 1)
  else if (event.key === 'Home') nextHour = 0
  else if (event.key === 'End') nextHour = 23
  else return

  event.preventDefault()
  void focusExplicitSlot(slotId(weekdays[nextDay]!, nextHour))
}

function messageForFailure(error: ApiFailure): string {
  if (error.kind === 'api') {
    if (error.code === 'invalid_event_name') return 'Sprawdź nazwę spotkania.'
    if (error.code === 'invalid_slot' || error.code === 'invalid_body') return 'Sprawdź dni i godziny.'
    if (error.code === 'internal') return 'Serwer ma chwilowy problem. Spróbuj ponownie.'
  }
  return error.message
}

async function createEvent(): Promise<void> {
  if (pending.value) return
  submitError.value = ''
  errors.value = createMode.value === 'range'
    ? validateCreateEvent({
        name: name.value,
        days: selectedDays.value,
        from: Number(from.value),
        to: Number(to.value),
      })
    : validateCreateEventSlots(name.value, selectedSlots.value)

  if (Object.keys(errors.value).length > 0) {
    await nextTick()
    const firstInvalidId = errors.value.name
      ? 'event-name'
      : errors.value.days ? 'day-pn' : errors.value.slots ? 'slot-pn-00' : 'from-hour'
    document.getElementById(firstInvalidId)?.focus()
    return
  }

  pending.value = true
  const result = await gateway.createEvent(createMode.value === 'range'
    ? {
        name: name.value,
        days: selectedDays.value,
        hours: { from: Number(from.value), to: Number(to.value) },
      }
    : { name: name.value, slots: selectedSlots.value })
  pending.value = false

  if (!result.ok) {
    submitError.value = messageForFailure(result.error)
    return
  }

  await navigateTo(`/e/${result.data.event_id}`)
}
</script>

<template>
  <div class="min-h-screen px-4 py-8 sm:px-8 sm:py-14">
    <main id="main-content" class="mx-auto grid w-full max-w-6xl gap-8 lg:grid-cols-[0.85fr_1.15fr] lg:items-center">
      <section aria-labelledby="page-title" class="py-4">
        <p class="mb-4 inline-flex rounded-full bg-blush px-4 py-2 text-sm font-bold tracking-wide text-berry-dark">
          ✦ Znajdź wspólną godzinę
        </p>
        <h1 id="page-title" class="max-w-xl font-display text-5xl leading-[1.05] font-bold text-berry-dark sm:text-7xl">
          Kiedy się widzimy?
        </h1>
        <p class="mt-6 max-w-xl text-lg leading-8 text-ink-soft">
          Wybierz dni i godziny, a potem podeślij znajomym link.
        </p>
        <p class="mt-8 text-sm font-semibold text-ink-soft">Bez konta i zbędnego kombinowania.</p>
      </section>

      <section class="paper-card relative overflow-hidden p-5 sm:p-8" aria-labelledby="form-title">
        <div class="absolute -top-3 right-8 h-10 w-28 rotate-2 bg-sakura/45" aria-hidden="true" />
        <h2 id="form-title" class="font-display text-3xl font-bold text-berry-dark">Nowe spotkanie</h2>
        <p class="mt-2 text-ink-soft">Ustal, jakie terminy bierzecie pod uwagę.</p>

        <form class="mt-7 space-y-7" novalidate @submit.prevent="createEvent">
          <div>
            <label for="event-name" class="mb-2 block font-bold">Nazwa spotkania</label>
            <input
              id="event-name"
              v-model="name"
              class="field"
              type="text"
              autocomplete="off"
              placeholder="np. Wspólne izbienie"
              :aria-invalid="Boolean(errors.name)"
              :aria-describedby="errors.name ? 'event-name-error' : undefined"
            >
            <p v-if="errors.name" id="event-name-error" class="error-text mt-2 text-sm">{{ errors.name }}</p>
          </div>

          <fieldset>
            <legend class="font-bold">Sposób wyboru terminów</legend>
            <div class="mt-3 grid grid-cols-2 gap-2 rounded-2xl bg-blush/60 p-1.5">
              <button type="button" class="min-h-11 rounded-xl px-3 py-2 text-sm font-bold" :class="createMode === 'range' ? 'bg-white text-berry-dark shadow-sm' : 'text-ink-soft'" :aria-pressed="createMode === 'range'" @click="setCreateMode('range')">
                Zakres godzin
              </button>
              <button type="button" class="min-h-11 rounded-xl px-3 py-2 text-sm font-bold" :class="createMode === 'slots' ? 'bg-white text-berry-dark shadow-sm' : 'text-ink-soft'" :aria-pressed="createMode === 'slots'" @click="setCreateMode('slots')">
                Konkretne godziny
              </button>
            </div>
          </fieldset>

          <fieldset v-if="createMode === 'range'" :aria-describedby="errors.days ? 'event-days-error' : undefined">
            <legend class="font-bold">Dni tygodnia</legend>
            <div class="mt-3 grid grid-cols-4 gap-2 sm:grid-cols-7">
              <button
                v-for="day in weekdays"
                :id="`day-${day}`"
                :key="day"
                type="button"
                class="min-h-11 rounded-xl border px-2 py-2 text-sm font-bold transition"
                :class="selectedDays.includes(day)
                  ? 'border-berry-dark bg-berry text-white shadow-sm'
                  : 'border-berry/20 bg-white text-ink hover:border-berry'"
                :aria-pressed="selectedDays.includes(day)"
                @click="toggleDay(day)"
              >
                <span aria-hidden="true">{{ day.toUpperCase() }}</span>
                <span class="sr-only">{{ weekdayLabels[day] }}</span>
              </button>
            </div>
            <p v-if="errors.days" id="event-days-error" class="error-text mt-2 text-sm">{{ errors.days }}</p>
          </fieldset>

          <fieldset v-if="createMode === 'range'" :aria-describedby="errors.hours ? 'event-hours-error' : 'event-hours-hint'">
            <legend class="font-bold">Zakres godzin</legend>
            <p id="event-hours-hint" class="mt-1 text-sm text-ink-soft">Godzina końcowa jest pierwszą godziną poza spotkaniem.</p>
            <div class="mt-3 grid grid-cols-[1fr_auto_1fr] items-center gap-3">
              <div>
                <label class="sr-only" for="from-hour">Od godziny</label>
                <select id="from-hour" v-model.number="from" class="field" :aria-invalid="Boolean(errors.hours)" :aria-describedby="errors.hours ? 'event-hours-error' : 'event-hours-hint'">
                  <option v-for="hour in hourOptions.slice(0, 24)" :key="hour.value" :value="hour.value">{{ hour.label }}</option>
                </select>
              </div>
              <span aria-hidden="true" class="font-display text-xl text-berry">→</span>
              <div>
                <label class="sr-only" for="to-hour">Do godziny, wyłącznie</label>
                <select id="to-hour" v-model.number="to" class="field" :aria-invalid="Boolean(errors.hours)" :aria-describedby="errors.hours ? 'event-hours-error' : 'event-hours-hint'">
                  <option v-for="hour in hourOptions.slice(1)" :key="hour.value" :value="hour.value">{{ hour.label }}</option>
                </select>
              </div>
            </div>
            <p v-if="errors.hours" id="event-hours-error" class="error-text mt-2 text-sm">{{ errors.hours }}</p>
          </fieldset>

          <fieldset v-else :aria-describedby="errors.slots ? 'event-slots-error event-slots-hint' : 'event-slots-hint'">
            <legend class="font-bold">Konkretne godziny</legend>
            <p id="event-slots-hint" class="mt-1 text-sm text-ink-soft">Wybierz dowolne godziny. Nie muszą tworzyć ciągłego zakresu. Po siatce poruszaj się strzałkami, a Home i End przechodzą na początek lub koniec dnia.</p>
            <div class="mt-3 max-h-[28rem] space-y-3 overflow-y-auto rounded-2xl border border-berry/15 bg-white/70 p-3">
              <section v-for="day in weekdays" :key="day" :aria-labelledby="`explicit-${day}`">
                <div class="flex items-center justify-between gap-3">
                  <h3 :id="`explicit-${day}`" class="font-bold text-berry-dark">{{ weekdayLabels[day] }}</h3>
                  <button type="button" class="min-h-11 px-2 text-xs font-bold text-berry-dark underline underline-offset-2" :aria-label="`${Array.from({ length: 24 }, (_, hour) => slotId(day, hour)).every(slot => selectedSlots.includes(slot)) ? 'Wyczyść' : 'Wybierz'} cały dzień: ${weekdayLabels[day]}`" @click="toggleWholeDay(day)">
                    {{ Array.from({ length: 24 }, (_, hour) => slotId(day, hour)).every(slot => selectedSlots.includes(slot)) ? 'Wyczyść dzień' : 'Cały dzień' }}
                  </button>
                </div>
                <div class="mt-1 grid grid-cols-4 gap-1.5 sm:grid-cols-8">
                  <button
                    v-for="hour in 24"
                    :id="`slot-${day}-${String(hour - 1).padStart(2, '0')}`"
                    :key="hour"
                    type="button"
                    class="min-h-11 rounded-lg border px-1 text-xs font-bold"
                    :class="selectedSlots.includes(slotId(day, hour - 1)) ? 'border-berry-dark bg-berry text-white' : 'border-berry/15 bg-white text-ink'"
                    :aria-pressed="selectedSlots.includes(slotId(day, hour - 1))"
                    :tabindex="explicitTabIndex(slotId(day, hour - 1))"
                    @focus="explicitActiveSlot = slotId(day, hour - 1)"
                    @keydown="handleExplicitSlotKeydown($event, day, hour - 1)"
                    @click="toggleSlot(slotId(day, hour - 1))"
                  >
                    {{ String(hour - 1).padStart(2, '0') }}
                  </button>
                </div>
              </section>
            </div>
            <p class="mt-2 text-sm text-ink-soft">Wybrano: <strong>{{ selectedSlots.length }}</strong></p>
            <p v-if="errors.slots" id="event-slots-error" class="error-text mt-2 text-sm">{{ errors.slots }}</p>
          </fieldset>

          <p v-if="submitError" role="alert" class="error-text rounded-xl border border-red-200 bg-red-50 p-3 text-sm">
            {{ submitError }}
          </p>

          <button class="button-primary w-full" type="submit" :disabled="pending">
            {{ pending ? 'Tworzę spotkanie…' : 'Utwórz spotkanie' }}
          </button>
        </form>
      </section>
    </main>
  </div>
</template>
