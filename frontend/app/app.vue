<template>
  <NuxtRouteAnnouncer />
  <a class="skip-link" href="#main-content">Przejdź do treści</a>
  <FerrisBackground />
  <!-- Lifts the page above the drifting crabs without touching each page's layout. -->
  <div class="relative z-[1]">
    <NuxtPage />
  </div>
  <DevtoolsHint />
</template>
