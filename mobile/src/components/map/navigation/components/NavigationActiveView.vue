<script setup lang="ts">
import { ref } from 'vue'
import { Icon } from '@iconify/vue'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { unlockDevice } from '@/lib/api/tauri'
import { info } from '@tauri-apps/plugin-log'
import NavigationStepCard from './NavigationStepCard.vue'
import { formatDistance } from '../utils/formatters'
import type { NavigationStep, NavigationProgress } from '../types'

const props = defineProps<{
  destinationName: string
  currentStep: NavigationStep
  nextStep?: NavigationStep | null
  progress: NavigationProgress
  entityId: string
  targetId: string
}>()

const emit = defineEmits<{
  'next-step': []
  'previous-step': []
  'stop-navigation': []
}>()

const unlockErrorMessage = ref('')

function handleNextStep() {
  emit('next-step')
}

function handlePreviousStep() {
  emit('previous-step')
}

function handleStopNavigation() {
  emit('stop-navigation')
}

async function unlockDoor() {
  unlockErrorMessage.value = ''
  await info(`Unlocking door for ${props.destinationName}`)

  try {
    const res = await unlockDevice(props.entityId, props.targetId)
    if (res.status === 'success') {
      handleNextStep()
    } else {
      unlockErrorMessage.value = 'Failed to unlock door: ' + res.message
    }
  } catch (err) {
    unlockErrorMessage.value = `Error: ${err}`
  }
}

const isLastStep = (progress: NavigationProgress) =>
  progress.currentStep === progress.totalSteps - 1
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Icon icon="mdi:navigation" class="w-5 h-5 text-primary" />
          Navigating
        </div>
        <Button variant="ghost" size="icon" @click="handleStopNavigation">
          <Icon icon="mdi:stop" class="w-5 h-5" />
        </Button>
      </CardTitle>
      <CardDescription> To {{ destinationName }} </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <!-- Progress Bar -->
      <div class="space-y-2">
        <div class="flex items-center justify-between text-sm">
          <span>Step {{ progress.currentStep + 1 }} of {{ progress.totalSteps }}</span>
          <span class="text-muted-foreground">
            {{ formatDistance(progress.remainingDistance) }} remaining
          </span>
        </div>
        <div class="w-full bg-secondary rounded-full h-2">
          <div
            class="bg-primary h-2 rounded-full transition-all duration-300"
            :style="{ width: `${progress.progress}%` }"
          ></div>
        </div>
      </div>

      <Separator />

      <!-- Current Instruction -->
      <div class="space-y-4">
        <NavigationStepCard :step="currentStep" size="lg">
          <template #actions>
            <Button
              v-if="currentStep.type === 'unlock'"
              class="mt-2"
              size="sm"
              @click="unlockDoor"
            >
              <Icon icon="mdi:lock-open-variant" class="w-4 h-4 mr-1" />
              Unlock
            </Button>
            <p v-if="unlockErrorMessage" class="text-sm text-red-500 mt-1">
              {{ unlockErrorMessage }}
            </p>
          </template>
        </NavigationStepCard>

        <!-- Next Instruction Preview -->
        <div v-if="nextStep" class="p-3 rounded-lg border bg-card/50">
          <p class="text-xs text-muted-foreground mb-2">Next:</p>
          <NavigationStepCard :step="nextStep" size="sm" :is-preview="true" />
        </div>
      </div>

      <!-- Navigation Controls -->
      <div class="flex gap-2">
        <Button
          variant="outline"
          class="flex-1"
          :disabled="progress.currentStep === 0"
          @click="handlePreviousStep"
        >
          <Icon icon="mdi:chevron-left" class="w-4 h-4 mr-1" />
          Previous
        </Button>
        <Button class="flex-1" @click="handleNextStep">
          {{ isLastStep(progress) ? 'Finish' : 'Next' }}
          <Icon icon="mdi:chevron-right" class="w-4 h-4 ml-1" />
        </Button>
      </div>
    </CardContent>
  </Card>
</template>

