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
</script>

<template>
  <div
    class="flex items-start gap-3 p-3 rounded-lg border bg-card transition-colors"
    :class="{
      'bg-accent border-primary': isCurrent,
      'hover:bg-accent/50': !isCurrent && !isPreview,
    }"
  >
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
</template>
