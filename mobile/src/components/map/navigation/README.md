# Navigation Module

This module provides a complete navigation solution for indoor navigation, integrated into the map components.

## Structure

```
navigation/
├── components/           # UI Components
│   ├── NavigationDestinationSelector.vue   # Search and select destination
│   ├── NavigationRoutePreview.vue          # Preview route before starting
│   ├── NavigationActiveView.vue            # Active turn-by-turn navigation
│   └── NavigationStepCard.vue              # Individual step display
├── composables/          # Composable functions
│   └── useNavigation.ts                    # Navigation state management
├── utils/                # Utility functions
│   ├── extractInstructions.ts              # Convert route to instructions
│   ├── formatters.ts                       # Distance/time formatting
│   └── icons.ts                            # Icon selection logic
├── types.ts              # TypeScript type definitions
├── NavigationPanel.vue   # Main navigation panel component
└── index.ts              # Module exports

## Usage

### Basic Usage

```vue
<script setup lang="ts">
import { NavigationPanel } from '@/components/map/navigation'

const merchants = ref<MapMerchant[]>([])
const entityId = 'entity-123'
const currentLocation = 'area-456'
const currentExactLocation = [10.5, 20.3]
</script>

<template>
  <NavigationPanel
    :entity-id="entityId"
    :current-location="currentLocation"
    :current-exact-location="currentExactLocation"
    :merchants="merchants"
    @route-calculated="handleRouteCalculated"
    @navigation-started="handleNavigationStarted"
    @navigation-ended="handleNavigationEnded"
  />
</template>
```

### Using the Composable

```typescript
import { useNavigation } from '@/components/map/navigation'

const {
  route,
  isNavigating,
  navigationSteps,
  currentNavigationStep,
  progress,
  calculateRoute,
  startNavigation,
  stopNavigation,
  nextStep,
  previousStep,
} = useNavigation(entityId, currentLocation, currentExactLocation)

// Calculate a route
await calculateRoute(targetMerchantId)

// Start navigation
startNavigation()

// Navigate through steps
nextStep()
previousStep()

// Stop navigation
stopNavigation()
```

### Using Individual Components

```vue
<script setup lang="ts">
import {
  NavigationDestinationSelector,
  NavigationRoutePreview,
  NavigationActiveView,
} from '@/components/map/navigation'
</script>

<template>
  <!-- Destination selection -->
  <NavigationDestinationSelector
    :merchants="merchants"
    :preferences="preferences"
    @calculate-route="handleCalculate"
  />

  <!-- Route preview -->
  <NavigationRoutePreview
    :destination-name="destination"
    :steps="steps"
    @start-navigation="handleStart"
  />

  <!-- Active navigation -->
  <NavigationActiveView
    :destination-name="destination"
    :current-step="currentStep"
    :progress="progress"
    @next-step="handleNext"
    @stop-navigation="handleStop"
  />
</template>
```

## Type Definitions

### NavigationStep

```typescript
interface NavigationStep {
  type: 'straight' | 'turn' | 'transport' | 'unlock'
  straight?: number
  transport?: [string, string, string]
  turn?: 'left' | 'right' | 'around'
}
```

### RoutePreferences

```typescript
interface RoutePreferences {
  elevator: boolean
  stairs: boolean
  escalator: boolean
}
```

### NavigationProgress

```typescript
interface NavigationProgress {
  currentStep: number
  totalSteps: number
  progress: number // Percentage (0-100)
  remainingDistance: number // In meters
  remainingTime: number // In seconds (estimated)
}
```

## Utility Functions

### Formatters

- `formatDistance(meters)` - Format distance to human-readable string
- `formatTime(seconds)` - Format time to human-readable string
- `estimateWalkingTime(meters)` - Estimate walking time based on distance

### Icon Helpers

- `getNavigationStepIcon(step)` - Get icon name for a step
- `getNavigationStepColor(step)` - Get color class for a step
- `getNavigationStepTitle(step)` - Get human-readable title
- `getNavigationStepDescription(step)` - Get human-readable description

## Features

- **Modular Components**: Each part of navigation is a separate, reusable component
- **Composable State Management**: Use the `useNavigation` composable for centralized state
- **Type Safety**: Full TypeScript support with detailed type definitions
- **Customizable**: Easy to extend and customize individual components
- **Responsive**: Works on mobile and desktop
- **Accessibility**: Proper ARIA labels and keyboard navigation

## Architecture

The navigation module follows a modular architecture:

1. **Presentation Layer** (Components): UI components that display navigation information
2. **Business Logic Layer** (Composables): State management and navigation logic
3. **Utility Layer** (Utils): Pure functions for formatting and helper operations
4. **Type Layer** (Types): TypeScript interfaces and type definitions

This separation ensures:
- Easy testing
- Reusability
- Maintainability
- Clear separation of concerns
