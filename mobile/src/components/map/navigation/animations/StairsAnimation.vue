<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    fromFloor: string
    toFloor: string
    /** Number of floors to pass through */
    floorCount?: number
  }>(),
  {
    floorCount: 3,
  },
)

// Determine direction based on floor names/numbers
const direction = computed(() => {
  const fromNum = parseInt(props.fromFloor.replace(/\D/g, '')) || 0
  const toNum = parseInt(props.toFloor.replace(/\D/g, '')) || 0
  return toNum > fromNum ? 'up' : 'down'
})

// Generate floor labels between from and to
const floors = computed(() => {
  const fromNum = parseInt(props.fromFloor.replace(/\D/g, '')) || 0
  const toNum = parseInt(props.toFloor.replace(/\D/g, '')) || 0
  const count = Math.abs(toNum - fromNum) || props.floorCount

  const floorList: Array<{ label: string; side: 'left' | 'right' }> = []

  for (let i = 0; i <= count; i++) {
    const floorNum = direction.value === 'up' ? fromNum + i : fromNum - i
    floorList.push({
      label: `Floor ${floorNum}`,
      side: i % 2 === 0 ? 'left' : 'right',
    })
  }

  return floorList
})
</script>

<template>
  <div class="relative w-full h-48 bg-gradient-to-b from-slate-100 to-slate-200 dark:from-slate-800 dark:to-slate-900 rounded-lg overflow-hidden">
    <!-- Stairs visual -->
    <div class="absolute inset-0 flex items-center justify-center">
      <svg class="w-full h-full" viewBox="0 0 200 200" preserveAspectRatio="xMidYMid meet">
        <!-- Animated stairs steps -->
        <g v-for="(floor, index) in floors" :key="index">
          <g
            class="animate-fade-slide"
            :style="{
              animationDelay: `${index * 0.3}s`,
              animationDuration: '1.5s',
            }"
          >
            <!-- Step platform -->
            <rect
              :x="floor.side === 'left' ? 20 : 100"
              :y="30 + index * 25"
              width="80"
              height="4"
              :class="index === floors.length - 1 ? 'fill-green-500' : 'fill-slate-400 dark:fill-slate-600'"
              rx="2"
            />
            <!-- Step riser -->
            <rect
              :x="floor.side === 'left' ? 20 : 100"
              :y="34 + index * 25"
              width="4"
              height="25"
              class="fill-slate-500 dark:fill-slate-700"
            />
            <!-- Floor label -->
            <text
              :x="floor.side === 'left' ? 10 : 190"
              :y="40 + index * 25"
              :text-anchor="floor.side === 'left' ? 'end' : 'start'"
              class="text-xs fill-slate-600 dark:fill-slate-400"
              font-size="10"
            >
              {{ floor.label }}
            </text>
          </g>
        </g>

        <!-- Direction arrow -->
        <g class="animate-bounce-slow">
          <path
            v-if="direction === 'up'"
            d="M 100 170 L 100 180 M 100 170 L 95 175 M 100 170 L 105 175"
            class="stroke-blue-500 dark:stroke-blue-400"
            stroke-width="3"
            fill="none"
            stroke-linecap="round"
          />
          <path
            v-else
            d="M 100 180 L 100 170 M 100 180 L 95 175 M 100 180 L 105 175"
            class="stroke-blue-500 dark:stroke-blue-400"
            stroke-width="3"
            fill="none"
            stroke-linecap="round"
          />
        </g>
      </svg>
    </div>

    <!-- Direction indicator -->
    <div class="absolute top-2 left-2 px-2 py-1 bg-white/80 dark:bg-black/60 rounded text-xs font-medium">
      {{ direction === 'up' ? '↑' : '↓' }} {{ direction.toUpperCase() }}
    </div>
  </div>
</template>

<style scoped>
@keyframes fade-slide {
  0% {
    opacity: 0;
    transform: translateY(-10px);
  }
  50% {
    opacity: 1;
  }
  100% {
    opacity: 0.3;
    transform: translateY(10px);
  }
}

@keyframes bounce-slow {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

.animate-fade-slide {
  animation: fade-slide 1.5s ease-in-out infinite;
}

.animate-bounce-slow {
  animation: bounce-slow 2s ease-in-out infinite;
}
</style>
