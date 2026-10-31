<script setup lang="ts">
const props = defineProps<{ error: { statusCode?: number; statusMessage?: string; message?: string } }>()

const title = computed(() => props.error.statusCode === 404
  ? 'Nie znaleźliśmy tej strony'
  : 'Coś przerwało tę historię')

async function returnHome(): Promise<void> {
  await clearError({ redirect: '/' })
}
</script>

<template>
  <main id="main-content" class="grid min-h-screen place-items-center px-4 py-10">
    <section class="paper-card w-full max-w-xl p-7 text-center sm:p-10" aria-labelledby="error-title">
      <p class="text-5xl" aria-hidden="true">✉️</p>
      <h1 id="error-title" class="mt-4 font-display text-4xl font-bold text-rust-dark">{{ title }}</h1>
      <p class="mt-3 text-ink-soft">
        {{ error.statusCode === 404 ? 'Sprawdź adres albo wróć do tworzenia spotkania.' : 'Spróbuj wrócić na początek i otworzyć plan ponownie.' }}
      </p>
      <button type="button" class="button-primary mt-7" @click="returnHome">Wróć na początek</button>
    </section>
  </main>
</template>
