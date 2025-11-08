<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import NavigationStepCard from './NavigationStepCard.vue'
import type { NavigationStep } from '../types'

defineProps<{
  destinationName: string
  steps: NavigationStep[]
}>()

const emit = defineEmits<{
  'start-navigation': []
  'clear-route': []
}>()

function handleStartNavigation() {
  emit('start-navigation')
}

function handleClearRoute() {
  emit('clear-route')
}
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Icon icon="mdi:map-marker-path" class="w-5 h-5" />
          Route to {{ destinationName }}
        </div>
        <Button variant="ghost" size="icon" @click="handleClearRoute">
          <Icon icon="mdi:close" class="w-5 h-5" />
        </Button>
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      <!-- Route Instructions Preview -->
      <div class="max-h-80 overflow-y-auto space-y-2">
        <NavigationStepCard
          v-for="(step, idx) in steps"
          :key="idx"
          :step="step"
          :index="idx"
          :is-preview="true"
          size="sm"
        />
      </div>

      <!-- Start Navigation Button -->
      <Button class="w-full" @click="handleStartNavigation">
        <Icon icon="mdi:navigation" class="w-4 h-4 mr-2" />
        Start Navigation
      </Button>
    </CardContent>
  </Card>
</template>
