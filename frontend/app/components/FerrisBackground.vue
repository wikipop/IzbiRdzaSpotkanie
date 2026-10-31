<script setup lang="ts">
// Large, faint Ferris crabs drifting behind the page. They bounce off the viewport edges and
// off each other as circles of equal mass. Purely decorative: no pointer events, hidden from
// assistive technology, frozen in place for people who prefer reduced motion.

interface Crab {
  x: number
  y: number
  vx: number
  vy: number
  width: number
  radius: number
  spin: number
  angle: number
}

const aspect = 800 / 1200
const crabs = ref<Crab[]>([])
const elements = ref<HTMLElement[]>([])
let frame = 0
let last = 0

function random(min: number, max: number): number {
  return min + Math.random() * (max - min)
}

function spawn(): Crab[] {
  const count = window.innerWidth < 640 ? 3 : 6
  // Size follows the viewport so every crab fits even in a small window.
  const base = Math.min(window.innerWidth, window.innerHeight)
  const result: Crab[] = []
  for (let attempt = 0; result.length < count && attempt < 400; attempt++) {
    const width = Math.min(340, base * random(0.26, 0.4))
    const radius = width * 0.36
    const crab: Crab = {
      x: random(radius, window.innerWidth - radius),
      y: random(radius, window.innerHeight - radius),
      vx: random(35, 80) * (Math.random() < 0.5 ? -1 : 1),
      vy: random(35, 80) * (Math.random() < 0.5 ? -1 : 1),
      width,
      radius,
      spin: random(-8, 8),
      angle: random(-20, 20),
    }
    // Start without overlaps so nobody is stuck inside a neighbour.
    if (result.every(other => Math.hypot(other.x - crab.x, other.y - crab.y) > other.radius + crab.radius)) {
      result.push(crab)
    }
  }
  return result
}

function render(): void {
  crabs.value.forEach((crab, index) => {
    const element = elements.value[index]
    if (!element) return
    const height = crab.width * aspect
    element.style.transform = `translate(${crab.x - crab.width / 2}px, ${crab.y - height / 2}px) rotate(${crab.angle}deg)`
  })
}

function step(seconds: number): void {
  const width = window.innerWidth
  const height = window.innerHeight
  const list = crabs.value

  for (const crab of list) {
    crab.x += crab.vx * seconds
    crab.y += crab.vy * seconds
    crab.angle += crab.spin * seconds
    if (crab.x < crab.radius) { crab.x = crab.radius; crab.vx = Math.abs(crab.vx); crab.spin = -crab.spin }
    if (crab.x > width - crab.radius) { crab.x = width - crab.radius; crab.vx = -Math.abs(crab.vx); crab.spin = -crab.spin }
    if (crab.y < crab.radius) { crab.y = crab.radius; crab.vy = Math.abs(crab.vy) }
    if (crab.y > height - crab.radius) { crab.y = height - crab.radius; crab.vy = -Math.abs(crab.vy) }
  }

  for (let i = 0; i < list.length; i++) {
    for (let j = i + 1; j < list.length; j++) {
      const a = list[i]!
      const b = list[j]!
      const dx = b.x - a.x
      const dy = b.y - a.y
      const distance = Math.hypot(dx, dy) || 1
      const overlap = a.radius + b.radius - distance
      if (overlap <= 0) continue

      const nx = dx / distance
      const ny = dy / distance
      // Push apart, then swap the velocity components along the contact normal.
      a.x -= nx * overlap / 2
      a.y -= ny * overlap / 2
      b.x += nx * overlap / 2
      b.y += ny * overlap / 2
      const approaching = (a.vx - b.vx) * nx + (a.vy - b.vy) * ny
      if (approaching > 0) {
        a.vx -= approaching * nx
        a.vy -= approaching * ny
        b.vx += approaching * nx
        b.vy += approaching * ny
        a.spin = random(-12, 12)
        b.spin = random(-12, 12)
      }
    }
  }
}

function tick(now: number): void {
  // Clamp the step so a backgrounded tab does not teleport crabs through walls.
  const seconds = Math.min((now - last) / 1000, 0.05)
  last = now
  step(seconds)
  render()
  frame = requestAnimationFrame(tick)
}

onMounted(async () => {
  crabs.value = spawn()
  await nextTick()
  render()
  if (matchMedia('(prefers-reduced-motion: reduce)').matches) return
  last = performance.now()
  frame = requestAnimationFrame(tick)
})

onBeforeUnmount(() => cancelAnimationFrame(frame))
</script>

<template>
  <div class="ferris-background" aria-hidden="true">
    <img
      v-for="(crab, index) in crabs"
      :key="index"
      ref="elements"
      src="/assets/ferris/ferris.svg"
      alt=""
      class="ferris"
      :style="{ width: `${crab.width}px` }"
    >
  </div>
</template>

<style scoped>
.ferris-background {
  position: fixed;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  pointer-events: none;
}

.ferris {
  position: absolute;
  top: 0;
  left: 0;
  opacity: 0.15;
  user-select: none;
  will-change: transform;
  animation: ferris-in 900ms ease-out both;
}

@keyframes ferris-in {
  from {
    opacity: 0;
  }
}
</style>
