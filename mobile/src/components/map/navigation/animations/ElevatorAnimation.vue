<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'

const props = withDefaults(
  defineProps<{
    fromFloor: string
    toFloor: string
    /** Number of floors to pass through */
    floorCount?: number
  }>(),
  {
    floorCount: 5,
  },
)

// Determine direction and floor numbers
const direction = computed(() => {
  const fromNum = parseInt(props.fromFloor.replace(/\D/g, '')) || 0
  const toNum = parseInt(props.toFloor.replace(/\D/g, '')) || 0
  return toNum > fromNum ? 'up' : 'down'
})

const fromFloorNum = computed(() => parseInt(props.fromFloor.replace(/\D/g, '')) || 0)
const toFloorNum = computed(() => parseInt(props.toFloor.replace(/\D/g, '')) || 0)

// Current floor being shown (animated)
const currentFloor = ref(fromFloorNum.value)

onMounted(() => {
  const interval = setInterval(() => {
    if (direction.value === 'up') {
      if (currentFloor.value < toFloorNum.value) {
        currentFloor.value++
      } else {
        currentFloor.value = fromFloorNum.value
      }
    } else {
      if (currentFloor.value > toFloorNum.value) {
        currentFloor.value--
      } else {
        currentFloor.value = fromFloorNum.value
      }
    }
  }, 800)

  return () => clearInterval(interval)
})

// Generate all floors for display
const allFloors = computed(() => {
  const floors: number[] = []
  const start = Math.min(fromFloorNum.value, toFloorNum.value)
  const end = Math.max(fromFloorNum.value, toFloorNum.value)

  for (let i = start; i <= end; i++) {
    floors.push(i)
  }

  return direction.value === 'up' ? floors : floors.reverse()
})
</script>

<template>
  <div class="relative w-full h-48 bg-gradient-to-b from-purple-100 to-purple-200 dark:from-purple-950 dark:to-purple-900 rounded-lg overflow-hidden">
    <!-- Elevator shaft -->
    <div class="absolute left-1/2 top-0 bottom-0 w-24 -ml-12 bg-slate-300 dark:bg-slate-700 border-x-4 border-slate-400 dark:border-slate-600">
      <!-- Floor indicators on the side -->
      <div class="absolute -left-16 top-4 space-y-2">
        <div
          v-for="floor in allFloors"
          :key="floor"
          class="flex items-center gap-2 transition-all duration-300"
        >
          <div
            class="w-2 h-2 rounded-full transition-all duration-300"
            :class="
              floor === currentFloor
                ? 'bg-green-500 scale-150'
                : floor === toFloorNum
                  ? 'bg-blue-500'
                  : 'bg-slate-400'
            "
          ></div>
          <span
            class="text-xs font-medium transition-all duration-300"
            :class="
              floor === currentFloor
                ? 'text-green-600 dark:text-green-400 font-bold'
                : 'text-slate-600 dark:text-slate-400'
            "
          >
            F{{ floor }}
          </span>
        </div>
      </div>

      <!-- Elevator car (animated) -->
      <div
        class="absolute left-0 right-0 h-16 bg-purple-500 dark:bg-purple-600 border-4 border-purple-700 dark:border-purple-800 rounded-sm transition-all duration-700 ease-in-out"
        :style="{
          top: `${((currentFloor - Math.min(fromFloorNum, toFloorNum)) / (Math.abs(toFloorNum - fromFloorNum) || 1)) * 70 + 10}%`,
        }"
      >
        <!-- Elevator door -->
        <div class="absolute inset-1 flex gap-0.5">
          <div class="flex-1 bg-slate-200 dark:bg-slate-800 rounded-sm"></div>
          <div class="flex-1 bg-slate-200 dark:bg-slate-800 rounded-sm"></div>
        </div>

        <!-- Floor display -->
        <div class="absolute -top-8 left-1/2 -ml-6 w-12 h-6 bg-black rounded flex items-center justify-center">
          <span class="text-green-400 font-mono text-sm font-bold">{{ currentFloor }}</span>
        </div>
      </div>

      <!-- Elevator cables -->
      <div class="absolute top-0 left-1/2 w-0.5 h-full bg-slate-600 dark:bg-slate-500 -ml-0.25"></div>
    </div>

    <!-- Direction indicator -->
    <div class="absolute top-2 left-2 px-2 py-1 bg-white/80 dark:bg-black/60 rounded text-xs font-medium flex items-center gap-1">
      <span class="text-lg">{{ direction === 'up' ? '↑' : '↓' }}</span>
      <span>{{ direction.toUpperCase() }}</span>
    </div>

    <!-- Destination info -->
    <div class="absolute bottom-2 left-1/2 -ml-16 px-3 py-1 bg-white/90 dark:bg-black/70 rounded text-xs font-medium">
      Going to Floor {{ toFloorNum }}
    </div>
  </div>
</template>
