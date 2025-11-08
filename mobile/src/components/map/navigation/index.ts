/**
 * Navigation module exports
 */

export { default as NavigationPanel } from './NavigationPanel.vue'
export { default as NavigationDestinationSelector } from './components/NavigationDestinationSelector.vue'
export { default as NavigationRoutePreview } from './components/NavigationRoutePreview.vue'
export { default as NavigationActiveView } from './components/NavigationActiveView.vue'
export { default as NavigationStepCard } from './components/NavigationStepCard.vue'

export { useNavigation } from './composables/useNavigation'
export { extractInstructions } from './utils/extractInstructions'
export { formatDistance, formatTime, estimateWalkingTime } from './utils/formatters'
export {
  getNavigationStepIcon,
  getNavigationStepColor,
  getNavigationStepTitle,
  getNavigationStepDescription,
} from './utils/icons'

export type * from './types'
