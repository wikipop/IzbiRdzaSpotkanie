<script setup lang="ts">
import { validateCreateEvent, validateCreateEventSlots, type ValidationErrors } from '~/domain/validation'
import { slotId, slotsForRange, weekdayLabels } from '~/domain/slots'
import type { ApiFailure } from '~/services/api/result'
import { gatewayBaseUrl, type ApiMode } from '~/services/api/urls'
import { weekdays, type SlotId, type Weekday } from '~/types/api/contracts'

useHead({
  title: 'Izbi Rdza Meet — znajdź wspólną godzinę',
  meta: [{ name: 'description', content: 'Zaplanuj spotkanie i znajdź godzinę pasującą całej grupie.' }],
})

const { gateway } = useApi()
const config = useRuntimeConfig()
const repositoryUrl = 'https://github.com/wikipop/IzbiRdzaSpotkanie'
const apiDocsUrl = `${gatewayBaseUrl({
  mode: config.public.apiMode as ApiMode,
  gatewayBase: config.public.gatewayBase || undefined,
})}/docs`
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

// A sample week for the hero: warmer evenings on weekdays, daytime at weekends,
// and one hour that suits everybody.
const sharedHour = { day: 'pt', hour: 19 } as const
const heroWeek = weekdays.map(day => ({
  day,
  hours: Array.from({ length: 24 }, (_, hour) => {
    if (day === sharedHour.day && hour === sharedHour.hour) return 'shared'
    const weekend = day === 'sb' || day === 'nd'
    const busy = weekend ? hour >= 11 && hour <= 21 : hour >= 17 && hour <= 22
    if (hour < 8) return 0
    return busy ? 2 + ((hour + day.charCodeAt(0)) % 3) : (hour + day.charCodeAt(1)) % 4 === 0 ? 1 : 0
  }),
}))

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

const hours = Array.from({ length: 24 }, (_, hour) => hour)
const selectedSet = computed(() => new Set(selectedSlots.value))
// A drag selects the rectangle between its first cell and the current one, so fast mouse
// moves that skip cells still fill every hour in between.
let paint: { target: boolean; anchor: SlotId; base: Set<SlotId> } | null = null
let pointerPainted = false

function hourLabel(hour: number): string {
  return `${String(hour).padStart(2, '0')}:00`
}

function setSlots(slots: SlotId[], selected: boolean): void {
  const next = new Set(selectedSlots.value)
  for (const slot of slots) {
    if (selected) next.add(slot)
    else next.delete(slot)
  }
  selectedSlots.value = [...next]
  errors.value.slots = undefined
}

function daySlots(day: Weekday): SlotId[] {
  return hours.map(hour => slotId(day, hour))
}

function hourSlots(hour: number): SlotId[] {
  return weekdays.map(day => slotId(day, hour))
}

function allSelected(slots: SlotId[]): boolean {
  return slots.every(slot => selectedSet.value.has(slot))
}

function toggleWholeHour(hour: number): void {
  setSlots(hourSlots(hour), !allSelected(hourSlots(hour)))
}

// Mouse and pen paint by dragging; touch keeps native scrolling and toggles on tap.
function beginSlotPaint(event: PointerEvent, slot: SlotId): void {
  if (event.pointerType === 'touch' || event.button !== 0) return
  event.preventDefault()
  paint = { target: !selectedSet.value.has(slot), anchor: slot, base: new Set(selectedSlots.value) }
  pointerPainted = true
  explicitActiveSlot.value = slot
  continueSlotPaint(slot)
}

function continueSlotPaint(slot: SlotId): void {
  if (!paint) return
  const [anchorDay, anchorHour] = [weekdays.indexOf(paint.anchor.slice(0, 2) as Weekday), Number(paint.anchor.slice(3))]
  const [day, hour] = [weekdays.indexOf(slot.slice(0, 2) as Weekday), Number(slot.slice(3))]
  const next = new Set(paint.base)
  for (let d = Math.min(anchorDay, day); d <= Math.max(anchorDay, day); d++) {
    for (let h = Math.min(anchorHour, hour); h <= Math.max(anchorHour, hour); h++) {
      const cell = slotId(weekdays[d]!, h)
      if (paint.target) next.add(cell)
      else next.delete(cell)
    }
  }
  selectedSlots.value = [...next]
  errors.value.slots = undefined
}

function endSlotPaint(): void {
  paint = null
  // The click that follows pointerup has already been handled by the paint.
  setTimeout(() => { pointerPainted = false }, 0)
}

function clickSlot(slot: SlotId): void {
  explicitActiveSlot.value = slot
  if (pointerPainted) return
  toggleSlot(slot)
}

onMounted(() => window.addEventListener('pointerup', endSlotPaint))
onBeforeUnmount(() => window.removeEventListener('pointerup', endSlotPaint))

function toggleWholeDay(day: Weekday): void {
  setSlots(daySlots(day), !allSelected(daySlots(day)))
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

  if (event.key === 'ArrowLeft') nextDay = Math.max(0, dayIndex - 1)
  else if (event.key === 'ArrowRight') nextDay = Math.min(weekdays.length - 1, dayIndex + 1)
  else if (event.key === 'ArrowUp') nextHour = Math.max(0, hour - 1)
  else if (event.key === 'ArrowDown') nextHour = Math.min(23, hour + 1)
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
  <div class="flex min-h-screen flex-col px-4 pt-8 pb-6 sm:px-8 sm:pt-14">
    <main id="main-content" class="mx-auto grid w-full max-w-6xl flex-1 content-start gap-8 lg:grid-cols-[0.85fr_1.15fr] lg:items-start">
      <section aria-labelledby="page-title" class="min-w-0 py-2">
        <p class="wordmark inline-flex items-center gap-2 font-display text-lg font-extrabold text-rust-dark">
          <img src="/assets/ferris/ferris.svg" alt="" class="wordmark-crab h-6 w-auto"> Izbi Rdza Meet
        </p>
        <h1 id="page-title" class="mt-10 max-w-xl font-display text-5xl leading-[0.98] font-extrabold tracking-[-0.03em] text-ink sm:text-7xl">
          Kiedy się widzimy?
        </h1>
        <p class="mt-5 max-w-md text-lg leading-8 text-ink-soft">
          Zaznacz dni i godziny, wyślij znajomym link, a mapa pokaże, kiedy pasuje wszystkim.
        </p>

        <figure class="mt-10 max-w-md">
          <div class="hero-week" aria-hidden="true">
            <template v-for="row in heroWeek" :key="row.day">
              <span class="hero-week-day">{{ row.day }}</span>
              <span
                v-for="(level, hour) in row.hours"
                :key="hour"
                class="hero-week-cell"
                :class="level === 'shared' ? 'is-shared' : `is-${level}`"
              >
                <img v-if="level === 'shared'" src="/assets/ferris/ferris.svg" alt="" class="hero-crab">
              </span>
            </template>
          </div>
          <figcaption class="mt-4 text-sm leading-6 text-ink-soft">
            Tydzień ma 168 godzin i każdą obsługuje osobny serwis.
            <span class="font-semibold text-verdigris">Zielone pole</span> to godzina, która pasuje całej grupie.
          </figcaption>
        </figure>
      </section>

      <section class="paper-card relative overflow-hidden p-5 sm:p-8" aria-labelledby="form-title">
        <h2 id="form-title" class="font-display text-3xl font-bold text-rust-dark">Nowe spotkanie</h2>
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
            <div class="mt-3 grid grid-cols-2 gap-2 rounded-2xl bg-flake/60 p-1.5">
              <button type="button" class="min-h-11 rounded-xl px-3 py-2 text-sm font-bold" :class="createMode === 'range' ? 'bg-white text-rust-dark shadow-sm' : 'text-ink-soft'" :aria-pressed="createMode === 'range'" @click="setCreateMode('range')">
                Zakres godzin
              </button>
              <button type="button" class="min-h-11 rounded-xl px-3 py-2 text-sm font-bold" :class="createMode === 'slots' ? 'bg-white text-rust-dark shadow-sm' : 'text-ink-soft'" :aria-pressed="createMode === 'slots'" @click="setCreateMode('slots')">
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
                  ? 'border-rust-dark bg-rust text-white shadow-sm'
                  : 'border-rust/20 bg-white text-ink hover:border-rust'"
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
              <span aria-hidden="true" class="font-display text-xl text-rust">→</span>
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
            <p id="event-slots-hint" class="mt-1 text-sm text-ink-soft">
              Kliknij godzinę albo przeciągnij po siatce. Nagłówek dnia lub godziny zaznacza cały rząd, a strzałki przesuwają między polami.
            </p>
            <div class="slot-picker-scroll mt-3">
              <div class="slot-picker" role="group" aria-label="Godziny tygodnia">
                <span class="slot-picker-corner" aria-hidden="true" />
                <button
                  v-for="day in weekdays"
                  :key="day"
                  type="button"
                  class="slot-picker-head"
                  :aria-label="`${allSelected(daySlots(day)) ? 'Wyczyść' : 'Wybierz'} cały dzień: ${weekdayLabels[day]}`"
                  @click="toggleWholeDay(day)"
                >
                  {{ day.toUpperCase() }}
                </button>
                <template v-for="hour in hours" :key="hour">
                  <button
                    type="button"
                    class="slot-picker-hour"
                    :aria-label="`${allSelected(hourSlots(hour)) ? 'Wyczyść' : 'Wybierz'} godzinę ${hourLabel(hour)} we wszystkie dni`"
                    @click="toggleWholeHour(hour)"
                  >
                    {{ String(hour).padStart(2, '0') }}
                  </button>
                  <button
                    v-for="day in weekdays"
                    :id="`slot-${slotId(day, hour)}`"
                    :key="day"
                    type="button"
                    class="slot-picker-cell"
                    :class="{ 'is-selected': selectedSet.has(slotId(day, hour)) }"
                    :aria-label="`${weekdayLabels[day]}, ${hourLabel(hour)}`"
                    :aria-pressed="selectedSet.has(slotId(day, hour))"
                    :tabindex="explicitTabIndex(slotId(day, hour))"
                    @focus="explicitActiveSlot = slotId(day, hour)"
                    @keydown="handleExplicitSlotKeydown($event, day, hour)"
                    @pointerdown="beginSlotPaint($event, slotId(day, hour))"
                    @pointerenter="continueSlotPaint(slotId(day, hour))"
                    @click="clickSlot(slotId(day, hour))"
                  />
                </template>
              </div>
            </div>
            <div class="mt-2 flex items-center justify-between gap-3 text-sm text-ink-soft">
              <p>Wybrano: <strong>{{ selectedSlots.length }}</strong> godz.</p>
              <button v-if="selectedSlots.length" type="button" class="min-h-11 px-2 font-bold text-rust-dark underline underline-offset-2" @click="selectedSlots = []">
                Wyczyść wszystko
              </button>
            </div>
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

    <footer class="mx-auto mt-16 w-full max-w-6xl border-t border-rust-dark/15 pt-6 text-sm text-ink-soft">
      <nav aria-label="Projekt">
        <ul class="flex flex-wrap gap-x-6 gap-y-2 font-semibold">
          <li><a class="footer-link" :href="repositoryUrl">Kod na GitHubie</a></li>
          <li>Frontend: <a class="footer-link" href="https://github.com/wojpo">wojpo</a></li>
          <li>Backend: <a class="footer-link" href="https://github.com/wikipop">wikipop</a></li>
          <li><a class="footer-link" :href="apiDocsUrl">Dokumentacja API</a></li>
          <li><a class="footer-link" :href="`${repositoryUrl}/tree/master/backend/docs/adr`">Decyzje architektoniczne</a></li>
        </ul>
      </nav>
      <p class="mt-3 text-xs">
        Ferris: Karen Rustad Tölva, <a class="footer-link" href="https://rustacean.net/">rustacean.net</a>, domena publiczna (CC0).
      </p>
    </footer>
  </div>
</template>

<style scoped>
.slot-picker-scroll {
  max-height: 24rem;
  overflow-y: auto;
  border: 1px solid rgb(92 35 16 / 15%);
  border-radius: 1rem;
  background: rgb(255 255 255 / 70%);
  padding: 0 0.5rem 0.5rem;
}

.slot-picker {
  display: grid;
  grid-template-columns: 2.25rem repeat(7, minmax(0, 1fr));
  gap: 3px;
  user-select: none;
}

.slot-picker-corner,
.slot-picker-head {
  position: sticky;
  top: 0;
  z-index: 1;
  background: #fbfcfb;
  padding-top: 0.5rem;
}

.slot-picker-head {
  min-height: 2.5rem;
  border-radius: 0 0 0.4rem 0.4rem;
  color: var(--color-rust-dark);
  font-size: 0.75rem;
  font-weight: 800;
}

.slot-picker-hour {
  color: var(--color-ink-soft);
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  font-weight: 700;
}

.slot-picker-head:hover,
.slot-picker-hour:hover {
  color: var(--color-rust);
}

.slot-picker-cell {
  min-height: 1.6rem;
  border-radius: 0.3rem;
  background: var(--color-steel);
  transition: background 80ms ease;
}

.slot-picker-cell:hover {
  background: var(--color-flake);
}

.slot-picker-cell.is-selected {
  background: var(--color-rust);
}

.slot-picker-cell.is-selected:hover {
  background: var(--color-rust-dark);
}

.footer-link {
  color: var(--color-rust-dark);
  text-decoration: underline;
  text-decoration-color: rgb(163 64 29 / 35%);
  text-underline-offset: 3px;
}

.footer-link:hover {
  text-decoration-color: currentcolor;
}

.hero-week {
  display: grid;
  grid-template-columns: 1.6rem repeat(24, minmax(0, 1fr));
  gap: 3px;
}

.hero-week-day {
  align-self: center;
  color: var(--color-ink-soft);
  font-size: 0.7rem;
  font-weight: 700;
}

.hero-week-cell {
  aspect-ratio: 1;
  border-radius: 2px;
  background: var(--color-steel);
}

.hero-week-cell.is-1 { background: #ebb89c; }
.hero-week-cell.is-2 { background: #d07a52; }
.hero-week-cell.is-3 { background: var(--color-rust); }
.hero-week-cell.is-4 { background: var(--color-rust-dark); }

.hero-week-cell {
  position: relative;
}

.hero-crab {
  position: absolute;
  bottom: 110%;
  left: 50%;
  width: 1.75rem;
  max-width: none;
  translate: -50% 0;
  animation: hero-crab-hop 1.4s cubic-bezier(0.3, 0, 0.4, 1) infinite;
}

@keyframes hero-crab-hop {
  0%, 40%, 100% { transform: translateY(0); }
  65% { transform: translateY(-0.9rem) rotate(-12deg); }
}

.wordmark-crab {
  display: inline-block;
  transition: transform 200ms ease;
}

.wordmark:hover .wordmark-crab {
  transform: translateY(-0.35rem) rotate(-15deg);
}

.hero-week-cell.is-shared {
  background: var(--color-verdigris);
  box-shadow: 0 0 0 2px var(--color-paper), 0 0 0 4px var(--color-verdigris);
  animation: shared-pulse 2.4s ease-in-out 1s 2;
}

@keyframes shared-pulse {
  50% {
    box-shadow: 0 0 0 2px var(--color-paper), 0 0 0 7px rgb(44 122 111 / 35%);
  }
}
</style>
