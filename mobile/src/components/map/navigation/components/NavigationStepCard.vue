<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { Badge } from '@/components/ui/badge'
import {
  getNavigationStepIcon,
  getNavigationStepColor,
  getNavigationStepTitle,
  getNavigationStepDescription,
} from '../utils/icons'
import { formatDistance } from '../utils/formatters'
import type { NavigationStep } from '../types'
import StairsAnimation from '../animations/StairsAnimation.vue'
import ElevatorAnimation from '../animations/ElevatorAnimation.vue'
import EscalatorAnimation from '../animations/EscalatorAnimation.vue'
import GateAnimation from '../animations/GateAnimation.vue'

const props = withDefaults(
  defineProps<{
    step: NavigationStep
    index?: number
    isCurrent?: boolean
    isPreview?: boolean
    size?: 'sm' | 'md' | 'lg'
  }>(),
  {
    isCurrent: false,
    isPreview: false,
    size: 'md',
  },
)

const icon = computed(() => getNavigationStepIcon(props.step))
const color = computed(() => getNavigationStepColor(props.step))
const title = computed(() => getNavigationStepTitle(props.step))
const description = computed(() =>
  getNavigationStepDescription(props.step, formatDistance),
)

const iconSize = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'w-4 h-4'
    case 'lg':
      return 'w-8 h-8'
    default:
      return 'w-5 h-5'
  }
})

const containerSize = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'w-6 h-6'
    case 'lg':
      return 'w-16 h-16'
    default:
      return 'w-8 h-8'
  }
})

// Show animation for transport steps when in large size and not preview
const showAnimation = computed(() => {
  return props.step.type === 'transport' && props.size === 'lg' && !props.isPreview
})

// Get transport type for animation selection
const transportType = computed(() => {
  if (props.step.type === 'transport' && props.step.transport) {
    return props.step.transport[2]
  }
  return null
})

const fromArea = computed(() => {
  if (props.step.type === 'transport' && props.step.transport) {
    return props.step.transport[0]
  }
  return ''
})

const toArea = computed(() => {
  if (props.step.type === 'transport' && props.step.transport) {
    return props.step.transport[1]
  }
  return ''
})
</script>

<template>
  <div
    class="rounded-lg border bg-card transition-colors"
    :class="{
      'bg-accent border-primary': isCurrent,
      'hover:bg-accent/50': !isCurrent && !isPreview,
    }"
  >
    <div class="flex items-start gap-3 p-3">
      <div class="flex-shrink-0 mt-1">
        <div
          :class="[
            'rounded-full bg-accent flex items-center justify-center',
            containerSize,
            { 'bg-background': size === 'lg' },
          ]"
        >
          <Icon :icon="icon" :class="[iconSize, color]" />
        </div>
      </div>
      <div class="flex-1 min-w-0">
        <div class="flex items-center justify-between">
          <span
            :class="[
              'font-medium capitalize',
              size === 'sm' ? 'text-xs' : 'text-sm',
            ]"
          >
            {{ title }}
          </span>
          <Badge v-if="index !== undefined" variant="outline" class="text-xs">
            Step {{ index + 1 }}
          </Badge>
        </div>
        <p
          :class="[
            'text-muted-foreground mt-1',
            size === 'sm' ? 'text-xs' : 'text-sm',
          ]"
        >
          {{ description }}
        </p>
        <slot name="actions"></slot>
      </div>
    </div>

    <!-- Transport animations -->
    <div v-if="showAnimation" class="px-3 pb-3">
      <StairsAnimation
        v-if="transportType === 'stairs'"
        :from-floor="fromArea"
        :to-floor="toArea"
      />
      <ElevatorAnimation
        v-else-if="transportType === 'elevator'"
        :from-floor="fromArea"
        :to-floor="toArea"
      />
      <EscalatorAnimation
        v-else-if="transportType === 'escalator'"
        :from-floor="fromArea"
        :to-floor="toArea"
      />
      <GateAnimation
        v-else-if="transportType === 'gate' || transportType === 'turnstile'"
        :from-area="fromArea"
        :to-area="toArea"
        :gate-type="transportType as 'gate' | 'turnstile'"
      />
    </div>
  </div>
</template>
