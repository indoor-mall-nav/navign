<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute } from "vue-router";
import MapDisplay from "@/components/map/MapDisplay.vue";
import NavigationPanel from "@/components/map/NavigationPanel.vue";
import {
  getAllMerchants,
  getMapData,
  locateDevice,
  type RouteResponse,
} from "@/lib/api/tauri";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Icon } from "@iconify/vue";
import { Separator } from "@/components/ui/separator";
import { info, error as logError } from "@tauri-apps/plugin-log";

const route = useRoute();

// Entity and area from route params or defaults
const entityId = ref((route.query.entity as string) || "default-entity");
const areaId = ref((route.query.area as string) || "default-area");

// Navigation state
const currentRoute = ref<RouteResponse | null>(null);
const currentStep = ref<number>(0);
const targetMerchantId = ref<string | null>(null);
const userLocation = ref<{ x: number; y: number } | null>(null);
const currentLocationId = ref<string | undefined>(undefined);

// Map data for navigation panel
const mapData = ref<any>(null);
const isLocating = ref(false);
const locationError = ref("");
const merchantsData = ref<Array<any>>([]);

// Layout state
const showNavigationPanel = ref(true);

async function loadMapData() {
  try {
    const result = await getMapData(entityId.value, areaId.value);
    if (result.status === "success" && result.data) {
      mapData.value = result.data;
    }
    const merchants = await getAllMerchants(entityId.value);
    if (merchants.status === "success" && merchants.data) {
      merchantsData.value = merchants.data.map((x) => ({
        ...x,
        id: x._id.$oid,
      }));
    }
  } catch (err) {
    await logError("Failed to load map data: " + JSON.stringify(err));
  }
}

async function locateUserPosition() {
  isLocating.value = true;
  locationError.value = "";

  try {
    await info(`Locating device in area ${areaId.value}...`);
    const result = await locateDevice(areaId.value, entityId.value);
    if (result.status === "success" && result.x && result.y) {
      userLocation.value = { x: result.x, y: result.y };
      if (result.area && result.area !== areaId.value) {
        areaId.value = result.area;
        await loadMapData();
      }
      currentLocationId.value = result.area;
    } else {
      locationError.value = result.message || "Failed to locate position";
    }
  } catch (err) {
    locationError.value = `Error: ${err}`;
  } finally {
    isLocating.value = false;
  }
}

function handleRouteCalculated(route: RouteResponse) {
  currentRoute.value = route;
  currentStep.value = 0;
}

function handleNavigationStarted(targetId: string) {
  targetMerchantId.value = targetId;
  currentStep.value = 0;
}

function handleNavigationEnded() {
  currentRoute.value = null;
  targetMerchantId.value = null;
  currentStep.value = 0;
}

function toggleNavigationPanel() {
  showNavigationPanel.value = !showNavigationPanel.value;
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

onMounted(() => {
  loadMapData();
});
</script>

<template>
  <div class="navigation-view h-screen flex flex-col">
    <!-- Header -->
    <div class="bg-background border-b px-4 py-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <Button variant="ghost" size="icon" @click="$router.back()">
            <Icon icon="mdi:arrow-left" class="w-5 h-5" />
          </Button>
          <div>
            <h1 class="text-xl font-bold">Indoor Navigation</h1>
            <p class="text-sm text-muted-foreground">Find your way around</p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            @click="locateUserPosition"
            :disabled="isLocating"
          >
            <Icon
              :icon="isLocating ? 'mdi:loading' : 'mdi:crosshairs-gps'"
              :class="['w-4 h-4 mr-2', { 'animate-spin': isLocating }]"
            />
            {{ isLocating ? "Locating..." : "Locate Me" }}
          </Button>
          <Button
            variant="outline"
            size="icon"
            @click="toggleNavigationPanel"
            class="lg:hidden"
          >
            <Icon
              :icon="showNavigationPanel ? 'mdi:close' : 'mdi:menu'"
              class="w-5 h-5"
            />
          </Button>
        </div>
      </div>

      <!-- Location Error -->
      <div
        v-if="locationError"
        class="mt-2 p-2 bg-destructive/10 text-destructive text-sm rounded-md flex items-center gap-2"
      >
        <Icon icon="mdi:alert-circle" class="w-4 h-4" />
        {{ locationError }}
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Map Display Area -->
      <div class="flex-1 overflow-auto p-4 bg-muted/30">
        <div class="max-w-7xl mx-auto">
          <MapDisplay
            :entity-id="entityId"
            :area-id="areaId"
            :width="1200"
            :height="800"
            :user-location="userLocation"
            :route="currentRoute"
            :current-step="currentStep"
            :target-merchant-id="targetMerchantId"
          />

          <!-- Quick Stats (below map on mobile) -->
          <div
            v-if="currentRoute"
            class="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4 lg:hidden"
          >
            <Card>
              <CardContent class="pt-6">
                <div class="text-center">
                  <Icon
                    icon="mdi:map-marker-distance"
                    class="w-8 h-8 mx-auto mb-2 text-primary"
                  />
                  <p class="text-2xl font-bold">
                    {{ formatDistance(currentRoute.total_distance) }}
                  </p>
                  <p class="text-xs text-muted-foreground">Total Distance</p>
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent class="pt-6">
                <div class="text-center">
                  <Icon
                    icon="mdi:foot-print"
                    class="w-8 h-8 mx-auto mb-2 text-blue-500"
                  />
                  <p class="text-2xl font-bold">
                    {{ currentRoute.instructions.length }}
                  </p>
                  <p class="text-xs text-muted-foreground">Steps</p>
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent class="pt-6">
                <div class="text-center">
                  <Icon
                    icon="mdi:layers"
                    class="w-8 h-8 mx-auto mb-2 text-green-500"
                  />
                  <!--                  <p class="text-2xl font-bold">{{ currentRoute.areas.length }}</p>-->
                  <p class="text-xs text-muted-foreground">Areas</p>
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent class="pt-6">
                <div class="text-center">
                  <Icon
                    icon="mdi:progress-check"
                    class="w-8 h-8 mx-auto mb-2 text-orange-500"
                  />
                  <p class="text-2xl font-bold">
                    {{ currentStep + 1 }}/{{ currentRoute.instructions.length }}
                  </p>
                  <p class="text-xs text-muted-foreground">Progress</p>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>

      <!-- Navigation Panel Sidebar -->
      <div
        v-if="showNavigationPanel"
        class="w-full lg:w-96 border-l bg-background overflow-y-auto absolute lg:relative inset-0 lg:inset-auto z-50 lg:z-auto"
      >
        <div class="sticky top-0 bg-background z-10 border-b p-4 lg:hidden">
          <div class="flex items-center justify-between">
            <h2 class="font-semibold">Navigation</h2>
            <Button variant="ghost" size="icon" @click="toggleNavigationPanel">
              <Icon icon="mdi:close" class="w-5 h-5" />
            </Button>
          </div>
        </div>

        <div class="p-4 space-y-4">
          <!-- Quick Stats (sidebar on desktop) -->
          <div
            v-if="currentRoute"
            class="hidden lg:grid grid-cols-2 gap-2 mb-4"
          >
            <Card class="bg-primary/5">
              <CardContent class="pt-4 pb-3">
                <div class="text-center">
                  <p class="text-xl font-bold">
                    {{ formatDistance(currentRoute.total_distance) }}
                  </p>
                  <p class="text-xs text-muted-foreground">Distance</p>
                </div>
              </CardContent>
            </Card>
            <Card class="bg-blue-500/5">
              <CardContent class="pt-4 pb-3">
                <div class="text-center">
                  <p class="text-xl font-bold">
                    {{ currentRoute.instructions.length }}
                  </p>
                  <p class="text-xs text-muted-foreground">Steps</p>
                </div>
              </CardContent>
            </Card>
          </div>

          <Separator v-if="currentRoute" class="hidden lg:block" />

          <!-- Navigation Panel Component -->
          <NavigationPanel
            v-if="mapData"
            :entity-id="entityId"
            :current-location="currentLocationId"
            :current-exact-location="
              userLocation ? [userLocation?.x, userLocation?.y] : undefined
            "
            :merchants="merchantsData || []"
            @route-calculated="handleRouteCalculated"
            @navigation-started="handleNavigationStarted"
            @navigation-ended="handleNavigationEnded"
          />

          <!-- Help Card -->
          <Card v-if="!currentRoute" class="bg-muted/50">
            <CardHeader>
              <CardTitle class="text-sm flex items-center gap-2">
                <Icon icon="mdi:information" class="w-4 h-4" />
                How to Navigate
              </CardTitle>
            </CardHeader>
            <CardContent class="text-sm space-y-2">
              <ol
                class="list-decimal list-inside space-y-1 text-muted-foreground"
              >
                <li>Search for your destination</li>
                <li>Select route preferences</li>
                <li>Calculate the route</li>
                <li>Start navigation for turn-by-turn directions</li>
              </ol>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>

    <!-- Floating Action Button (mobile only when panel is hidden) -->
    <Button
      v-if="!showNavigationPanel"
      class="fixed bottom-6 right-6 h-14 w-14 rounded-full shadow-lg lg:hidden z-40"
      size="icon"
      @click="toggleNavigationPanel"
    >
      <Icon icon="mdi:navigation" class="w-6 h-6" />
    </Button>
  </div>
</template>

<style scoped>
@media (max-width: 1024px) {
  .navigation-view {
    @apply relative;
  }
}
</style>
