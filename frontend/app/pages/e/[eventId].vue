<script setup lang="ts">
import { forgetEventSession, readEventSession, tokenExpiryStatus, writeEventSession, type EventSession } from '~/services/session'
import { validateJoin } from '~/domain/validation'
import { evaluateSelectedOverlap, filterAvailability, type ParticipantSummary } from '~/domain/availability'
import { shouldShowGarnek } from '~/domain/garnek'
import type { ApiFailure, ApiResult } from '~/services/api/result'
import type { AvailabilityState, EventResponse, SlotId } from '~/types/api/contracts'

const route = useRoute()
const { gateway } = useApi()
const eventId = computed(() => String(route.params.eventId))

const event = ref<EventResponse | null>(null)
const loadState = ref<'loading' | 'ready' | 'not-found' | 'invalid-link' | 'failed'>('loading')
const loadError = ref('')
const session = ref<EventSession | null>(null)
const nick = ref('')
const password = ref('')
const joinPending = ref(false)
const joinError = ref('')
const nickError = ref('')
const passwordError = ref('')
const sessionNotice = ref('')
const reauthNotice = ref(false)
const copyStatus = ref('')
const joinDialog = ref<HTMLDialogElement | null>(null)
const scheduleFocusTarget = ref<HTMLElement | null>(null)
const participantFilterIds = ref<string[]>([])
const comparisonRequested = ref(false)
const garnekDismissed = ref(false)
const automaticGarnekCheckArmed = ref(false)
const automaticGarnekDelayElapsed = ref(false)
const nowSeconds = ref(Math.floor(Date.now() / 1000))
const reauthUserId = ref<string | null>(null)
let dialogReturnTarget: HTMLElement | null = null
let expiryTimer: ReturnType<typeof setInterval> | undefined
let automaticGarnekTimer: ReturnType<typeof setTimeout> | undefined
let loadVersion = 0
let joinVersion = 0
const schedule = reactive(useEventSchedule(eventId, event))
const selectableParticipants = computed<ParticipantSummary[]>(() => {
  const participants = [...schedule.participants]
  if (session.value && !participants.some(participant => participant.userId === session.value?.userId)) {
    participants.push({ userId: session.value.userId, nick: session.value.nick })
  }
  return participants.sort((left, right) => left.nick.localeCompare(right.nick, 'pl', { sensitivity: 'base' }))
})
const effectiveParticipantIds = computed(() => (
  participantFilterIds.value.length
    ? participantFilterIds.value
    : selectableParticipants.value.map(participant => participant.userId)
))
const selectedParticipants = computed(() => {
  const selected = new Set(effectiveParticipantIds.value)
  return selectableParticipants.value.filter(participant => selected.has(participant.userId))
})
const garnekParticipantIds = computed(() => selectableParticipants.value.map(participant => participant.userId))
const sessionExpiry = computed(() => session.value
  ? tokenExpiryStatus(session.value.token, nowSeconds.value)
  : { status: 'unknown' as const })
const canEditSelectedView = computed(() => Boolean(
  session.value && effectiveParticipantIds.value.includes(session.value.userId),
))
const filteredSummaries = computed(() => filterAvailability(schedule.summaries, participantFilterIds.value))
const overlapEvaluation = computed(() => evaluateSelectedOverlap({
  summaries: schedule.summaries,
  participantIds: garnekParticipantIds.value,
  currentUserId: session.value?.userId,
  allSlotsKnown: Boolean(event.value)
    && !schedule.isLoading
    && schedule.failedSlots.length === 0
    && schedule.successfulCount === event.value?.slots.length,
  hasPendingWrites: Object.keys(schedule.writeStates).length > 0,
}))
const showGarnek = computed(() => shouldShowGarnek({
  comparisonRequested: comparisonRequested.value,
  evaluation: overlapEvaluation.value,
  dismissed: garnekDismissed.value,
}))
const canRequestComparison = computed(() => Boolean(
  session.value
  && garnekParticipantIds.value.length >= 2
  && session.value.userId && garnekParticipantIds.value.includes(session.value.userId)
  && !schedule.isLoading
  && schedule.failedSlots.length === 0
  && Object.keys(schedule.writeStates).length === 0,
))

useHead(() => ({ title: event.value ? `${event.value.name} — Izbi Rdza Spotkanie` : 'Spotkanie — Izbi Rdza Spotkanie' }))

function userMessage(error: ApiFailure): string {
  if (error.kind === 'api') {
    if (error.code === 'invalid_credentials') return 'Hasło do tego pseudonimu jest niepoprawne.'
    if (error.code === 'invalid_nick') return 'Pseudonim musi mieć od 1 do 32 znaków i nie może zawierać znaków sterujących.'
    if (error.code === 'invalid_password') return 'Hasło może mieć maksymalnie 128 znaków.'
    if (error.code === 'event_not_found') return 'To spotkanie nie istnieje.'
    if (error.code === 'internal') return 'Serwer ma chwilowy problem. Spróbuj ponownie.'
  }
  return error.message
}

async function loadEvent(): Promise<void> {
  const requestVersion = ++loadVersion
  const requestedEventId = eventId.value
  loadState.value = 'loading'
  loadError.value = ''
  const result = await gateway.getEvent(requestedEventId)
  applyEventResult(result, requestedEventId, requestVersion)
}

function applyEventResult(result: ApiResult<EventResponse>, requestedEventId: string, requestVersion: number): void {
  if (requestVersion !== loadVersion || requestedEventId !== eventId.value) return

  if (result.ok) {
    event.value = result.data
    loadState.value = 'ready'
    return
  }

  if (result.error.kind === 'api' && result.error.code === 'event_not_found') {
    loadState.value = 'not-found'
    return
  }
  if (result.error.kind === 'api' && result.error.code === 'invalid_event_id') {
    loadState.value = 'invalid-link'
    return
  }

  loadError.value = userMessage(result.error)
  loadState.value = 'failed'
}

function failInitialEventLoad(): void {
  loadError.value = 'Nie udało się otworzyć spotkania.'
  loadState.value = 'failed'
}

async function retryUnavailableSlots(): Promise<void> {
  // The retry button is removed as soon as failed cells become queued. Move focus first so it is
  // never dropped to the document body while the grid refreshes.
  scheduleFocusTarget.value?.focus()
  await schedule.retrySlots()
}

async function join(): Promise<void> {
  if (joinPending.value) return
  joinError.value = ''
  nickError.value = ''
  passwordError.value = ''
  sessionNotice.value = ''
  const fieldErrors = validateJoin(nick.value, password.value)
  nickError.value = fieldErrors.nick ?? ''
  passwordError.value = fieldErrors.password ?? ''
  const trimmedNick = nick.value.trim()
  if (nickError.value || passwordError.value) {
    await nextTick()
    document.getElementById(nickError.value ? 'nick' : 'password')?.focus()
    return
  }

  const requestedEventId = eventId.value
  const requestVersion = ++joinVersion
  joinPending.value = true
  const result = await gateway.joinEvent(requestedEventId, {
    nick: nick.value,
    ...(password.value ? { password: password.value } : {}),
  })
  if (requestVersion !== joinVersion || requestedEventId !== eventId.value) return
  joinPending.value = false

  if (!result.ok) {
    if (result.error.kind === 'api' && result.error.code === 'invalid_nick') {
      nickError.value = userMessage(result.error)
    } else if (result.error.kind === 'api' && result.error.code === 'invalid_password') {
      passwordError.value = userMessage(result.error)
    } else {
      joinError.value = userMessage(result.error)
    }
    return
  }

  joinDialog.value?.close()
  const previousSession = session.value
  const sameIdentity = previousSession?.userId === result.data.user_id || reauthUserId.value === result.data.user_id
  if ((previousSession || reauthUserId.value) && !sameIdentity) {
    schedule.discardWrites()
    if (automaticGarnekTimer) clearTimeout(automaticGarnekTimer)
    automaticGarnekCheckArmed.value = false
    automaticGarnekDelayElapsed.value = false
    comparisonRequested.value = false
  }
  const nextSession: EventSession = {
    userId: result.data.user_id,
    nick: trimmedNick,
    token: result.data.token,
    ...(sameIdentity && garnekDismissed.value ? { garnekDismissed: true } : {}),
  }
  session.value = nextSession
  participantFilterIds.value = [nextSession.userId]
  garnekDismissed.value = nextSession.garnekDismissed ?? false
  reauthUserId.value = null
  if (!writeEventSession(requestedEventId, nextSession)) {
    sessionNotice.value = 'Pamięć sesji jest zablokowana. Pozostaniesz zalogowany do odświeżenia strony.'
  }
  password.value = ''
  if (reauthNotice.value) {
    reauthNotice.value = false
    schedule.retryFailedWrites(nextSession)
  }
  await nextTick()
  scheduleFocusTarget.value?.focus()
}

async function openJoinDialog(): Promise<void> {
  dialogReturnTarget = document.activeElement instanceof HTMLElement ? document.activeElement : null
  if (!joinDialog.value?.open) joinDialog.value?.showModal()
  await nextTick()
  document.getElementById('nick')?.focus()
}

function handleJoinDialogClose(): void {
  if (joinPending.value) {
    joinVersion++
    joinPending.value = false
  }
  restoreDialogFocus()
}

function restoreDialogFocus(): void {
  if (dialogReturnTarget?.isConnected) dialogReturnTarget.focus()
  else scheduleFocusTarget.value?.focus()
  dialogReturnTarget = null
}

function forgetSession(): void {
  if (automaticGarnekTimer) clearTimeout(automaticGarnekTimer)
  automaticGarnekCheckArmed.value = false
  automaticGarnekDelayElapsed.value = false
  comparisonRequested.value = false
  schedule.discardWrites()
  session.value = null
  reauthUserId.value = null
  garnekDismissed.value = false
  sessionNotice.value = ''
  forgetEventSession(eventId.value)
  nick.value = ''
  password.value = ''
}

function participantIsSelected(userId: string): boolean {
  return participantFilterIds.value.length === 0 || participantFilterIds.value.includes(userId)
}

function toggleParticipant(userId: string): void {
  if (participantFilterIds.value.length === 0) participantFilterIds.value = [userId]
  else if (participantFilterIds.value.includes(userId)) {
    participantFilterIds.value = participantFilterIds.value.filter(id => id !== userId)
  } else participantFilterIds.value = [...participantFilterIds.value, userId]
}

function selectAllParticipants(): void {
  participantFilterIds.value = []
}

function editSlot(slot: SlotId, desired: AvailabilityState | null): void {
  if (!session.value) return
  schedule.setDesiredAvailability(slot, session.value, desired)
  armAutomaticGarnekCheck()
}

function armAutomaticGarnekCheck(): void {
  if (automaticGarnekCheckArmed.value || garnekDismissed.value) return
  automaticGarnekCheckArmed.value = true
  automaticGarnekTimer = setTimeout(() => {
    automaticGarnekDelayElapsed.value = true
    if (canRequestComparison.value) comparisonRequested.value = true
  }, 2_000)
}

async function focusGridSlot(slot?: SlotId): Promise<void> {
  await nextTick()
  const selector = slot ? `button[data-slot="${slot}"]` : 'button[data-slot]'
  document.querySelector<HTMLButtonElement>(`.schedule-scroll ${selector}`)?.focus()
}

function dismissGarnek(): void {
  if (overlapEvaluation.value.status === 'no-overlap') {
    if (automaticGarnekTimer) clearTimeout(automaticGarnekTimer)
    automaticGarnekCheckArmed.value = false
    garnekDismissed.value = true
    if (session.value) {
      session.value = { ...session.value, garnekDismissed: true }
      writeEventSession(eventId.value, session.value)
    }
  }
}

async function copyLink(): Promise<void> {
  if (!import.meta.client) return
  try {
    await navigator.clipboard.writeText(window.location.href)
    copyStatus.value = 'Link skopiowany.'
  } catch {
    copyStatus.value = 'Skopiuj adres z paska przeglądarki.'
  }
}

onMounted(() => {
  session.value = readEventSession(eventId.value)
  if (session.value) {
    nick.value = session.value.nick
    participantFilterIds.value = [session.value.userId]
    garnekDismissed.value = session.value.garnekDismissed ?? false
  }
  expiryTimer = setInterval(() => { nowSeconds.value = Math.floor(Date.now() / 1000) }, 60_000)
})

watch(eventId, async () => {
  joinVersion++
  joinPending.value = false
  event.value = null
  session.value = readEventSession(eventId.value)
  nick.value = session.value?.nick ?? ''
  password.value = ''
  joinError.value = ''
  nickError.value = ''
  passwordError.value = ''
  sessionNotice.value = ''
  participantFilterIds.value = session.value ? [session.value.userId] : []
  comparisonRequested.value = false
  automaticGarnekCheckArmed.value = false
  automaticGarnekDelayElapsed.value = false
  if (automaticGarnekTimer) clearTimeout(automaticGarnekTimer)
  garnekDismissed.value = session.value?.garnekDismissed ?? false
  joinDialog.value?.close()
  await loadEvent()
})

watch([selectableParticipants, () => schedule.isLoading], ([participants, isLoading]) => {
  if (isLoading) {
    comparisonRequested.value = false
    return
  }
  const visibleIds = new Set(participants.map(participant => participant.userId))
  participantFilterIds.value = participantFilterIds.value.filter(id => visibleIds.has(id))
})

watch(canRequestComparison, (ready) => {
  if (
    ready
    && automaticGarnekCheckArmed.value
    && automaticGarnekDelayElapsed.value
    && !garnekDismissed.value
  ) comparisonRequested.value = true
})


watch(() => schedule.reauthRequired, async (required) => {
  if (!required) return
  reauthUserId.value = session.value?.userId ?? null
  session.value = null
  reauthNotice.value = true
  forgetEventSession(eventId.value)
  await openJoinDialog()
})

onBeforeUnmount(() => {
  joinVersion++
  if (expiryTimer) clearInterval(expiryTimer)
  if (automaticGarnekTimer) clearTimeout(automaticGarnekTimer)
})

const initialEventId = eventId.value
const initialRequestVersion = ++loadVersion
loadState.value = 'loading'
const { data: initialEventResult } = await useAsyncData(
  `event-metadata:${initialEventId}`,
  () => gateway.getEvent(initialEventId),
)
if (initialEventResult.value) applyEventResult(initialEventResult.value, initialEventId, initialRequestVersion)
else failInitialEventLoad()
</script>

<template>
  <div class="min-h-screen px-3 py-6 sm:px-5 sm:py-8 lg:px-8">
    <main id="main-content" class="mx-auto w-full max-w-[96rem]">
      <NuxtLink to="/" class="inline-flex min-h-11 items-center font-bold text-berry-dark underline decoration-sakura decoration-2 underline-offset-4">
        ← Nowe spotkanie
      </NuxtLink>

      <section v-if="loadState === 'loading'" class="paper-card mt-8 p-8" aria-live="polite">
        <p class="font-display text-2xl text-berry-dark">Otwieram notes spotkania…</p>
      </section>

      <section v-else-if="loadState === 'not-found'" class="paper-card mt-8 p-8 text-center">
        <p class="text-5xl" aria-hidden="true">✉️</p>
        <h1 class="mt-4 font-display text-4xl font-bold text-berry-dark">Nie znaleźliśmy tego spotkania</h1>
        <p class="mt-3 text-ink-soft">Sprawdź link albo utwórz nowy plan.</p>
        <NuxtLink class="button-primary mt-6" to="/">Utwórz spotkanie</NuxtLink>
      </section>

      <section v-else-if="loadState === 'invalid-link'" class="paper-card mt-8 p-8 text-center">
        <p class="text-5xl" aria-hidden="true">✂️</p>
        <h1 class="mt-4 font-display text-4xl font-bold text-berry-dark">Ten link jest niepełny</h1>
        <p class="mt-3 text-ink-soft">Poproś o ponowne przesłanie całego adresu spotkania.</p>
        <NuxtLink class="button-primary mt-6" to="/">Utwórz nowe spotkanie</NuxtLink>
      </section>

      <section v-else-if="loadState === 'failed'" class="paper-card mt-8 p-8 text-center">
        <h1 class="font-display text-4xl font-bold text-berry-dark">Nie udało się otworzyć spotkania</h1>
        <p role="alert" class="error-text mt-3">{{ loadError }}</p>
        <button class="button-primary mt-6" type="button" @click="loadEvent">Spróbuj ponownie</button>
      </section>

      <template v-else-if="event">
        <header class="mt-6 flex flex-wrap items-end justify-between gap-5">
          <div>
            <p class="text-sm font-bold tracking-widest text-berry uppercase">Wspólny notes</p>
            <h1 class="mt-2 font-display text-4xl font-bold text-berry-dark sm:text-6xl">{{ event.name }}</h1>
            <p class="mt-3 text-sm text-ink-soft">
              {{ event.slots.length }} niezależnych usług godzinowych ·
              <strong class="text-emerald-700">{{ schedule.successfulCount }} online</strong>
              <template v-if="schedule.failedSlots.length"> · <strong class="text-red-700">{{ schedule.failedSlots.length }} z błędem odczytu</strong></template>
            </p>
          </div>
          <div class="flex flex-wrap gap-3">
            <button class="button-secondary" type="button" @click="copyLink">Skopiuj link</button>
          </div>
          <p class="sr-only" aria-live="polite">{{ copyStatus }}</p>
        </header>

        <section class="paper-card mt-6 min-w-0 p-3 sm:p-5 lg:p-6" aria-labelledby="schedule-title">
          <div class="flex flex-wrap items-center justify-between gap-4">
            <div>
              <p class="text-sm font-bold tracking-wide text-berry uppercase">Plan grupy</p>
              <h2 id="schedule-title" ref="scheduleFocusTarget" tabindex="-1" class="mt-1 font-display text-3xl font-bold text-berry-dark">Dostępność</h2>
            </div>
            <div class="flex flex-wrap items-center gap-3">
              <div v-if="session" class="inline-flex min-h-11 items-center gap-3 rounded-full border border-berry/15 bg-white px-4 text-sm shadow-sm">
                <span>Edytujesz jako <strong>{{ session.nick }}</strong></span>
                <button type="button" class="font-bold text-berry-dark underline underline-offset-2" @click="forgetSession">Wyjdź</button>
              </div>
              <button v-else class="button-primary" type="button" @click="openJoinDialog">Dodaj swoją dostępność</button>
              <button class="button-secondary" type="button" :disabled="schedule.isLoading" @click="schedule.refreshAll">
                {{ schedule.isLoading ? 'Odświeżam…' : 'Odśwież' }}
              </button>
            </div>
          </div>

          <p v-if="sessionNotice" class="mt-4 rounded-xl bg-amber-50 p-3 text-sm text-ink" role="status">{{ sessionNotice }}</p>
          <div v-if="session && (sessionExpiry.status === 'expired' || sessionExpiry.status === 'expiring')" class="mt-4 flex flex-wrap items-center justify-between gap-3 rounded-xl border border-amber-200 bg-amber-50 p-3 text-sm text-ink" role="status">
            <p>{{ sessionExpiry.status === 'expired' ? 'Ta sesja prawdopodobnie wygasła. Serwer potwierdzi to przy zapisie.' : 'Ta sesja wygaśnie w ciągu 24 godzin.' }}</p>
            <button type="button" class="font-bold text-berry-dark underline underline-offset-2" @click="openJoinDialog">Dołącz ponownie</button>
          </div>

          <div
            v-if="schedule.failedSlots.length"
            class="mt-4 flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-red-200 bg-red-50 p-4"
            role="status"
          >
            <p class="text-sm text-ink">
              Nie udało się odczytać danych z {{ schedule.failedSlots.length }} {{ schedule.failedSlots.length === 1 ? 'serwisu godziny' : 'serwisów godzin' }}.
              Czerwone pola są nieznane, a nie puste.
            </p>
            <button class="button-secondary" type="button" @click="retryUnavailableSlots">Ponów niedostępne</button>
          </div>

          <div class="mt-4 flex flex-wrap gap-2 text-xs font-bold text-ink-soft" aria-label="Legenda siatki i usług">
            <span class="inline-flex items-center gap-2 rounded-full bg-white px-3 py-1.5 shadow-sm">
              mniej osób <span class="availability-gradient h-3 w-20 rounded-full" aria-hidden="true" /> więcej osób
            </span>
            <span class="rounded-full bg-lavender/60 px-3 py-1.5">„może” = połowa nasycenia</span>
            <span class="inline-flex items-center gap-1.5 rounded-full bg-white px-3 py-1.5"><span class="size-2 rounded-full bg-emerald-500" /> serwis online</span>
            <span class="inline-flex items-center gap-1.5 rounded-full bg-white px-3 py-1.5"><span class="size-2 rounded-full bg-red-500" /> błąd odczytu</span>
            <span class="inline-flex items-center gap-1.5 rounded-full bg-white px-3 py-1.5"><span class="size-2 animate-pulse rounded-full bg-amber-400" /> sprawdzanie</span>
          </div>
          <p class="mt-2 text-xs text-ink-soft">Każda godzina jest osobnym serwisem. Kropka pokazuje wynik ostatniej próby odczytu, nie monitoring na żywo.</p>

          <fieldset v-if="selectableParticipants.length" class="mt-5 rounded-2xl border border-berry/15 bg-white/65 p-3">
            <legend class="px-2 text-sm font-bold text-berry-dark">Czyje plany porównujemy?</legend>
            <div class="flex flex-wrap gap-2">
              <button
                type="button"
                class="min-h-11 rounded-full border px-4 py-2 text-sm font-bold transition"
                :class="participantFilterIds.length === 0 ? 'border-berry bg-berry text-white' : 'border-berry/20 bg-white text-ink'"
                :aria-pressed="participantFilterIds.length === 0"
                @click="selectAllParticipants"
              >
                Wszyscy
              </button>
              <button
                v-for="participant in selectableParticipants"
                :key="participant.userId"
                type="button"
                class="min-h-11 rounded-full border px-4 py-2 text-sm font-bold transition"
                :class="participantIsSelected(participant.userId) ? 'border-berry-dark bg-blush text-berry-dark' : 'border-berry/15 bg-white text-ink-soft'"
                :aria-pressed="participantIsSelected(participant.userId)"
                @click="toggleParticipant(participant.userId)"
              >
                {{ participant.nick }}<span v-if="participant.userId === session?.userId" class="ml-1 text-xs">(Ty)</span>
              </button>
            </div>
            <p class="mt-2 text-xs text-ink-soft">Kliknij jedną osobę, aby zobaczyć tylko jej plan. Kolejne kliknięcia dodają osoby do porównania.</p>
          </fieldset>

          <ScheduleAvailabilityGrid
            class="mt-4"
            :slots="event.slots"
            :reads="schedule.reads"
            :summaries="filteredSummaries"
            :participants="selectedParticipants"
            :participant-count="selectedParticipants.length"
            :busy="schedule.isLoading"
            :editable="canEditSelectedView"
            :current-user-id="session?.userId"
            :comparison-user-id="selectedParticipants.length === 1 ? selectedParticipants[0]?.userId : undefined"
            :write-states="schedule.writeStates"
            @retry="slot => schedule.retrySlots([slot])"
            @edit="editSlot"
            @retry-write="slot => session && schedule.retryWrite(slot, session)"
          />

        </section>

        <dialog ref="joinDialog" class="paper-card fixed inset-0 m-auto w-[min(30rem,calc(100%-2rem))] p-0 backdrop:bg-ink/45" aria-labelledby="join-title" @close="handleJoinDialogClose">
          <form class="p-6 sm:p-8" novalidate @submit.prevent="join">
            <div class="flex items-start justify-between gap-4">
              <div>
                <p class="text-sm font-bold tracking-wide text-berry uppercase">Dodaj dostępność</p>
                <h2 id="join-title" class="mt-1 font-display text-3xl font-bold text-berry-dark">Jak mamy Cię podpisać?</h2>
              </div>
              <button type="button" class="grid size-11 shrink-0 place-items-center rounded-full border border-berry/20 bg-white text-xl" aria-label="Zamknij" @click="joinDialog?.close()">×</button>
            </div>

            <p v-if="reauthNotice" class="mt-4 rounded-xl bg-amber-50 p-3 text-sm text-ink" role="alert">
              Sesja wygasła. Dołącz ponownie, a ponowimy niezapisane wybory.
            </p>
            <label for="nick" class="mt-6 mb-2 block font-bold">Pseudonim</label>
            <input id="nick" v-model="nick" class="field" type="text" autocomplete="username" :aria-invalid="Boolean(nickError)" :aria-describedby="nickError ? 'nick-error' : undefined">
            <p v-if="nickError" id="nick-error" class="error-text mt-2 text-sm">{{ nickError }}</p>

            <label for="password" class="mt-4 mb-2 block font-bold">Hasło <span class="font-normal text-ink-soft">(opcjonalne)</span></label>
            <input id="password" v-model="password" class="field" type="password" autocomplete="current-password" :aria-invalid="Boolean(passwordError)" :aria-describedby="passwordError ? 'password-error password-hint' : 'password-hint'">
            <p id="password-hint" class="mt-2 text-xs leading-5 text-ink-soft">Hasło chroni pseudonim tylko wtedy, gdy ustawisz je przy pierwszym dołączeniu.</p>
            <p v-if="passwordError" id="password-error" class="error-text mt-2 text-sm">{{ passwordError }}</p>

            <p v-if="joinError" role="alert" class="error-text mt-4 rounded-xl bg-red-50 p-3 text-sm">{{ joinError }}</p>
            <button class="button-primary mt-5 w-full" type="submit" :disabled="joinPending">
              {{ joinPending ? 'Dołączam…' : 'Przejdź do zaznaczania' }}
            </button>
          </form>
        </dialog>

        <GarnekDialog
          :visible="showGarnek"
          :participant-names="selectableParticipants.map(participant => participant.nick)"
          @dismiss="dismissGarnek"
          @return-to-grid="focusGridSlot()"
        />
      </template>
    </main>
  </div>
</template>
