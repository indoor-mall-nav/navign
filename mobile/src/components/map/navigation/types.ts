/**
 * Type definitions for navigation
 */

/**
 * Navigation step type
 */
export interface NavigationStep {
  type: 'straight' | 'turn' | 'transport' | 'unlock'
  straight?: number
  transport?: [string, string, string] // [from_area, to_area, transport_type]
  turn?: 'left' | 'right' | 'around'
}

/**
 * Route preferences for pathfinding
 */
export interface RoutePreferences {
  elevator: boolean
  stairs: boolean
  escalator: boolean
}

/**
 * Navigation state
 */
export interface NavigationState {
  isNavigating: boolean
  currentStep: number
  route: object | null // RouteResponse from API
  targetId: string | null
  error: string | null
}

/**
 * Navigation progress information
 */
export interface NavigationProgress {
  currentStep: number
  totalSteps: number
  progress: number // Percentage (0-100)
  remainingDistance: number // In meters
  remainingTime: number // In seconds (estimated)
}
