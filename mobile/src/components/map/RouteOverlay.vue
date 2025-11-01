<script setup lang="ts">
import { computed } from 'vue'
import type { RouteResponse, RouteInstruction, MapData } from '@/lib/api/tauri'

const props = defineProps<{
  route: RouteResponse
  mapData: MapData
  mapWidth: number
  mapHeight: number
  currentStep?: number
  showTarget?: boolean
  userLocation?: { x: number; y: number } | null
}>()

// Calculate bounds and scaling (same as backend)
const bounds = computed(() => {
  let min_x = Number.MAX_VALUE
  let max_x = Number.MIN_VALUE
  let min_y = Number.MAX_VALUE
  let max_y = Number.MIN_VALUE

  for (const [x, y] of props.mapData.polygon) {
    min_x = Math.min(min_x, x)
    max_x = Math.max(max_x, x)
    min_y = Math.min(min_y, y)
    max_y = Math.max(max_y, y)
  }

  const scale_x = (props.mapWidth - 20) / (max_x - min_x)
  const scale_y = (props.mapHeight - 20) / (max_y - min_y)
  const scale = Math.min(scale_x, scale_y)

  return { min_x, min_y, scale }
})

const transform = (x: number, y: number) => {
  const { min_x, min_y, scale } = bounds.value
  return {
    x: (x - min_x) * scale + 10,
    y: (y - min_y) * scale + 10,
  }
}

// Extract coordinates from instruction
function getInstructionCoords(
  instruction: RouteInstruction,
): [number, number] | null {
  if ('move' in instruction) {
    return instruction.move
  }
  // For transport instructions, we don't have exact coordinates in the instruction
  return null
}

function getInstructionType(instruction: RouteInstruction): string {
  if ('move' in instruction) {
    return 'move'
  } else if ('transport' in instruction) {
    return instruction.transport[2]
  }
  return 'move'
}

const routePoints = computed(() => {
  if (!props.route || !props.route.instructions.length) return []

  const points: Array<{ x: number; y: number; type: string; index: number }> =
    []

  // Add user location as starting point if available
  if (props.userLocation) {
    const startPoint = transform(props.userLocation.x, props.userLocation.y)
    points.push({ ...startPoint, type: 'start', index: -1 })
  }

  // Add all instruction points
  props.route.instructions.forEach((inst, idx) => {
    const coords = getInstructionCoords(inst)
    if (coords) {
      const point = transform(coords[0], coords[1])
      points.push({ ...point, type: getInstructionType(inst), index: idx })
    }
  })

  return points
})

const routePath = computed(() => {
  if (routePoints.value.length < 2) return ''
  return routePoints.value.map((p) => `${p.x},${p.y}`).join(' ')
})

const routeSegments = computed(() => {
  if (routePoints.value.length < 2) return []

  const segments = []
  for (let i = 0; i < routePoints.value.length - 1; i++) {
    const from = routePoints.value[i]
    const to = routePoints.value[i + 1]

    const isPassed = props.currentStep !== undefined && i < props.currentStep
    const isCurrent = props.currentStep === i

    segments.push({
      from,
      to,
      type: to.type,
      isPassed,
      isCurrent,
      index: i,
    })
  }

  return segments
})

const startPoint = computed(() => {
  return routePoints.value.length > 0 ? routePoints.value[0] : null
})

const endPoint = computed(() => {
  return routePoints.value.length > 0
    ? routePoints.value[routePoints.value.length - 1]
    : null
})

const currentPosition = computed(() => {
  if (props.currentStep === undefined || !routePoints.value.length) return null
  if (props.currentStep >= routePoints.value.length) return null
  return routePoints.value[props.currentStep]
})

function getSegmentColor(
  type: string,
  isPassed: boolean,
  isCurrent: boolean,
): string {
  if (isPassed) return '#9ca3af' // gray for passed segments
  if (isCurrent) return '#3b82f6' // blue for current segment

  switch (type) {
    case 'move':
      return '#10b981' // green
    case 'elevator':
      return '#8b5cf6' // purple
    case 'stairs':
      return '#f97316' // orange
    case 'escalator':
      return '#14b8a6' // teal
    case 'gate':
    case 'turnstile':
      return '#ef4444' // red
    default:
      return '#6b7280' // gray
  }
}

function getSegmentWidth(isCurrent: boolean): number {
  return isCurrent ? 6 : 4
}
</script>

<template>
  <svg
    :width="mapWidth"
    :height="mapHeight"
    class="absolute top-0 left-0 pointer-events-none"
    style="z-index: 10"
  >
    <defs>
      <!-- Arrow marker for route direction -->
      <marker
        id="arrowhead"
        markerWidth="10"
        markerHeight="10"
        refX="9"
        refY="3"
        orient="auto"
      >
        <polygon points="0 0, 10 3, 0 6" fill="#10b981" />
      </marker>
    </defs>

    <!-- Route polyline (full path) -->
    <polyline
      v-if="routePath"
      :points="routePath"
      fill="none"
      stroke="#10b981"
      stroke-width="3"
      stroke-opacity="0.3"
      stroke-linejoin="round"
      stroke-linecap="round"
    />

    <!-- Individual route segments with different colors -->
    <g v-for="segment in routeSegments" :key="segment.index">
      <line
        :x1="segment.from.x"
        :y1="segment.from.y"
        :x2="segment.to.x"
        :y2="segment.to.y"
        :stroke="
          getSegmentColor(segment.type, segment.isPassed, segment.isCurrent)
        "
        :stroke-width="getSegmentWidth(segment.isCurrent)"
        stroke-linecap="round"
        :stroke-dasharray="segment.isCurrent ? '10 5' : 'none'"
        :class="{ 'animate-dash': segment.isCurrent }"
      />

      <!-- Direction arrows on segments -->
      <g v-if="!segment.isPassed && segment.index % 2 === 0">
        <line
          :x1="(segment.from.x + segment.to.x) / 2"
          :y1="(segment.from.y + segment.to.y) / 2"
          :x2="(segment.from.x + segment.to.x) / 2"
          :y2="(segment.from.y + segment.to.y) / 2"
          :stroke="
            getSegmentColor(segment.type, segment.isPassed, segment.isCurrent)
          "
          stroke-width="2"
          marker-end="url(#arrowhead)"
        />
      </g>

      <!-- Step markers -->
      <circle
        :cx="segment.from.x"
        :cy="segment.from.y"
        :r="segment.isCurrent ? 8 : 5"
        :fill="segment.isCurrent ? '#3b82f6' : '#fff'"
        :stroke="
          getSegmentColor(segment.type, segment.isPassed, segment.isCurrent)
        "
        :stroke-width="segment.isCurrent ? 3 : 2"
      />
    </g>

    <!-- Start point marker -->
    <g v-if="startPoint">
      <circle
        :cx="startPoint.x"
        :cy="startPoint.y"
        r="12"
        fill="#10b981"
        stroke="#fff"
        stroke-width="3"
      />
      <text
        :x="startPoint.x"
        :y="startPoint.y + 4"
        text-anchor="middle"
        fill="#fff"
        font-size="12"
        font-weight="bold"
      >
        S
      </text>
    </g>

    <!-- End point marker (target) -->
    <g v-if="endPoint && showTarget">
      <circle
        :cx="endPoint.x"
        :cy="endPoint.y"
        r="12"
        fill="#ef4444"
        stroke="#fff"
        stroke-width="3"
      />
      <!-- Pulsing animation for target -->
      <circle
        :cx="endPoint.x"
        :cy="endPoint.y"
        r="12"
        fill="#ef4444"
        opacity="0.5"
      >
        <animate
          attributeName="r"
          from="12"
          to="24"
          dur="1.5s"
          repeatCount="indefinite"
        />
        <animate
          attributeName="opacity"
          from="0.5"
          to="0"
          dur="1.5s"
          repeatCount="indefinite"
        />
      </circle>
      <text
        :x="endPoint.x"
        :y="endPoint.y + 4"
        text-anchor="middle"
        fill="#fff"
        font-size="12"
        font-weight="bold"
      >
        T
      </text>
    </g>

    <!-- Current position indicator (when navigating) -->
    <g v-if="currentPosition">
      <circle
        :cx="currentPosition.x"
        :cy="currentPosition.y"
        r="10"
        fill="#3b82f6"
        stroke="#fff"
        stroke-width="3"
      />
      <!-- Pulsing effect -->
      <circle
        :cx="currentPosition.x"
        :cy="currentPosition.y"
        r="10"
        fill="#3b82f6"
        opacity="0.6"
      >
        <animate
          attributeName="r"
          from="10"
          to="20"
          dur="1s"
          repeatCount="indefinite"
        />
        <animate
          attributeName="opacity"
          from="0.6"
          to="0"
          dur="1s"
          repeatCount="indefinite"
        />
      </circle>
    </g>
  </svg>
</template>

<style scoped>
.animate-dash {
  animation: dash 1s linear infinite;
}

@keyframes dash {
  to {
    stroke-dashoffset: -20;
  }
}
</style>
