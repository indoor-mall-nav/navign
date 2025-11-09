<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  fromFloor: string
  toFloor: string
}>()

// Determine direction based on floor names/numbers
const direction = computed(() => {
  const fromNum = parseInt(props.fromFloor.replace(/\D/g, '')) || 0
  const toNum = parseInt(props.toFloor.replace(/\D/g, '')) || 0
  return toNum > fromNum ? 'up' : 'down'
})
</script>

<template>
  <div class="relative w-full h-48 bg-gradient-to-br from-green-100 to-green-200 dark:from-green-950 dark:to-green-900 rounded-lg overflow-hidden">
    <!-- Escalator structure -->
    <svg class="w-full h-full" viewBox="0 0 200 200" preserveAspectRatio="xMidYMid meet">
      <!-- Escalator frame -->
      <g v-if="direction === 'up'">
        <!-- Steps moving up (left low, right high) -->
        <line
          x1="20" y1="160"
          x2="180" y2="40"
          class="stroke-slate-400 dark:stroke-slate-600"
          stroke-width="8"
        />

        <!-- Animated steps -->
        <g v-for="i in 8" :key="i" class="animate-step-up" :style="{ animationDelay: `${i * 0.15}s` }">
          <rect
            :x="20 + i * 20"
            :y="160 - i * 15"
            width="16"
            height="3"
            class="fill-green-600 dark:fill-green-700"
            rx="1"
          />
        </g>

        <!-- Handrail -->
        <line
          x1="15" y1="155"
          x2="175" y2="35"
          class="stroke-green-700 dark:stroke-green-500"
          stroke-width="3"
          stroke-linecap="round"
        />

        <!-- Floor labels -->
        <text x="10" y="175" class="text-xs fill-slate-600 dark:fill-slate-400" font-size="12">
          {{ fromFloor }}
        </text>
        <text x="185" y="35" text-anchor="end" class="text-xs fill-green-600 dark:fill-green-400 font-bold" font-size="12">
          {{ toFloor }}
        </text>
      </g>

      <g v-else>
        <!-- Steps moving down (left high, right low) -->
        <line
          x1="20" y1="40"
          x2="180" y2="160"
          class="stroke-slate-400 dark:stroke-slate-600"
          stroke-width="8"
        />

        <!-- Animated steps -->
        <g v-for="i in 8" :key="i" class="animate-step-down" :style="{ animationDelay: `${i * 0.15}s` }">
          <rect
            :x="20 + i * 20"
            :y="40 + i * 15"
            width="16"
            height="3"
            class="fill-green-600 dark:fill-green-700"
            rx="1"
          />
        </g>

        <!-- Handrail -->
        <line
          x1="15" y1="35"
          x2="175" y2="155"
          class="stroke-green-700 dark:stroke-green-500"
          stroke-width="3"
          stroke-linecap="round"
        />

        <!-- Floor labels -->
        <text x="10" y="35" class="text-xs fill-slate-600 dark:fill-slate-400" font-size="12">
          {{ fromFloor }}
        </text>
        <text x="185" y="175" text-anchor="end" class="text-xs fill-green-600 dark:fill-green-400 font-bold" font-size="12">
          {{ toFloor }}
        </text>
      </g>

      <!-- Direction arrow -->
      <g class="animate-pulse">
        <path
          v-if="direction === 'up'"
          d="M 100 100 L 110 90 M 100 100 L 90 90"
          class="stroke-green-600 dark:stroke-green-400"
          stroke-width="4"
          fill="none"
          stroke-linecap="round"
        />
        <path
          v-else
          d="M 100 100 L 110 110 M 100 100 L 90 110"
          class="stroke-green-600 dark:stroke-green-400"
          stroke-width="4"
          fill="none"
          stroke-linecap="round"
        />
      </g>
    </svg>

    <!-- Direction indicator -->
    <div class="absolute top-2 left-2 px-2 py-1 bg-white/80 dark:bg-black/60 rounded text-xs font-medium">
      {{ direction === 'up' ? '↗' : '↘' }} {{ direction.toUpperCase() }}
    </div>

    <!-- Connected areas info -->
    <div class="absolute bottom-2 left-1/2 -translate-x-1/2 px-3 py-1 bg-white/90 dark:bg-black/70 rounded text-xs font-medium whitespace-nowrap">
      {{ fromFloor }} → {{ toFloor }}
    </div>
  </div>
</template>

<style scoped>
@keyframes step-up {
  0% {
    opacity: 0;
    transform: translate(-10px, 10px);
  }
  50% {
    opacity: 1;
  }
  100% {
    opacity: 0;
    transform: translate(10px, -10px);
  }
}

@keyframes step-down {
  0% {
    opacity: 0;
    transform: translate(-10px, -10px);
  }
  50% {
    opacity: 1;
  }
  100% {
    opacity: 0;
    transform: translate(10px, 10px);
  }
}

.animate-step-up {
  animation: step-up 2s ease-in-out infinite;
}

.animate-step-down {
  animation: step-down 2s ease-in-out infinite;
}
</style>
