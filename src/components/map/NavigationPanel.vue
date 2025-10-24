<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  getRoute,
  type MapMerchant,
  type RouteResponse,
  unlockDevice,
} from "@/lib/api/tauri";
import { extractInstructions } from "./extractInstructions";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import { Icon } from "@iconify/vue";
import { Label } from "@/components/ui/label";
import { info } from "@tauri-apps/plugin-log";

const props = defineProps<{
  entityId: string;
  currentLocation?: string; // merchant/area id
  currentExactLocation?: [number, number]; // precise coordinates if available
  merchants: MapMerchant[];
}>();

const emit = defineEmits<{
  routeCalculated: [route: RouteResponse];
  navigationStarted: [targetId: string];
  navigationEnded: [];
}>();

const searchQuery = ref("");
const selectedTarget = ref<MapMerchant | null>(null);
const route = ref<RouteResponse | null>(null);
const currentStep = ref(0);
const loading = ref(false);
const error = ref("");
const isNavigating = ref(false);

// Connectivity options
const allowElevator = ref(true);
const allowStairs = ref(true);
const allowEscalator = ref(true);

const filteredMerchants = computed(() => {
  if (!searchQuery.value) return props.merchants;
  const query = searchQuery.value.toLowerCase();
  return props.merchants.filter(
    (m) =>
      m.name.toLowerCase().includes(query) ||
      m.tags.some((tag) => tag.toLowerCase().includes(query)),
  );
});

const navigationSteps = computed(() => {
  if (!route.value) return [];
  return extractInstructions(route.value.instructions);
});

const currentNavigationStep = computed(() => {
  if (!isNavigating.value || navigationSteps.value.length === 0) return null;
  return navigationSteps.value[currentStep.value];
});

const progress = computed(() => {
  if (navigationSteps.value.length === 0) return 0;
  return ((currentStep.value + 1) / navigationSteps.value.length) * 100;
});

const remainingDistance = computed(() => {
  if (!route.value || !isNavigating.value) return 0;
  // Estimate based on progress through instructions
  const progressRatio = currentStep.value / navigationSteps.value.length;
  return route.value.total_distance * (1 - progressRatio);
});

function selectTarget(merchant: MapMerchant) {
  selectedTarget.value = merchant;
  error.value = "";
}

async function calculateRoute() {
  if (!props.currentLocation || !selectedTarget.value) {
    error.value = "Please select a destination";
    return;
  }

  loading.value = true;
  error.value = "";

  try {
    const result = await getRoute(
      props.entityId,
      `${props.currentExactLocation[0]},${props.currentExactLocation[1]},${props.currentLocation}`,
      selectedTarget.value.id,
      {
        elevator: allowElevator.value,
        stairs: allowStairs.value,
        escalator: allowEscalator.value,
      },
    );

    if (result.status === "success" && result.data) {
      route.value = result.data;
      currentStep.value = 0;
      emit("routeCalculated", result.data);
    } else {
      error.value = result.message || "Failed to calculate route";
    }
  } catch (err) {
    error.value = `Error: ${err}`;
  } finally {
    loading.value = false;
  }
}

function startNavigation() {
  if (!route.value || !selectedTarget.value) return;
  isNavigating.value = true;
  currentStep.value = 0;
  emit("navigationStarted", selectedTarget.value.id);
}

function stopNavigation() {
  isNavigating.value = false;
  currentStep.value = 0;
  emit("navigationEnded");
}

function nextStep() {
  if (navigationSteps.value.length === 0) return;
  if (currentStep.value < navigationSteps.value.length - 1) {
    currentStep.value++;
  } else {
    stopNavigation();
  }
}

function previousStep() {
  if (currentStep.value > 0) {
    currentStep.value--;
  }
}

function clearRoute() {
  route.value = null;
  selectedTarget.value = null;
  currentStep.value = 0;
  isNavigating.value = false;
  searchQuery.value = "";
  error.value = "";
}

function formatDistance(meters: number): string {
  if (meters < 1) {
    return `${Math.round(meters * 100)} cm`;
  } else if (meters < 1000) {
    return `${Math.round(meters)} m`;
  } else {
    return `${(meters / 1000).toFixed(2)} km`;
  }
}

const unlockErrorMessage = ref("");

function unlockDoor() {
  const targetMerchant = selectedTarget.value;
  if (!targetMerchant) return;
  info("Unlocking door for", targetMerchant.name);
  unlockDevice(props.entityId, targetMerchant.id).then((res) => {
    if (res.status === "success") {
      nextStep();
    } else {
      unlockErrorMessage.value = "Failed to unlock door: " + res.message;
    }
  });
}

function getNavigationStepIcon(
  step: (typeof navigationSteps.value)[number],
): string {
  switch (step.type) {
    case "straight":
      return "mdi:arrow-up";
    case "turn":
      if (step.turn === "left") return "mdi:arrow-left";
      if (step.turn === "right") return "mdi:arrow-right";
      if (step.turn === "around") return "mdi:arrow-u-left-top";
      return "mdi:navigation";
    case "transport":
      const transportType = step.transport?.[2];
      if (transportType === "elevator") return "mdi:elevator";
      if (transportType === "stairs") return "mdi:stairs";
      if (transportType === "escalator") return "mdi:escalator";
      if (transportType === "gate") return "mdi:gate";
      if (transportType === "turnstile") return "mdi:turnstile";
      return "mdi:transit-connection-variant";
    case "unlock":
      return "mdi:lock-open-variant";
    default:
      return "mdi:navigation";
  }
}

function getNavigationStepColor(
  step: (typeof navigationSteps.value)[number],
): string {
  switch (step.type) {
    case "straight":
      return "text-blue-500";
    case "turn":
      return "text-yellow-500";
    case "transport":
      const transportType = step.transport?.[2];
      if (transportType === "elevator") return "text-purple-500";
      if (transportType === "stairs") return "text-orange-500";
      if (transportType === "escalator") return "text-green-500";
      if (transportType === "gate" || transportType === "turnstile")
        return "text-red-500";
      return "text-gray-500";
    case "unlock":
      return "text-emerald-500";
    default:
      return "text-gray-500";
  }
}

function getNavigationStepTitle(
  step: (typeof navigationSteps.value)[number],
): string {
  switch (step.type) {
    case "straight":
      return "Walk Straight";
    case "turn":
      if (step.turn === "left") return "Turn Left";
      if (step.turn === "right") return "Turn Right";
      if (step.turn === "around") return "Turn Around";
      return "Turn";
    case "transport":
      const transportType = step.transport?.[2];
      return `Take ${transportType?.charAt(0).toUpperCase()}${transportType?.slice(1)}`;
    case "unlock":
      return "Unlock Door";
    default:
      return "Navigate";
  }
}

function getNavigationStepDescription(
  step: (typeof navigationSteps.value)[number],
): string {
  switch (step.type) {
    case "straight":
      return `Walk straight for ${formatDistance(step.straight || 0)}`;
    case "turn":
      if (step.turn === "left") return "Turn left";
      if (step.turn === "right") return "Turn right";
      if (step.turn === "around") return "Turn around";
      return "Turn";
    case "transport":
      const transportType = step.transport?.[2];
      const targetArea = step.transport?.[1];
      return `Take ${transportType} to ${targetArea || "next area"}`;
    case "unlock":
      return "Unlock the door to access your destination";
    default:
      return "Continue";
  }
}

watch(
  () => props.currentLocation,
  () => {
    if (isNavigating.value) {
      stopNavigation();
    }
  },
);
</script>

<template>
  <div class="navigation-panel space-y-4 w-full">
    <!-- Target Selection -->
    <Card v-if="!route">
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Icon icon="mdi:map-marker" class="w-5 h-5" />
          Select Destination
        </CardTitle>
        <CardDescription>
          Search and select where you want to go
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="space-y-2">
          <Input
            v-model="searchQuery"
            placeholder="Search merchants, stores, or tags..."
            class="w-full"
          >
            <template #prefix>
              <Icon icon="mdi:magnify" class="w-4 h-4" />
            </template>
          </Input>

          <div class="max-h-64 overflow-y-auto space-y-2">
            <Card
              v-for="merchant in filteredMerchants"
              :key="merchant.id"
              class="p-3 cursor-pointer transition-colors"
              :class="{
                'bg-accent border-primary': selectedTarget?.id === merchant.id,
                'hover:bg-accent/50': selectedTarget?.id !== merchant.id,
              }"
              @click="selectTarget(merchant)"
            >
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-2">
                    <Icon icon="mdi:store" class="w-4 h-4 text-blue-500" />
                    <span class="font-medium">{{ merchant.name }}</span>
                  </div>
                  <div class="flex flex-wrap gap-1 mt-2">
                    <Badge
                      v-for="tag in merchant.tags"
                      :key="tag"
                      variant="secondary"
                      class="text-xs"
                    >
                      {{ tag }}
                    </Badge>
                  </div>
                </div>
                <Icon
                  v-if="selectedTarget?.id === merchant.id"
                  icon="mdi:check-circle"
                  class="w-5 h-5 text-primary flex-shrink-0"
                />
              </div>
            </Card>
          </div>
        </div>

        <Separator />

        <!-- Connectivity Options -->
        <div class="space-y-3">
          <Label class="text-sm font-medium">Route Preferences</Label>
          <div class="flex flex-col gap-2">
            <div class="flex items-center space-x-2">
              <Checkbox
                :id="`elevator-${entityId}`"
                v-model:checked="allowElevator"
              />
              <label
                :for="`elevator-${entityId}`"
                class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer flex items-center gap-2"
              >
                <Icon icon="mdi:elevator" class="w-4 h-4 text-purple-500" />
                Allow Elevators
              </label>
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                :id="`stairs-${entityId}`"
                v-model:checked="allowStairs"
              />
              <label
                :for="`stairs-${entityId}`"
                class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer flex items-center gap-2"
              >
                <Icon icon="mdi:stairs" class="w-4 h-4 text-orange-500" />
                Allow Stairs
              </label>
            </div>
            <div class="flex items-center space-x-2">
              <Checkbox
                :id="`escalator-${entityId}`"
                v-model:checked="allowEscalator"
              />
              <label
                :for="`escalator-${entityId}`"
                class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer flex items-center gap-2"
              >
                <Icon icon="mdi:escalator" class="w-4 h-4 text-green-500" />
                Allow Escalators
              </label>
            </div>
          </div>
        </div>

        <div v-if="error" class="text-sm text-red-500">
          {{ error }}
        </div>

        <Button
          class="w-full"
          :disabled="!selectedTarget || !currentLocation || loading"
          @click="calculateRoute"
        >
          <Icon
            v-if="loading"
            icon="mdi:loading"
            class="w-4 h-4 mr-2 animate-spin"
          />
          <Icon v-else icon="mdi:routes" class="w-4 h-4 mr-2" />
          {{ loading ? "Calculating..." : "Calculate Route" }}
        </Button>
      </CardContent>
    </Card>

    <!-- Route Overview -->
    <Card v-if="route && !isNavigating">
      <CardHeader>
        <CardTitle class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Icon icon="mdi:map-marker-path" class="w-5 h-5" />
            Route to {{ selectedTarget?.name }}
          </div>
          <Button variant="ghost" size="icon" @click="clearRoute">
            <Icon icon="mdi:close" class="w-5 h-5" />
          </Button>
        </CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <!-- Route Instructions Preview -->
        <div class="max-h-80 overflow-y-auto space-y-2">
          <div
            v-for="(step, idx) in navigationSteps"
            :key="idx"
            class="flex items-start gap-3 p-3 rounded-lg border bg-card"
          >
            <div class="flex-shrink-0 mt-1">
              <div
                class="w-8 h-8 rounded-full bg-accent flex items-center justify-center"
              >
                <Icon
                  :icon="getNavigationStepIcon(step)"
                  :class="['w-5 h-5', getNavigationStepColor(step)]"
                />
              </div>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <span class="text-sm font-medium capitalize">
                  {{ getNavigationStepTitle(step) }}
                </span>
                <Badge variant="outline" class="text-xs">
                  Step {{ idx + 1 }}
                </Badge>
              </div>
              <p class="text-sm text-muted-foreground mt-1">
                {{ getNavigationStepDescription(step) }}
              </p>
            </div>
          </div>
        </div>

        <Button class="w-full" @click="startNavigation">
          <Icon icon="mdi:navigation" class="w-4 h-4 mr-2" />
          Start Navigation
        </Button>
      </CardContent>
    </Card>

    <!-- Active Navigation -->
    <Card v-if="isNavigating && currentNavigationStep">
      <CardHeader>
        <CardTitle class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Icon icon="mdi:navigation" class="w-5 h-5 text-primary" />
            Navigating
          </div>
          <Button variant="ghost" size="icon" @click="stopNavigation">
            <Icon icon="mdi:stop" class="w-5 h-5" />
          </Button>
        </CardTitle>
        <CardDescription> To {{ selectedTarget?.name }} </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <!-- Progress Bar -->
        <div class="space-y-2">
          <div class="flex items-center justify-between text-sm">
            <span
              >Step {{ currentStep + 1 }} of {{ navigationSteps.length }}</span
            >
            <span class="text-muted-foreground">
              {{ formatDistance(remainingDistance) }} remaining
            </span>
          </div>
          <div class="w-full bg-secondary rounded-full h-2">
            <div
              class="bg-primary h-2 rounded-full transition-all duration-300"
              :style="{ width: `${progress}%` }"
            ></div>
          </div>
        </div>

        <Separator />

        <!-- Current Instruction -->
        <div class="space-y-4">
          <div class="flex items-center gap-4 p-4 rounded-lg bg-accent">
            <div class="flex-shrink-0">
              <div
                class="w-16 h-16 rounded-full bg-background flex items-center justify-center"
              >
                <Icon
                  :icon="getNavigationStepIcon(currentNavigationStep)"
                  :class="[
                    'w-8 h-8',
                    getNavigationStepColor(currentNavigationStep),
                  ]"
                />
              </div>
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-semibold capitalize">
                {{ getNavigationStepTitle(currentNavigationStep) }}
              </h3>
              <p class="text-sm text-muted-foreground mt-1">
                {{ getNavigationStepDescription(currentNavigationStep) }}
              </p>
              <Button
                v-if="currentNavigationStep.type === 'unlock'"
                class="mt-2"
                size="sm"
                @click="unlockDoor"
              >
                Unlock
              </Button>
              <p v-if="unlockErrorMessage" class="text-sm text-red-500 mt-1">
                {{ unlockErrorMessage }}
              </p>
            </div>
          </div>

          <!-- Next Instruction Preview -->
          <div
            v-if="currentStep < navigationSteps.length - 1"
            class="p-3 rounded-lg border bg-card/50"
          >
            <p class="text-xs text-muted-foreground mb-2">Next:</p>
            <div class="flex items-center gap-2">
              <Icon
                :icon="getNavigationStepIcon(navigationSteps[currentStep + 1])"
                class="w-4 h-4"
              />
              <span class="text-sm font-medium capitalize">
                {{ getNavigationStepTitle(navigationSteps[currentStep + 1]) }}
              </span>
            </div>
            <p class="text-xs text-muted-foreground mt-1">
              {{
                getNavigationStepDescription(navigationSteps[currentStep + 1])
              }}
            </p>
          </div>
        </div>

        <!-- Navigation Controls -->
        <div class="flex gap-2">
          <Button
            variant="outline"
            class="flex-1"
            :disabled="currentStep === 0"
            @click="previousStep"
          >
            <Icon icon="mdi:chevron-left" class="w-4 h-4 mr-1" />
            Previous
          </Button>
          <Button class="flex-1" @click="nextStep">
            {{ currentStep === navigationSteps.length - 1 ? "Finish" : "Next" }}
            <Icon icon="mdi:chevron-right" class="w-4 h-4 ml-1" />
          </Button>
        </div>
      </CardContent>
    </Card>
  </div>
</template>

<style scoped>
.navigation-panel {
  @apply w-full;
}
</style>
