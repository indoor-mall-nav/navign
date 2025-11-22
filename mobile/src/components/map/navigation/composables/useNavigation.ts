/**
 * Composable for navigation state management
 */

import { computed, ref, watch } from 'vue'
import { getRoute, type RouteResponse } from '@/lib/api/tauri'
import { extractInstructions } from '../utils/extractInstructions'
import { estimateWalkingTime } from '../utils/formatters'
import type {
  NavigationStep,
  RoutePreferences,
  NavigationProgress,
} from '../types'

export function useNavigation(
  entityId: string,
  currentLocation?: number,
  currentExactLocation?: [number, number],
) {
  // State
  const route = ref<RouteResponse | null>(null)
  const currentStep = ref(0)
  const isNavigating = ref(false)
  const loading = ref(false)
  const error = ref('')

  // Route preferences
  const routePreferences = ref<RoutePreferences>({
    elevator: true,
    stairs: true,
    escalator: true,
  })

  // Computed
  const navigationSteps = computed<NavigationStep[]>(() => {
    if (!route.value) return []
    return extractInstructions(route.value.instructions)
  })

  const currentNavigationStep = computed<NavigationStep | null>(() => {
    if (!isNavigating.value || navigationSteps.value.length === 0) return null
    return navigationSteps.value[currentStep.value] || null
  })

  const nextNavigationStep = computed<NavigationStep | null>(() => {
    if (
      !isNavigating.value ||
      currentStep.value >= navigationSteps.value.length - 1
    )
      return null
    return navigationSteps.value[currentStep.value + 1] || null
  })

  const progress = computed<NavigationProgress>(() => {
    const totalSteps = navigationSteps.value.length
    const progressPercentage =
      totalSteps > 0 ? ((currentStep.value + 1) / totalSteps) * 100 : 0
    const progressRatio = totalSteps > 0 ? currentStep.value / totalSteps : 0
    const totalDistance = route.value?.total_distance || 0
    const remainingDistance = totalDistance * (1 - progressRatio)
    const remainingTime = estimateWalkingTime(remainingDistance)

    return {
      currentStep: currentStep.value,
      totalSteps,
      progress: progressPercentage,
      remainingDistance,
      remainingTime,
    }
  })

  // Actions
  async function calculateRoute(targetId: number) {
    if (!currentLocation || !currentExactLocation) {
      error.value = 'Current location is required'
      return false
    }

    loading.value = true
    error.value = ''

    try {
      const result = await getRoute(
        entityId,
        `${currentExactLocation[0]},${currentExactLocation[1]},${currentLocation}`,
        targetId.toString(),
        routePreferences.value,
      )

      if (result.status === 'success' && result.data) {
        route.value = result.data
        currentStep.value = 0
        return true
      } else {
        error.value = result.message || 'Failed to calculate route'
        return false
      }
    } catch (err) {
      error.value = `Error: ${(err as Error).toString()}`
      return false
    } finally {
      loading.value = false
    }
  }

  function startNavigation() {
    if (!route.value) {
      error.value = 'No route available'
      return false
    }
    isNavigating.value = true
    currentStep.value = 0
    return true
  }

  function stopNavigation() {
    isNavigating.value = false
    currentStep.value = 0
  }

  function nextStep() {
    if (navigationSteps.value.length === 0) return false
    if (currentStep.value < navigationSteps.value.length - 1) {
      currentStep.value++
      return true
    } else {
      stopNavigation()
      return false
    }
  }

  function previousStep() {
    if (currentStep.value > 0) {
      currentStep.value--
      return true
    }
    return false
  }

  function clearRoute() {
    route.value = null
    currentStep.value = 0
    isNavigating.value = false
    error.value = ''
  }

  // Watch for location changes
  watch(
    () => currentLocation,
    () => {
      if (isNavigating.value) {
        stopNavigation()
      }
    },
  )

  return {
    // State
    route,
    currentStep,
    isNavigating,
    loading,
    error,
    routePreferences,

    // Computed
    navigationSteps,
    currentNavigationStep,
    nextNavigationStep,
    progress,

    // Actions
    calculateRoute,
    startNavigation,
    stopNavigation,
    nextStep,
    previousStep,
    clearRoute,
  }
}
