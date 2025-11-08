/**
 * Utility functions for navigation icon selection
 */

import type { NavigationStep } from '../types'

/**
 * Get icon for a navigation step
 * @param step - Navigation step
 * @returns Icon name (from Iconify MDI collection)
 */
export function getNavigationStepIcon(step: NavigationStep): string {
  switch (step.type) {
    case 'straight':
      return 'mdi:arrow-up'
    case 'turn':
      if (step.turn === 'left') return 'mdi:arrow-left'
      if (step.turn === 'right') return 'mdi:arrow-right'
      if (step.turn === 'around') return 'mdi:arrow-u-left-top'
      return 'mdi:navigation'
    case 'transport':
      const transportType = step.transport?.[2]
      if (transportType === 'elevator') return 'mdi:elevator'
      if (transportType === 'stairs') return 'mdi:stairs'
      if (transportType === 'escalator') return 'mdi:escalator'
      if (transportType === 'gate') return 'mdi:gate'
      if (transportType === 'turnstile') return 'mdi:turnstile'
      return 'mdi:transit-connection-variant'
    case 'unlock':
      return 'mdi:lock-open-variant'
    default:
      return 'mdi:navigation'
  }
}

/**
 * Get color class for a navigation step
 * @param step - Navigation step
 * @returns Tailwind color class
 */
export function getNavigationStepColor(step: NavigationStep): string {
  switch (step.type) {
    case 'straight':
      return 'text-blue-500'
    case 'turn':
      return 'text-yellow-500'
    case 'transport':
      const transportType = step.transport?.[2]
      if (transportType === 'elevator') return 'text-purple-500'
      if (transportType === 'stairs') return 'text-orange-500'
      if (transportType === 'escalator') return 'text-green-500'
      if (transportType === 'gate' || transportType === 'turnstile')
        return 'text-red-500'
      return 'text-gray-500'
    case 'unlock':
      return 'text-emerald-500'
    default:
      return 'text-gray-500'
  }
}

/**
 * Get title for a navigation step
 * @param step - Navigation step
 * @returns Human-readable title
 */
export function getNavigationStepTitle(step: NavigationStep): string {
  switch (step.type) {
    case 'straight':
      return 'Walk Straight'
    case 'turn':
      if (step.turn === 'left') return 'Turn Left'
      if (step.turn === 'right') return 'Turn Right'
      if (step.turn === 'around') return 'Turn Around'
      return 'Turn'
    case 'transport':
      const transportType = step.transport?.[2]
      return `Take ${transportType?.charAt(0).toUpperCase()}${transportType?.slice(1)}`
    case 'unlock':
      return 'Unlock Door'
    default:
      return 'Navigate'
  }
}

/**
 * Get description for a navigation step
 * @param step - Navigation step
 * @param formatDistance - Function to format distance
 * @returns Human-readable description
 */
export function getNavigationStepDescription(
  step: NavigationStep,
  formatDistance: (meters: number) => string,
): string {
  switch (step.type) {
    case 'straight':
      return `Walk straight for ${formatDistance(step.straight || 0)}`
    case 'turn':
      if (step.turn === 'left') return 'Turn left'
      if (step.turn === 'right') return 'Turn right'
      if (step.turn === 'around') return 'Turn around'
      return 'Turn'
    case 'transport':
      const transportType = step.transport?.[2]
      const targetArea = step.transport?.[1]
      return `Take ${transportType} to ${targetArea || 'next area'}`
    case 'unlock':
      return 'Unlock the door to access your destination'
    default:
      return 'Continue'
  }
}
