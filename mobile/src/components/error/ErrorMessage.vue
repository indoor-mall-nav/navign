<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { Button } from '@/components/ui/button'

interface Props {
  error: string | Error | null
  title?: string
  type?: 'error' | 'warning' | 'info'
  dismissible?: boolean
  retryable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: 'Error',
  type: 'error',
  dismissible: true,
  retryable: false,
})

const emit = defineEmits<{
  dismiss: []
  retry: []
}>()

const errorMessage = computed(() => {
  if (!props.error) return ''
  return typeof props.error === 'string' ? props.error : props.error.message
})

const iconName = computed(() => {
  switch (props.type) {
    case 'warning':
      return 'mdi:alert'
    case 'info':
      return 'mdi:information'
    default:
      return 'mdi:alert-circle'
  }
})

const colorClasses = computed(() => {
  switch (props.type) {
    case 'warning':
      return {
        bg: 'bg-yellow-50 dark:bg-yellow-900/20',
        border: 'border-yellow-200 dark:border-yellow-800',
        text: 'text-yellow-800 dark:text-yellow-200',
        icon: 'text-yellow-600 dark:text-yellow-400',
      }
    case 'info':
      return {
        bg: 'bg-blue-50 dark:bg-blue-900/20',
        border: 'border-blue-200 dark:border-blue-800',
        text: 'text-blue-800 dark:text-blue-200',
        icon: 'text-blue-600 dark:text-blue-400',
      }
    default:
      return {
        bg: 'bg-red-50 dark:bg-red-900/20',
        border: 'border-red-200 dark:border-red-800',
        text: 'text-red-800 dark:text-red-200',
        icon: 'text-red-600 dark:text-red-400',
      }
  }
})
</script>

<template>
  <div
    v-if="error"
    :class="[
      'rounded-md border p-4',
      colorClasses.bg,
      colorClasses.border,
    ]"
  >
    <div class="flex items-start gap-3">
      <Icon :icon="iconName" :class="['w-5 h-5 flex-shrink-0 mt-0.5', colorClasses.icon]" />
      <div class="flex-1 min-w-0">
        <h3 :class="['text-sm font-medium', colorClasses.text]">{{ title }}</h3>
        <p :class="['text-sm mt-1', colorClasses.text]">{{ errorMessage }}</p>
        <div v-if="retryable" class="mt-3">
          <Button size="sm" variant="outline" @click="emit('retry')">
            <Icon icon="mdi:refresh" class="w-4 h-4 mr-2" />
            Retry
          </Button>
        </div>
      </div>
      <Button
        v-if="dismissible"
        variant="ghost"
        size="sm"
        @click="emit('dismiss')"
        :class="colorClasses.text"
      >
        <Icon icon="mdi:close" class="w-4 h-4" />
      </Button>
    </div>
  </div>
</template>
