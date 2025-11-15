<script setup lang="ts">
import { computed } from 'vue'
import { useOffline } from '@/lib/offline'
import { Icon } from '@iconify/vue'
import { Badge } from '@/components/ui/badge'

interface Props {
  showWhenOnline?: boolean
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showWhenOnline: false,
  compact: false,
})

const { isOnline, offlineDuration, queuedActions } = useOffline()

const statusText = computed(() => {
  if (isOnline.value) {
    return 'Online'
  }

  const duration = offlineDuration.value
  const minutes = Math.floor(duration / (1000 * 60))
  const hours = Math.floor(duration / (1000 * 60 * 60))

  if (hours > 0) {
    return `Offline for ${hours}h`
  } else if (minutes > 0) {
    return `Offline for ${minutes}m`
  } else {
    return 'Offline'
  }
})

const statusIcon = computed(() => {
  return isOnline.value ? 'mdi:wifi' : 'mdi:wifi-off'
})

const statusVariant = computed(() => {
  return isOnline.value ? 'default' : 'destructive'
})
</script>

<template>
  <div v-if="!isOnline || showWhenOnline">
    <!-- Compact Mode -->
    <Badge v-if="compact" :variant="statusVariant" class="flex items-center gap-1">
      <Icon :icon="statusIcon" class="w-3 h-3" />
      <span v-if="!isOnline">{{ statusText }}</span>
    </Badge>

    <!-- Full Mode -->
    <div
      v-else
      :class="[
        'fixed top-0 left-0 right-0 z-50 py-2 px-4 flex items-center justify-center gap-2 text-sm font-medium',
        isOnline
          ? 'bg-green-500 text-white'
          : 'bg-red-500 text-white',
      ]"
    >
      <Icon :icon="statusIcon" class="w-4 h-4" />
      <span>{{ statusText }}</span>
      <span v-if="!isOnline && queuedActions > 0" class="ml-2 text-xs opacity-90">
        ({{ queuedActions }} pending action{{ queuedActions > 1 ? 's' : '' }})
      </span>
    </div>
  </div>
</template>
