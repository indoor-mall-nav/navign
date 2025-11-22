<script setup lang="ts">
import { ref, watch } from 'vue'
import { type MapMerchant, type RouteResponse } from '@/lib/api/tauri'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Icon } from '@iconify/vue'
import NavigationDestinationSelector from './components/NavigationDestinationSelector.vue'
import NavigationRoutePreview from './components/NavigationRoutePreview.vue'
import NavigationActiveView from './components/NavigationActiveView.vue'
import { useNavigation } from './composables/useNavigation'

const props = withDefaults(
  defineProps<{
    entityId: string
    currentLocation?: number // merchant/area id
    currentExactLocation?: [number, number] // precise coordinates if available
    merchants: MapMerchant[]
  }>(),
  {},
)

const emit = defineEmits<{
  routeCalculated: [route: RouteResponse]
  navigationStarted: [targetId: number]
  navigationEnded: []
}>()

// Selected target merchant
const selectedTarget = ref<MapMerchant | null>(null)

// Use navigation composable
const {
  route,
  isNavigating,
  loading,
  error,
  routePreferences,
  navigationSteps,
  currentNavigationStep,
  nextNavigationStep,
  progress,
  calculateRoute,
  startNavigation,
  stopNavigation,
  nextStep,
  previousStep,
  clearRoute,
} = useNavigation(
  props.entityId,
  props.currentLocation,
  props.currentExactLocation,
)

// Handlers
function handleSelectDestination(merchant: MapMerchant) {
  selectedTarget.value = merchant
}

async function handleCalculateRoute(merchant: MapMerchant) {
  selectedTarget.value = merchant
  const success = await calculateRoute(merchant.id)
  if (success && route.value) {
    emit('routeCalculated', route.value)
  }
}

function handleStartNavigation() {
  if (!selectedTarget.value) return
  const success = startNavigation()
  if (success) {
    emit('navigationStarted', selectedTarget.value.id)
  }
}

function handleStopNavigation() {
  stopNavigation()
  emit('navigationEnded')
}

function handleClearRoute() {
  clearRoute()
  selectedTarget.value = null
}

function handleUpdatePreferences(newPreferences: any) {
  routePreferences.value = newPreferences
}

// Watch for location changes
watch(
  () => props.currentLocation,
  () => {
    if (isNavigating.value) {
      handleStopNavigation()
    }
  },
)
</script>

<template>
  <div class="navigation-panel space-y-4 w-full">
    <!-- Destination Selection (when no route) -->
    <NavigationDestinationSelector
      v-if="!route"
      :merchants="merchants"
      :preferences="routePreferences"
      :loading="loading"
      :error="error"
      :entity-id="entityId"
      @select-destination="handleSelectDestination"
      @calculate-route="handleCalculateRoute"
      @update:preferences="handleUpdatePreferences"
    />

    <!-- Route Preview (when route calculated but not navigating) -->
    <NavigationRoutePreview
      v-if="route && !isNavigating && selectedTarget"
      :destination-name="selectedTarget.name"
      :steps="navigationSteps"
      @start-navigation="handleStartNavigation"
      @clear-route="handleClearRoute"
    />

    <!-- Active Navigation (when navigating) -->
    <NavigationActiveView
      v-if="isNavigating && currentNavigationStep && selectedTarget"
      :destination-name="selectedTarget.name"
      :current-step="currentNavigationStep"
      :next-step="nextNavigationStep"
      :progress="progress"
      :entity-id="entityId"
      :target-id="selectedTarget.id"
      @next-step="nextStep"
      @previous-step="previousStep"
      @stop-navigation="handleStopNavigation"
    />

    <!-- Help Card (when no route) -->
    <Card v-if="!route" class="bg-muted/50">
      <CardHeader>
        <CardTitle class="text-sm flex items-center gap-2">
          <Icon icon="mdi:information" class="w-4 h-4" />
          How to Navigate
        </CardTitle>
      </CardHeader>
      <CardContent class="text-sm space-y-2">
        <ol class="list-decimal list-inside space-y-1 text-muted-foreground">
          <li>Search for your destination</li>
          <li>Select route preferences</li>
          <li>Calculate the route</li>
          <li>Start navigation for turn-by-turn directions</li>
        </ol>
      </CardContent>
    </Card>
  </div>
</template>

<style scoped>
.navigation-panel {
  @apply w-full;
}
</style>
