<script setup lang="ts">
import { garnekDialogue } from './dialogue'

const props = defineProps<{
  visible: boolean
  participantNames: string[]
}>()

const emit = defineEmits<{
  dismiss: []
  returnToGrid: []
}>()

const dialog = ref<HTMLDialogElement | null>(null)

onMounted(() => {
  if (props.visible && !dialog.value?.open) dialog.value?.showModal()
})

watch(() => props.visible, async (visible) => {
  await nextTick()
  if (visible && !dialog.value?.open) dialog.value?.showModal()
  else if (!visible && dialog.value?.open) dialog.value.close()
})

function returnToGrid(): void {
  dialog.value?.close()
  emit('returnToGrid')
}

</script>

<template>
  <dialog
    ref="dialog"
    class="fixed inset-0 m-auto w-[min(58rem,calc(100%-1.5rem))] overflow-visible rounded-[2rem] border-4 border-berry bg-paper p-0 shadow-[0_2rem_6rem_rgb(58_15_40/40%)] backdrop:bg-berry-dark/55"
    aria-labelledby="garnek-title"
    @close="emit('dismiss')"
  >
    <div class="relative grid min-h-[24rem] items-end overflow-hidden rounded-[1.7rem] bg-[linear-gradient(145deg,#fff9f2_0%,#fce7f1_55%,#e9ddff_100%)] p-4 sm:grid-cols-[17rem_1fr] sm:p-6">
      <button type="button" class="absolute top-3 right-3 z-20 grid size-11 place-items-center rounded-full border border-berry/25 bg-white text-xl text-berry-dark" aria-label="Zamknij wypowiedź Garnka" @click="dialog?.close()">×</button>
      <img src="/assets/garnek/garnek-sad.png" alt="" class="mx-auto max-h-72 w-auto object-contain drop-shadow-[0_1rem_1.5rem_rgb(92_37_67/20%)] sm:max-h-[22rem]" aria-hidden="true">
      <div class="relative z-10 -mt-8 rounded-2xl border-4 border-white bg-white/95 p-5 shadow-[0_0_0_2px_#b83268,0_1rem_2rem_rgb(92_37_67/18%)] sm:mt-0 sm:p-7">
        <span class="absolute -top-5 left-5 rounded-full border-2 border-white bg-berry px-5 py-1.5 font-display text-lg font-bold text-white shadow-[0_0_0_2px_#741842]">Garnek</span>
        <h2 id="garnek-title" class="sr-only">{{ garnekDialogue.noOverlap.title }}</h2>
        <p class="mt-2 font-display text-2xl leading-relaxed font-bold text-berry-dark">{{ garnekDialogue.noOverlap.message }}</p>
        <p class="mt-3 text-sm leading-6 text-ink-soft">
          Porównuję: {{ participantNames.join(', ') }}. {{ garnekDialogue.noOverlap.followUp }}
        </p>
        <div class="mt-6 flex flex-wrap gap-3">
          <button class="button-primary" type="button" @click="returnToGrid">{{ garnekDialogue.noOverlap.returnAction }}</button>
        </div>
      </div>
    </div>
  </dialog>
</template>
