<script setup lang="ts">
import type { DeepReadonly } from 'vue'
import type { AvailabilityIntent, CellWriteState, SlotReadState } from '~/composables/useEventSchedule'
import { availabilityLevel, type ParticipantSummary, type SlotSummary } from '~/domain/availability'
import { formatSlot, weekdayLabels } from '~/domain/slots'
import { weekdays, type AvailabilityState, type SlotId, type Weekday } from '~/types/api/contracts'

const props = defineProps<{
  slots: SlotId[]
  reads: DeepReadonly<Partial<Record<SlotId, SlotReadState>>>
  summaries: SlotSummary[]
  participants: ParticipantSummary[]
  participantCount: number
  busy: boolean
  editable: boolean
  currentUserId?: string
  comparisonUserId?: string
  writeStates: DeepReadonly<Partial<Record<SlotId, CellWriteState>>>
}>()

const emit = defineEmits<{
  retry: [slot: SlotId]
  edit: [slot: SlotId, desired: AvailabilityIntent]
  retryWrite: [slot: SlotId]
}>()

const configuredSlots = computed(() => new Set(props.slots))
const visibleDays = computed(() => weekdays.filter(day => (
  props.slots.some(slot => slot.startsWith(`${day}-`))
)))
const visibleHours = computed(() => [...new Set(props.slots.map(slot => Number(slot.slice(3))))].sort((a, b) => a - b))
const summaryBySlot = computed(() => new Map(props.summaries.map(summary => [summary.slot, summary])))
const tableRoot = ref<HTMLElement | null>(null)
const activeSlot = ref<SlotId | null>(null)
const inspectedSlot = ref<SlotId | null>(null)
const paintingActive = ref(false)
const paintingIntent = ref<AvailabilityIntent>(null)
const paintedSlots = new Set<SlotId>()
let paintingPointerId: number | null = null
let suppressPaintClick = false
let restoreFocusedSlot: SlotId | null = null
const firstNavigableSlot = computed(() => props.slots.find(slot => (
  props.reads[slot]?.status === 'success' || props.reads[slot]?.status === 'failed'
)) ?? null)
const currentTabStop = computed(() => (
  activeSlot.value
  && props.slots.includes(activeSlot.value)
  && (props.reads[activeSlot.value]?.status === 'success' || props.reads[activeSlot.value]?.status === 'failed')
    ? activeSlot.value
    : firstNavigableSlot.value
))
const inspectedSummary = computed(() => (
  inspectedSlot.value ? summaryBySlot.value.get(inspectedSlot.value) : undefined
))
const yesEntries = computed(() => inspectedSummary.value?.entries.filter(entry => entry.state === 'yes') ?? [])
const maybeEntries = computed(() => inspectedSummary.value?.entries.filter(entry => entry.state === 'maybe') ?? [])
const noEntryParticipants = computed(() => {
  const visibleEntryIds = new Set(inspectedSummary.value?.entries.map(entry => entry.user_id) ?? [])
  return props.participants.filter(participant => !visibleEntryIds.has(participant.userId))
})

onBeforeUpdate(() => {
  if (!import.meta.client || !(document.activeElement instanceof HTMLElement)) return
  const focusedSlot = document.activeElement.dataset.slot as SlotId | undefined
  restoreFocusedSlot = focusedSlot && props.slots.includes(focusedSlot) ? focusedSlot : null
})

onUpdated(() => {
  if (!restoreFocusedSlot) return
  document.querySelector<HTMLButtonElement>(`button[data-slot="${restoreFocusedSlot}"]`)?.focus()
  restoreFocusedSlot = null
})

function slotAt(day: Weekday, hour: number): SlotId | null {
  const slot = `${day}-${String(hour).padStart(2, '0')}` as SlotId
  return configuredSlots.value.has(slot) ? slot : null
}

function cellLabel(slot: SlotId): string {
  const state = props.reads[slot]
  if (!state || state.status === 'queued' || state.status === 'loading') return `${formatSlot(slot)}: sprawdzanie osobnego serwisu godziny`
  if (state.status === 'failed') return `${formatSlot(slot)}: nie udało się odczytać danych z serwisu godziny, spróbuj ponownie`

  const summary = summaryBySlot.value.get(slot)
  return `${formatSlot(slot)}: serwis online; ${summary?.yesCount ?? 0} tak, ${summary?.maybeCount ?? 0} może`
}

function colorClass(slot: SlotId): string {
  return `availability-level-${colorLevel(slot)}`
}

function colorLevel(slot: SlotId): number {
  return availabilityLevel(summaryBySlot.value.get(slot), props.participantCount)
}

function confirmedState(slot: SlotId): AvailabilityState | null {
  if (!props.currentUserId) return null
  return summaryBySlot.value.get(slot)?.entries.find(entry => entry.user_id === props.currentUserId)?.state ?? null
}

function comparisonState(slot: SlotId): AvailabilityState | null {
  if (!props.comparisonUserId) return null
  return summaryBySlot.value.get(slot)?.entries.find(entry => entry.user_id === props.comparisonUserId)?.state ?? null
}

function displayedState(slot: SlotId): AvailabilityIntent {
  return currentWrite(slot)?.desired ?? confirmedState(slot)
}

function currentWrite(slot: SlotId): DeepReadonly<CellWriteState> | undefined {
  const write = props.writeStates[slot]
  return write?.ownerUserId === props.currentUserId ? write : undefined
}

function stateLabel(state: AvailabilityIntent): string {
  if (state === 'yes') return 'tak'
  if (state === 'maybe') return 'może'
  return 'brak'
}

function nextState(slot: SlotId): AvailabilityIntent {
  const current = displayedState(slot)
  if (current === null) return 'yes'
  if (current === 'yes') return 'maybe'
  return null
}

function editableLabel(slot: SlotId): string {
  const write = currentWrite(slot)
  const suffix = write?.status === 'saving'
    ? ', zapisywanie'
    : write?.status === 'failed' ? `, ${write.message}` : `; kliknij, aby ustawić ${stateLabel(nextState(slot))}`
  return `${cellLabel(slot)}. Twój wybór: ${stateLabel(displayedState(slot))}${suffix}`
}

function activateCell(slot: SlotId): void {
  activeSlot.value = slot
  inspectedSlot.value = slot
  if (currentWrite(slot)?.status === 'failed') emit('retryWrite', slot)
  else emit('edit', slot, nextState(slot))
}

function inspectCell(slot: SlotId): void {
  activeSlot.value = slot
  inspectedSlot.value = slot
}

function handleCellClick(event: MouseEvent, slot: SlotId): void {
  inspectCell(slot)
  if (suppressPaintClick) {
    suppressPaintClick = false
    return
  }
  activateCell(slot)
}

function beginPaint(event: PointerEvent, slot: SlotId): void {
  if (!props.editable || event.button !== 0 || event.pointerType === 'touch') return
  // A failed cell is retried by its click handler; painting never starts from it.
  if (currentWrite(slot)?.status === 'failed') return
  inspectCell(slot)
  suppressPaintClick = true

  event.preventDefault()
  paintingIntent.value = nextState(slot)
  paintingActive.value = true
  paintingPointerId = event.pointerId
  paintedSlots.clear()
  paintedSlots.add(slot)
  emit('edit', slot, paintingIntent.value)
  tableRoot.value?.setPointerCapture?.(event.pointerId)
}

function continuePaint(event: PointerEvent): void {
  if (!paintingActive.value || event.pointerId !== paintingPointerId) return
  const target = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLButtonElement>('button[data-slot]')
  const slot = target?.dataset.slot as SlotId | undefined
  if (!slot || paintedSlots.has(slot) || !props.slots.includes(slot) || props.reads[slot]?.status !== 'success') return
  paintedSlots.add(slot)
  inspectedSlot.value = slot
  emit('edit', slot, paintingIntent.value)
}

function finishPaint(event: PointerEvent): void {
  if (!paintingActive.value || event.pointerId !== paintingPointerId) return
  if (tableRoot.value?.hasPointerCapture?.(event.pointerId)) tableRoot.value.releasePointerCapture(event.pointerId)
  paintingActive.value = false
  paintingPointerId = null
  paintedSlots.clear()
  setTimeout(() => { suppressPaintClick = false }, 0)
}

function cellTabIndex(slot: SlotId): 0 | -1 {
  return slot === currentTabStop.value ? 0 : -1
}

async function focusSlot(slot: SlotId): Promise<void> {
  activeSlot.value = slot
  inspectedSlot.value = slot
  await nextTick()
  tableRoot.value?.querySelector<HTMLButtonElement>(`button[data-slot="${slot}"]`)?.focus()
}

function navigableSlot(dayIndex: number, hourIndex: number): SlotId | null {
  const day = visibleDays.value[dayIndex]
  const hour = visibleHours.value[hourIndex]
  if (!day || hour === undefined) return null
  const slot = slotAt(day, hour)
  return slot && (props.reads[slot]?.status === 'success' || props.reads[slot]?.status === 'failed') ? slot : null
}

function handleCellKeydown(event: KeyboardEvent, slot: SlotId): void {
  const day = slot.slice(0, 2) as Weekday
  const hour = Number(slot.slice(3))
  const startDay = visibleDays.value.indexOf(day)
  const startHour = visibleHours.value.indexOf(hour)
  let next: SlotId | null = null

  if (event.key === 'Home' || event.key === 'End') {
    const direction = event.key === 'Home' ? 1 : -1
    for (
      let dayIndex = event.key === 'Home' ? 0 : visibleDays.value.length - 1;
      dayIndex >= 0 && dayIndex < visibleDays.value.length;
      dayIndex += direction
    ) {
      next = navigableSlot(dayIndex, startHour)
      if (next) break
    }
  } else if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
    const direction = event.key === 'ArrowRight' ? 1 : -1
    for (let dayIndex = startDay + direction; dayIndex >= 0 && dayIndex < visibleDays.value.length; dayIndex += direction) {
      next = navigableSlot(dayIndex, startHour)
      if (next) break
    }
  } else if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
    const direction = event.key === 'ArrowDown' ? 1 : -1
    for (let hourIndex = startHour + direction; hourIndex >= 0 && hourIndex < visibleHours.value.length; hourIndex += direction) {
      next = navigableSlot(startDay, hourIndex)
      if (next) break
    }
  } else {
    return
  }

  event.preventDefault()
  if (next) void focusSlot(next)
}
</script>

<template>
  <div class="grid items-start gap-4 lg:grid-cols-[minmax(0,1fr)_17rem]">
    <div
      ref="tableRoot"
      class="schedule-scroll overflow-auto rounded-2xl border border-rust/15 bg-white/75"
      @pointermove="continuePaint"
      @pointerup="finishPaint"
      @pointercancel="finishPaint"
    >
      <table class="w-full min-w-[36rem] table-fixed border-separate border-spacing-0 text-center md:min-w-0">
      <caption class="sr-only">
        Dostępność uczestników według dnia i godziny.
        <template v-if="editable">Klikaj albo przeciągaj po polach. Między polami poruszaj się strzałkami, klawisze Home i End przechodzą na początek i koniec wiersza, a wybór zmieniaj klawiszem Enter lub Spacja.</template>
      </caption>
      <thead>
        <tr>
          <th scope="col" class="sticky top-0 left-0 z-20 w-14 border-r border-b border-rust/15 bg-paper px-1.5 py-3 text-xs text-ink-soft sm:w-16 sm:text-sm">
            Godz.
          </th>
          <th
            v-for="day in visibleDays"
            :key="day"
            scope="col"
            class="sticky top-0 z-10 border-b border-rust/15 bg-paper px-1 py-3 text-xs font-bold text-rust-dark sm:text-sm"
          >
            <span aria-hidden="true">{{ day.toUpperCase() }}</span>
            <span class="sr-only">{{ weekdayLabels[day] }}</span>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="hour in visibleHours" :key="hour">
          <th scope="row" class="sticky left-0 z-10 border-r border-b border-rust/10 bg-paper px-1 py-3 text-xs font-semibold text-ink-soft sm:text-sm">
            {{ String(hour).padStart(2, '0') }}:00
          </th>
          <td v-for="day in visibleDays" :key="day" class="border-r border-b border-rust/10 p-1.5 align-middle">
            <template v-for="slot in [slotAt(day, hour)]" :key="slot ?? 'empty'">
              <template v-if="slot">
                <button
                  v-if="reads[slot]?.status === 'failed'"
                  type="button"
                  :disabled="busy"
                  class="relative min-h-16 w-full rounded-xl border border-red-300 bg-red-50 px-2 py-2 text-xs font-bold text-red-800 hover:bg-red-100"
                  :aria-label="cellLabel(slot)"
                  :data-slot="slot"
                  :tabindex="cellTabIndex(slot)"
                  @focus="inspectCell(slot)"
                  @keydown="handleCellKeydown($event, slot)"
                  @click="inspectCell(slot); emit('retry', slot)"
                >
                  <span class="absolute top-1 left-1 size-2 rounded-full bg-red-500" aria-hidden="true" />
                  <span aria-hidden="true">!</span>
                  <span class="mt-1 block">Ponów</span>
                </button>
                <button
                  v-else-if="reads[slot]?.status === 'success' && editable"
                  type="button"
                  class="relative grid min-h-14 w-full place-items-center rounded-lg border-2 px-1 py-1.5 transition sm:min-h-16 sm:rounded-xl"
                  :class="[
                    colorClass(slot),
                    currentWrite(slot)?.status === 'failed' ? 'border-red-600' : 'border-transparent',
                  ]"
                  :aria-label="editableLabel(slot)"
                  :title="editableLabel(slot)"
                  :data-slot="slot"
                  :tabindex="cellTabIndex(slot)"
                  @focus="inspectCell(slot)"
                  @keydown="handleCellKeydown($event, slot)"
                  @pointerdown="beginPaint($event, slot)"
                  @click="handleCellClick($event, slot)"
                >
                  <span class="absolute top-1 left-1 size-2 rounded-full bg-emerald-500 shadow-[0_0_0_2px_white]" aria-hidden="true" />
                  <span aria-hidden="true" class="text-xs font-black sm:text-sm">{{ stateLabel(displayedState(slot)) }}</span>
                  <span v-if="colorLevel(slot) > 0" aria-hidden="true" class="absolute bottom-1 flex gap-0.5">
                    <span
                      v-for="level in 4"
                      :key="level"
                      class="size-1 rounded-full border border-current sm:size-1.5"
                      :class="level <= colorLevel(slot) ? 'bg-current' : 'bg-transparent opacity-35'"
                    />
                  </span>
                  <span v-if="currentWrite(slot)?.status === 'saving'" aria-hidden="true" class="absolute top-1 right-1 size-2 animate-pulse rounded-full bg-current" />
                  <span v-else-if="currentWrite(slot)?.status === 'failed'" aria-hidden="true" class="absolute top-0.5 right-1 text-sm font-black">!</span>
                </button>
                <button
                  v-else-if="reads[slot]?.status === 'success'"
                  type="button"
                  class="relative grid min-h-14 w-full place-items-center rounded-lg px-1.5 py-2 transition-colors sm:min-h-16 sm:rounded-xl"
                  :class="colorClass(slot)"
                  :aria-label="cellLabel(slot)"
                  :title="cellLabel(slot)"
                  :data-slot="slot"
                  :tabindex="cellTabIndex(slot)"
                  @focus="inspectCell(slot)"
                  @keydown="handleCellKeydown($event, slot)"
                  @click="inspectCell(slot)"
                >
                  <span class="absolute top-1 left-1 size-2 rounded-full bg-emerald-500 shadow-[0_0_0_2px_white]" aria-hidden="true" />
                  <span v-if="comparisonUserId" aria-hidden="true" class="text-xs font-black sm:text-sm">
                    {{ stateLabel(comparisonState(slot)) }}
                  </span>
                  <span
                    v-else-if="colorLevel(slot) > 0"
                    aria-hidden="true"
                    class="flex items-center justify-center gap-0.5"
                  >
                    <span
                      v-for="level in 4"
                      :key="level"
                      class="size-1.5 rounded-full border border-current sm:size-2"
                      :class="level <= colorLevel(slot) ? 'bg-current' : 'bg-transparent opacity-35'"
                    />
                  </span>
                  <span v-else aria-hidden="true" class="text-ink-soft/55">—</span>
                </button>
                <button
                  v-else
                  type="button"
                  class="relative min-h-16 w-full animate-pulse rounded-xl bg-patina/45"
                  :aria-label="cellLabel(slot)"
                  aria-disabled="true"
                  :data-slot="slot"
                  :tabindex="cellTabIndex(slot)"
                  @focus="inspectCell(slot)"
                  @keydown="handleCellKeydown($event, slot)"
                >
                  <span class="absolute top-1 left-1 size-2 rounded-full bg-amber-400" aria-hidden="true" />
                  <span class="sr-only">Ładowanie</span>
                </button>
              </template>
              <span v-else class="text-ink-soft/40" aria-label="Poza zakresem spotkania">—</span>
            </template>
          </td>
        </tr>
      </tbody>
      </table>
    </div>

    <aside v-if="inspectedSlot && reads[inspectedSlot]?.status === 'success'" class="sticky top-2 z-20 order-first rounded-2xl border border-rust/15 bg-paper p-4 text-left shadow-[0_0.75rem_2rem_rgb(29_36_34/10%)] lg:order-last lg:top-4" aria-live="polite">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <h3 class="font-display text-xl font-bold text-rust-dark">{{ formatSlot(inspectedSlot) }}</h3>
        <span class="inline-flex items-center gap-1.5 text-xs font-bold text-emerald-700"><span class="size-2 rounded-full bg-emerald-500" /> serwis online</span>
      </div>
      <div v-if="participants.length" class="mt-3 grid gap-3 text-sm sm:grid-cols-3">
        <div>
          <p class="font-bold text-emerald-800">Tak · {{ yesEntries.length }}</p>
          <p class="mt-1 text-ink-soft">{{ yesEntries.map(entry => entry.nick).join(', ') || '—' }}</p>
        </div>
        <div>
          <p class="font-bold text-amber-800">Może · {{ maybeEntries.length }}</p>
          <p class="mt-1 text-ink-soft">{{ maybeEntries.map(entry => entry.nick).join(', ') || '—' }}</p>
        </div>
        <div>
          <p class="font-bold text-ink">Brak wpisu · {{ noEntryParticipants.length }}</p>
          <p class="mt-1 text-ink-soft">{{ noEntryParticipants.map(participant => participant.nick).join(', ') || '—' }}</p>
        </div>
      </div>
      <p v-else class="mt-2 text-sm text-ink-soft">Nikt jeszcze nie zaznaczył żadnej dostępności.</p>
      <p class="mt-3 text-xs text-ink-soft">Lista obejmuje osoby widoczne w co najmniej jednej odpowiedzi; API nie ujawnia uczestników bez żadnego wpisu.</p>
    </aside>
    <aside v-else class="sticky top-2 z-20 order-first rounded-2xl border border-dashed border-rust/25 bg-white/95 p-4 text-sm text-ink-soft lg:order-last lg:top-4">
      Wybierz godzinę, aby zobaczyć odpowiedzi wybranych osób.
    </aside>
  </div>
</template>
