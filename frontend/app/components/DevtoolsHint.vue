<script setup lang="ts">
const storageKey = 'izbi-rdza:devtools-hint-dismissed'
const visible = ref(false)

onMounted(() => {
  try {
    visible.value = localStorage.getItem(storageKey) !== '1'
  } catch {
    // Storage can be blocked; show the hint once for this page view.
    visible.value = true
  }
})

function dismiss(): void {
  visible.value = false
  try {
    localStorage.setItem(storageKey, '1')
  } catch {
    // The hint simply reappears on the next visit.
  }
}
</script>

<template>
  <!-- Deliberately a landmark, not a modal dialog: it must never block the page underneath. -->
  <aside
    v-if="visible"
    class="devtools-hint fixed right-3 bottom-3 left-3 z-40 rounded-2xl border border-rust-dark/20 bg-white p-5 shadow-[0_1.5rem_3rem_rgb(29_36_34/18%)] sm:left-auto sm:w-[25rem]"
    aria-labelledby="devtools-hint-title"
  >
    <h2 id="devtools-hint-title" class="font-display text-xl font-bold text-rust-dark">Zajrzyj pod maskę</h2>
    <p class="mt-2 text-sm leading-6 text-ink">
      Każda godzina tygodnia to osobny serwis w Ruście. Otwórz narzędzia deweloperskie
      (<kbd>F12</kbd>, na Macu <kbd>⌥⌘I</kbd>) i przejdź do karty <strong>Network</strong>.
    </p>
    <p class="mt-2 text-sm leading-6 text-ink-soft">
      W spotkaniu zobaczysz osobne zapytanie do każdej godziny, na przykład do <code>pn-09-meet.izbi.beer</code>.
    </p>
    <button type="button" class="button-primary mt-4" @click="dismiss">Rozumiem</button>
  </aside>
</template>

<style scoped>
.devtools-hint {
  animation: hint-in 320ms ease-out 600ms both;
}

kbd,
code {
  border-radius: 0.35rem;
  background: var(--color-steel);
  padding: 0.05rem 0.35rem;
  font-size: 0.85em;
}

@keyframes hint-in {
  from {
    opacity: 0;
    transform: translateY(0.75rem);
  }
}
</style>
