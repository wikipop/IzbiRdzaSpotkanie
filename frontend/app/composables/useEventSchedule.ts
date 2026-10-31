import { discoverParticipants, summarizeAvailability } from '~/domain/availability'
import type { ApiFailure } from '~/services/api/result'
import type { AvailabilityResponse, AvailabilityState, EventResponse, SlotId } from '~/types/api/contracts'
import { runWithConcurrency } from '~/utils/concurrency'

export type SlotReadState =
  | { status: 'queued' }
  | { status: 'loading' }
  | { status: 'success'; data: AvailabilityResponse }
  | { status: 'failed'; error: ApiFailure }

export type AvailabilityIntent = AvailabilityState | null

export type CellWriteState =
  | { status: 'saving'; desired: AvailabilityIntent; ownerUserId: string }
  | { status: 'failed'; desired: AvailabilityIntent; ownerUserId: string; message: string }

export interface WriteIdentity {
  userId: string
  token: string
}

const defaultConcurrency = 8

export function useEventSchedule(
  eventId: Ref<string>,
  event: Ref<EventResponse | null>,
) {
  const { slots: slotClient } = useApi()
  const reads = ref<Partial<Record<SlotId, SlotReadState>>>({})
  const writeStates = ref<Partial<Record<SlotId, CellWriteState>>>({})
  const activeControllers = new Set<AbortController>()
  const processingWrites = new Map<SlotId, number>()
  const pendingReadReconciliations = new Map<SlotId, {
    generation: number
    identity: WriteIdentity
    desired: AvailabilityIntent
    entry?: AvailabilityResponse['entries'][number]
  }>()
  const reauthRequired = ref(false)
  // The newest token handed in by the page; writes started with an older one retry with it.
  let latestIdentity: WriteIdentity | null = null
  let generation = 0
  let writeGeneration = 0
  let mounted = false

  const states = computed(() => Object.values(reads.value).filter(Boolean) as SlotReadState[])
  const successfulCount = computed(() => states.value.filter(state => state.status === 'success').length)
  const failedSlots = computed(() => (event.value?.slots ?? []).filter(slot => (
    reads.value[slot]?.status === 'failed'
  )))
  const isLoading = computed(() => states.value.some(state => (
    state.status === 'queued' || state.status === 'loading'
  )))
  const successfulResponses = computed(() => states.value.flatMap(state => (
    state.status === 'success' ? [state.data] : []
  )))
  const summaries = computed(() => summarizeAvailability(successfulResponses.value))
  const participants = computed(() => discoverParticipants(successfulResponses.value))

  function cancelActiveRequests(): void {
    generation++
    for (const controller of activeControllers) controller.abort()
    activeControllers.clear()
  }

  function applySuccessfulWrite(
    slot: SlotId,
    identity: WriteIdentity,
    desired: AvailabilityIntent,
    entry?: AvailabilityResponse['entries'][number],
  ): boolean {
    const read = reads.value[slot]
    if (read?.status !== 'success') return false

    const otherEntries = read.data.entries.filter(item => item.user_id !== identity.userId)
    reads.value[slot] = {
      status: 'success',
      data: {
        ...read.data,
        entries: desired === null || !entry ? otherEntries : [...otherEntries, entry],
      },
    }
    return true
  }

  async function processWrite(slot: SlotId, initialIdentity: WriteIdentity): Promise<void> {
    latestIdentity = initialIdentity
    if (processingWrites.has(slot)) return
    let identity = initialIdentity
    const processGeneration = writeGeneration
    processingWrites.set(slot, processGeneration)

    try {
      while (writeStates.value[slot]) {
        const pendingWrite = writeStates.value[slot]
        if (!pendingWrite || pendingWrite.ownerUserId !== identity.userId) return
        const desired = pendingWrite.desired
        const requestedEventId = eventId.value
        writeStates.value[slot] = { status: 'saving', desired, ownerUserId: identity.userId }

        const result = desired === null
          ? await slotClient.clearMyAvailability(slot, requestedEventId, identity.token)
          : await slotClient.setMyAvailability(slot, requestedEventId, identity.token, desired)

        if (requestedEventId !== eventId.value || processGeneration !== writeGeneration) return

        if (!result.ok) {
          if (result.error.kind === 'api'
            && (result.error.code === 'invalid_token' || result.error.code === 'missing_token')) {
            if (latestIdentity?.userId === identity.userId && latestIdentity.token !== identity.token) {
              // The user rejoined while this request was in flight; retry with the fresh token.
              identity = latestIdentity
              continue
            }
            reauthRequired.value = true
          }
          const latestWrite = writeStates.value[slot]
          writeStates.value[slot] = {
            status: 'failed',
            desired: latestWrite ? latestWrite.desired : desired,
            ownerUserId: identity.userId,
            message: 'Nie udało się zapisać. Kliknij pole, aby ponowić.',
          }
          return
        }

        const responseEntry = desired === null ? undefined : result.data
        if (!applySuccessfulWrite(slot, identity, desired, responseEntry)
          && (reads.value[slot]?.status === 'queued' || reads.value[slot]?.status === 'loading')) {
          pendingReadReconciliations.set(slot, {
            generation,
            identity,
            desired,
            ...(responseEntry ? { entry: responseEntry } : {}),
          })
        }
        if (writeStates.value[slot]?.desired === desired) {
          const { [slot]: _completed, ...remainingWrites } = writeStates.value
          writeStates.value = remainingWrites
          return
        }
      }
    } finally {
      if (processingWrites.get(slot) === processGeneration) processingWrites.delete(slot)
    }
  }

  function setDesiredAvailability(slot: SlotId, identity: WriteIdentity, desired: AvailabilityIntent): void {
    const read = reads.value[slot]
    if (read?.status !== 'success') return
    writeStates.value[slot] = { status: 'saving', desired, ownerUserId: identity.userId }
    void processWrite(slot, identity)
  }

  function retryWrite(slot: SlotId, identity: WriteIdentity): void {
    if (writeStates.value[slot]?.status !== 'failed'
      || writeStates.value[slot]?.ownerUserId !== identity.userId) return
    void processWrite(slot, identity)
  }

  function retryFailedWrites(identity: WriteIdentity): void {
    latestIdentity = identity
    reauthRequired.value = false
    for (const slot of Object.keys(writeStates.value) as SlotId[]) {
      const write = writeStates.value[slot]
      if (write?.ownerUserId !== identity.userId) {
        const { [slot]: _discarded, ...remainingWrites } = writeStates.value
        writeStates.value = remainingWrites
      } else if (write.status === 'failed') void processWrite(slot, identity)
    }
  }

  function discardWrites(): void {
    writeGeneration++
    processingWrites.clear()
    pendingReadReconciliations.clear()
    writeStates.value = {}
    reauthRequired.value = false
    latestIdentity = null
  }

  async function loadTargets(targets: SlotId[], replaceAll: boolean): Promise<void> {
    cancelActiveRequests()
    const requestedEventId = eventId.value
    const requestGeneration = generation

    if (replaceAll) reads.value = {}
    for (const slot of targets) reads.value[slot] = { status: 'queued' }

    await runWithConcurrency(targets, defaultConcurrency, async (slot) => {
      if (requestGeneration !== generation) return

      reads.value[slot] = { status: 'loading' }
      const controller = new AbortController()
      activeControllers.add(controller)
      const result = await slotClient.getAvailability(slot, requestedEventId, controller.signal)
      activeControllers.delete(controller)
      if (requestGeneration !== generation || requestedEventId !== eventId.value) return

      if (!result.ok) {
        pendingReadReconciliations.delete(slot)
        reads.value[slot] = { status: 'failed', error: result.error }
        return
      }

      const reconciliation = pendingReadReconciliations.get(slot)
      const data = reconciliation?.generation === requestGeneration
        ? {
            ...result.data,
            entries: [
              ...result.data.entries.filter(item => item.user_id !== reconciliation.identity.userId),
              ...(reconciliation.desired !== null && reconciliation.entry ? [reconciliation.entry] : []),
            ],
          }
        : result.data
      if (reconciliation?.generation === requestGeneration) pendingReadReconciliations.delete(slot)
      reads.value[slot] = { status: 'success', data }
    })
  }

  async function refreshAll(): Promise<void> {
    const eventSlots = event.value?.slots ?? []
    if (eventSlots.length === 0) {
      cancelActiveRequests()
      reads.value = {}
      return
    }
    await loadTargets(eventSlots, true)
  }

  async function retrySlots(targets: SlotId[] = failedSlots.value): Promise<void> {
    if (targets.length === 0 || isLoading.value) return
    await loadTargets(targets, false)
  }

  onMounted(() => {
    mounted = true
    void refreshAll()
  })
  watch(
    () => event.value,
    (next, previous) => {
      if (!import.meta.client || !mounted || next === previous) return
      writeGeneration++
      processingWrites.clear()
      pendingReadReconciliations.clear()
      writeStates.value = {}
      reauthRequired.value = false
      void refreshAll()
    },
  )
  onBeforeUnmount(() => {
    mounted = false
    cancelActiveRequests()
  })

  return {
    reads: readonly(reads),
    writeStates: readonly(writeStates),
    reauthRequired: readonly(reauthRequired),
    successfulCount,
    failedSlots,
    isLoading,
    summaries,
    participants,
    refreshAll,
    retrySlots,
    setDesiredAvailability,
    retryWrite,
    retryFailedWrites,
    discardWrites,
  }
}
