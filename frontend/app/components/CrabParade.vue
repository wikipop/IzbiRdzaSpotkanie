<template>
  <!-- Ferris and friends: purely decorative, never focusable and never in the way of a click. -->
  <div class="crab-parade" aria-hidden="true">
    <span class="crab crab-walker">🦀</span>
    <span class="crab crab-flyer">🦀</span>
    <span class="crab crab-hopper">🦀</span>
  </div>
</template>

<style scoped>
.crab-parade {
  position: fixed;
  inset: 0;
  z-index: 30;
  overflow: hidden;
  pointer-events: none;
}

.crab {
  position: absolute;
  line-height: 1;
  user-select: none;
}

/* Crabs walk sideways, so this one scuttles along the bottom edge and back. */
.crab-walker {
  bottom: 0.4rem;
  left: 0;
  font-size: 1.75rem;
  animation:
    crab-walk 26s linear infinite alternate,
    crab-scuttle 0.32s ease-in-out infinite alternate;
}

.crab-flyer {
  top: 12%;
  left: 0;
  font-size: 2.25rem;
  animation: crab-fly 19s ease-in-out 4s infinite;
}

.crab-hopper {
  bottom: 0.6rem;
  left: 1rem;
  font-size: 1.5rem;
  animation: crab-hop 3.6s cubic-bezier(0.3, 0, 0.4, 1) 1s infinite;
}

@keyframes crab-walk {
  from { translate: -3rem 0; }
  to { translate: calc(100vw + 1rem) 0; }
}

@keyframes crab-scuttle {
  from { rotate: -7deg; }
  to { rotate: 7deg; }
}

@keyframes crab-fly {
  0% { transform: translate(-4rem, 0) rotate(-12deg); }
  25% { transform: translate(25vw, -2.5rem) rotate(10deg); }
  50% { transform: translate(52vw, 1.5rem) rotate(-8deg); }
  75% { transform: translate(78vw, -2rem) rotate(14deg); }
  /* Out of view for the rest of the cycle so the sky is not always busy. */
  85%, 100% { transform: translate(calc(100vw + 4rem), 0) rotate(0deg); }
}

@keyframes crab-hop {
  0%, 55%, 100% { transform: translateY(0) scale(1, 1); }
  60% { transform: translateY(0) scale(1.15, 0.85); }
  72% { transform: translateY(-2.4rem) scale(0.95, 1.08) rotate(-10deg); }
  84% { transform: translateY(0) scale(1.1, 0.9); }
  90% { transform: translateY(-0.6rem) scale(1, 1); }
}

@media (prefers-reduced-motion: reduce) {
  .crab-parade {
    display: none;
  }
}
</style>
