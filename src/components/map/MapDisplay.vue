<script setup lang="ts">
import { computed, onMounted, ref, toRefs, watch } from "vue";
import {
  generateSvgMap,
  getMapData,
  type MapData,
  type RouteResponse,
} from "@/lib/api/tauri";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Icon } from "@iconify/vue";
import RouteOverlay from "./RouteOverlay.vue";

const props = defineProps<{
  entityId: string;
  areaId: string;
  width?: number;
  height?: number;
  userLocation?: { x: number; y: number } | null;
  route?: RouteResponse | null;
  currentStep?: number;
  targetMerchantId?: string | null;
}>();

const {
  entityId,
  areaId,
  userLocation,
  targetMerchantId,
  width,
  height,
  route,
  currentStep,
} = toRefs(props);

const emit = defineEmits<{
  beaconClick: [beaconId: string];
  merchantClick: [merchantId: string];
}>();

const mapData = ref<MapData | null>(null);
const svgContent = ref<string>("");
const loading = ref(false);
const error = ref<string>("");
const searchQuery = ref("");
const mapWidth = computed(() => width.value || 800);
const mapHeight = computed(() => height.value || 600);
const showBeacons = ref(true);
const showMerchants = ref(true);
const zoomLevel = ref(1);

async function loadMapData() {
  if (!entityId.value || !areaId.value) {
    error.value = "Entity ID and Area ID are required";
    return;
  }

  loading.value = true;
  error.value = "";

  try {
    const result = await getMapData(entityId.value, areaId.value);
    if (result.status === "success" && result.data) {
      mapData.value = result.data;
      await generateMap();
    } else {
      error.value = result.message || "Failed to load map data";
    }
  } catch (err) {
    error.value = `Error: ${err}`;
  } finally {
    loading.value = false;
  }
}

const targetMerchant = computed(() => {
  if (!targetMerchantId.value || !mapData.value) return null;
  return mapData.value.merchants.find((m) => m.id === targetMerchantId.value);
});

function addUserLocationToSvg() {
  if (!svgContent.value || !userLocation.value || !mapData.value) return;

  // Calculate bounds for scaling (same as backend)
  let min_x = Number.MAX_VALUE;
  let max_x = Number.MIN_VALUE;
  let min_y = Number.MAX_VALUE;
  let max_y = Number.MIN_VALUE;

  for (const [x, y] of mapData.value.polygon) {
    min_x = Math.min(min_x, x);
    max_x = Math.max(max_x, x);
    min_y = Math.min(min_y, y);
    max_y = Math.max(max_y, y);
  }

  const scale_x = (mapWidth.value - 20) / (max_x - min_x);
  const scale_y = (mapHeight.value - 20) / (max_y - min_y);
  const scale = Math.min(scale_x, scale_y);

  // FIXME Swap x and y to match SVG coordinate system
  const tx = (userLocation.value.x - min_x) * scale + 10;
  const ty = (userLocation.value.y - min_y) * scale + 10;

  // Create user location marker with pulsing animation
  const locationMarker = `
    <g id="user-location" class="user-location">
      <defs>
        <radialGradient id="pulseGradient">
          <stop offset="0%" style="stop-color:#4CAF50;stop-opacity:0.8"/>
          <stop offset="100%" style="stop-color:#4CAF50;stop-opacity:0"/>
        </radialGradient>
      </defs>
      <!-- Pulsing circle -->
      <circle cx="${tx}" cy="${ty}" r="20" fill="url(#pulseGradient)" opacity="0.5">
        <animate attributeName="r" from="20" to="40" dur="2s" repeatCount="indefinite"/>
        <animate attributeName="opacity" from="0.5" to="0" dur="2s" repeatCount="indefinite"/>
      </circle>
      <!-- Main marker -->
      <circle cx="${tx}" cy="${ty}" r="8" fill="#4CAF50" stroke="#fff" stroke-width="3"/>
      <circle cx="${tx}" cy="${ty}" r="4" fill="#fff"/>
      <!-- Label -->
      <text x="${tx}" y="${ty + 25}" font-size="12" font-weight="bold" text-anchor="middle" fill="#4CAF50">You are here</text>
    </g>
  `;

  // Insert before closing </svg> tag
  svgContent.value = svgContent.value.replace(
    "</svg>",
    locationMarker + "</svg>",
  );
}

function addTargetMarkerToSvg() {
  if (!svgContent.value || !targetMerchant.value || !mapData.value) return;

  // Calculate bounds for scaling (same as backend)
  let min_x = Number.MAX_VALUE;
  let max_x = Number.MIN_VALUE;
  let min_y = Number.MAX_VALUE;
  let max_y = Number.MIN_VALUE;

  for (const [x, y] of mapData.value.polygon) {
    min_x = Math.min(min_x, x);
    max_x = Math.max(max_x, x);
    min_y = Math.min(min_y, y);
    max_y = Math.max(max_y, y);
  }

  const scale_x = (mapWidth.value - 20) / (max_x - min_x);
  const scale_y = (mapHeight.value - 20) / (max_y - min_y);
  const scale = Math.min(scale_x, scale_y);

  const tx = (targetMerchant.value.location[0] - min_x) * scale + 10;
  const ty = (targetMerchant.value.location[1] - min_y) * scale + 10;

  // Create target marker with pulsing animation
  const targetMarker = `
    <g id="target-marker" class="target-marker">
      <!-- Pulsing circles -->
      <circle cx="${tx}" cy="${ty}" r="20" fill="#ef4444" opacity="0.3">
        <animate attributeName="r" from="20" to="40" dur="2s" repeatCount="indefinite"/>
        <animate attributeName="opacity" from="0.3" to="0" dur="2s" repeatCount="indefinite"/>
      </circle>
      <!-- Main marker -->
      <circle cx="${tx}" cy="${ty}" r="12" fill="#ef4444" stroke="#fff" stroke-width="3"/>
      <!-- Target icon -->
      <circle cx="${tx}" cy="${ty}" r="6" fill="#fff"/>
      <circle cx="${tx}" cy="${ty}" r="3" fill="#ef4444"/>
      <!-- Label -->
      <text x="${tx}" y="${ty + 28}" font-size="12" font-weight="bold" text-anchor="middle" fill="#ef4444">Target</text>
    </g>
  `;

  // Insert before closing </svg> tag
  svgContent.value = svgContent.value.replace(
    "</svg>",
    targetMarker + "</svg>",
  );
}

async function generateMap() {
  if (!entityId.value || !areaId.value) return;

  try {
    const result = await generateSvgMap(
      entityId.value,
      areaId.value,
      mapWidth.value,
      mapHeight.value,
    );
    if (result.status === "success" && result.svg) {
      svgContent.value = result.svg;
      // Add user location marker if available
      if (userLocation.value) {
        addUserLocationToSvg();
      }
      // Add target marker if available
      if (targetMerchantId.value) {
        addTargetMarkerToSvg();
      }
    }
  } catch (err) {
    console.error("Failed to generate SVG map:", err);
  }
}

function handleSvgClick(event: MouseEvent) {
  const target = event.target as SVGElement;
  const parentId = target.parentElement?.id;

  if (parentId?.startsWith("beacon-")) {
    const beaconId = parentId.replace("beacon-", "");
    emit("beaconClick", beaconId);
  } else if (parentId?.startsWith("merchant-")) {
    const merchantId = parentId.replace("merchant-", "");
    emit("merchantClick", merchantId);
  }
}

const filteredBeacons = computed(() => {
  if (!mapData.value || !searchQuery.value) return mapData.value?.beacons || [];
  const query = searchQuery.value.toLowerCase();
  return mapData.value.beacons.filter((b) =>
    b.name.toLowerCase().includes(query),
  );
});

const filteredMerchants = computed(() => {
  if (!mapData.value || !searchQuery.value)
    return mapData.value?.merchants || [];
  const query = searchQuery.value.toLowerCase();
  return mapData.value.merchants.filter(
    (m) =>
      m.name.toLowerCase().includes(query) ||
      m.tags.some((tag) => tag.toLowerCase().includes(query)),
  );
});

function zoomIn() {
  zoomLevel.value = Math.min(zoomLevel.value + 0.1, 3);
}

function zoomOut() {
  zoomLevel.value = Math.max(zoomLevel.value - 0.1, 0.5);
}

function resetZoom() {
  zoomLevel.value = 1;
}

onMounted(() => {
  loadMapData();
});

watch(entityId, loadMapData, { immediate: true });
watch(areaId, loadMapData, { immediate: true });
watch(userLocation, loadMapData, { immediate: true });
watch(targetMerchantId, loadMapData, { immediate: true });

watch([mapWidth, mapHeight], () => {
  if (mapData.value) {
    generateMap();
  }
});
</script>

<template>
  <div class="map-display-container">
    <Card class="w-full">
      <CardHeader>
        <CardTitle class="flex items-center justify-between">
          <span>{{ mapData?.name || "Map View" }}</span>
          <div class="flex gap-2">
            <Button variant="outline" size="icon" @click="zoomOut">
              <Icon icon="mdi:magnify-minus" class="w-5 h-5" />
            </Button>
            <Button variant="outline" size="icon" @click="resetZoom">
              <Icon icon="mdi:magnify" class="w-5 h-5" />
            </Button>
            <Button variant="outline" size="icon" @click="zoomIn">
              <Icon icon="mdi:magnify-plus" class="w-5 h-5" />
            </Button>
            <Button variant="outline" size="icon" @click="loadMapData">
              <Icon icon="mdi:refresh" class="w-5 h-5" />
            </Button>
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div class="mb-4 flex gap-2">
          <Input
            v-model="searchQuery"
            placeholder="Search beacons or merchants..."
            class="flex-1"
          />
          <Button
            variant="outline"
            size="sm"
            @click="showBeacons = !showBeacons"
            :class="{ 'bg-accent': showBeacons }"
          >
            <Icon icon="mdi:access-point" class="w-4 h-4 mr-1" />
            Beacons
          </Button>
          <Button
            variant="outline"
            size="sm"
            @click="showMerchants = !showMerchants"
            :class="{ 'bg-accent': showMerchants }"
          >
            <Icon icon="mdi:store" class="w-4 h-4 mr-1" />
            Merchants
          </Button>
        </div>

        <div v-if="loading" class="space-y-2">
          <Skeleton class="h-[400px] w-full" />
        </div>

        <div v-else-if="error" class="text-center text-red-500 py-8">
          {{ error }}
        </div>

        <div
          v-else-if="svgContent"
          class="map-svg-container overflow-auto border rounded-lg relative"
          :style="{
            maxHeight: mapHeight + 'px',
            cursor: 'pointer',
          }"
        >
          <div
            v-html="svgContent"
            @click="handleSvgClick"
            :style="{
              transform: `scale(${zoomLevel})`,
              transformOrigin: 'top left',
              transition: 'transform 0.2s',
            }"
          ></div>

          <!-- Route Overlay -->
          <RouteOverlay
            v-if="route && mapData"
            :route="route"
            :map-data="mapData"
            :map-width="mapWidth"
            :map-height="mapHeight"
            :current-step="currentStep"
            :show-target="true"
            :style="{
              transform: `scale(${zoomLevel})`,
              transformOrigin: 'top left',
              transition: 'transform 0.2s',
            }"
          />
        </div>

        <div
          v-if="
            mapData &&
            (filteredBeacons.length > 0 || filteredMerchants.length > 0)
          "
          class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4"
        >
          <div v-if="showBeacons && filteredBeacons.length > 0">
            <h3 class="font-semibold mb-2">
              Beacons ({{ filteredBeacons.length }})
            </h3>
            <div class="space-y-2 max-h-40 overflow-y-auto">
              <Card
                v-for="beacon in filteredBeacons"
                :key="beacon.id"
                class="p-2 cursor-pointer hover:bg-accent"
                @click="emit('beaconClick', beacon.id)"
              >
                <div class="flex items-center gap-2">
                  <Icon icon="mdi:access-point" class="w-4 h-4 text-red-500" />
                  <span class="text-sm">{{ beacon.name }}</span>
                </div>
              </Card>
            </div>
          </div>

          <div v-if="showMerchants && filteredMerchants.length > 0">
            <h3 class="font-semibold mb-2">
              Merchants ({{ filteredMerchants.length }})
            </h3>
            <div class="space-y-2 max-h-40 overflow-y-auto">
              <Card
                v-for="merchant in filteredMerchants"
                :key="merchant.id"
                class="p-2 cursor-pointer hover:bg-accent"
                @click="emit('merchantClick', merchant.id)"
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <Icon icon="mdi:store" class="w-4 h-4 text-blue-500" />
                    <span class="text-sm">{{ merchant.name }}</span>
                  </div>
                  <div class="flex gap-1">
                    <span
                      v-for="tag in merchant.tags.slice(0, 2)"
                      :key="tag"
                      class="text-xs bg-muted px-2 py-0.5 rounded"
                    >
                      {{ tag }}
                    </span>
                  </div>
                </div>
              </Card>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
</template>

<style scoped>
.map-svg-container {
  background: #f9fafb;
}
</style>
