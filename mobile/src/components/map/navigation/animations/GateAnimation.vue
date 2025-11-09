<script setup lang="ts">
import { ref, onMounted } from 'vue'

defineProps<{
  fromArea: string
  toArea: string
  gateType: 'gate' | 'turnstile'
}>()

// Animate gate opening
const gateOpen = ref(false)

onMounted(() => {
  const interval = setInterval(() => {
    gateOpen.value = !gateOpen.value
  }, 2000)

  return () => clearInterval(interval)
})
</script>

<template>
  <div class="relative w-full h-48 bg-gradient-to-b from-red-100 to-red-200 dark:from-red-950 dark:to-red-900 rounded-lg overflow-hidden">
    <svg class="w-full h-full" viewBox="0 0 200 200" preserveAspectRatio="xMidYMid meet">
      <g v-if="gateType === 'gate'">
        <!-- Gate frame -->
        <rect
          x="30" y="50"
          width="140" height="120"
          class="fill-none stroke-slate-600 dark:stroke-slate-400"
          stroke-width="4"
          rx="4"
        />

        <!-- Gate doors (animated) -->
        <rect
          x="30" y="50"
          :width="gateOpen ? 10 : 70"
          height="120"
          class="fill-red-600 dark:fill-red-700 transition-all duration-1000 ease-in-out"
        />
        <rect
          :x="gateOpen ? 160 : 100"
          y="50"
          :width="gateOpen ? 10 : 70"
          height="120"
          class="fill-red-600 dark:fill-red-700 transition-all duration-1000 ease-in-out"
        />

        <!-- Door handles -->
        <circle
          :cx="gateOpen ? 35 : 85"
          cy="110"
          r="3"
          class="fill-yellow-600 transition-all duration-1000"
        />
        <circle
          :cx="gateOpen ? 165 : 115"
          cy="110"
          r="3"
          class="fill-yellow-600 transition-all duration-1000"
        />
      </g>

      <g v-else>
        <!-- Turnstile structure -->
        <circle
          cx="100" cy="100"
          r="30"
          class="fill-none stroke-slate-600 dark:stroke-slate-400"
          stroke-width="4"
        />

        <!-- Turnstile arms (rotating) -->
        <g class="origin-center" :style="{ transform: gateOpen ? 'rotate(120deg)' : 'rotate(0deg)', transformOrigin: '100px 100px', transition: 'transform 1s ease-in-out' }">
          <line
            x1="100" y1="70"
            x2="100" y2="130"
            class="stroke-red-600 dark:stroke-red-700"
            stroke-width="6"
            stroke-linecap="round"
          />
          <line
            x1="70" y1="100"
            x2="130" y2="100"
            class="stroke-red-600 dark:stroke-red-700"
            stroke-width="6"
            stroke-linecap="round"
          />
        </g>

        <!-- Center hub -->
        <circle
          cx="100" cy="100"
          r="8"
          class="fill-slate-700 dark:fill-slate-300"
        />
      </g>

      <!-- Status indicator -->
      <circle
        cx="100" cy="30"
        r="8"
        :class="gateOpen ? 'fill-green-500' : 'fill-red-500'"
        class="transition-colors duration-500"
      />

      <!-- Access icon -->
      <g :class="gateOpen ? 'opacity-100' : 'opacity-30'" class="transition-opacity duration-500">
        <path
          d="M 95 25 L 100 30 L 110 20"
          class="stroke-white fill-none"
          stroke-width="2"
          stroke-linecap="round"
        />
      </g>
    </svg>

    <!-- Type indicator -->
    <div class="absolute top-2 left-2 px-2 py-1 bg-white/80 dark:bg-black/60 rounded text-xs font-medium capitalize">
      {{ gateType }}
    </div>

    <!-- Status text -->
    <div class="absolute bottom-2 left-1/2 -translate-x-1/2 px-3 py-1 bg-white/90 dark:bg-black/70 rounded text-xs font-medium">
      <span :class="gateOpen ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
        {{ gateOpen ? 'OPEN' : 'LOCKED' }}
      </span>
      <span class="mx-1">•</span>
      <span class="text-slate-600 dark:text-slate-400">{{ fromArea }} → {{ toArea }}</span>
    </div>
  </div>
</template>
