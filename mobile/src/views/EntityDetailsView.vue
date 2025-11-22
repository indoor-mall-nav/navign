<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useSessionStore } from "@/states/session";
import { useRouter } from "vue-router";
import { getAllAreas, getAllMerchants, getAllBeacons } from "@/lib/api/tauri";
import type { AreaDetails, MapBeacon } from "@/lib/api/tauri";
import type { Merchant } from "@/schema";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Icon } from "@iconify/vue";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";

const session = useSessionStore();
const router = useRouter();

const loading = ref(false);
const error = ref("");
const areas = ref<AreaDetails[]>([]);
const merchants = ref<Merchant[]>([]);
const beacons = ref<MapBeacon[]>([]);

const entityId = ref(session.entity?.id || "");
const entityName = computed(() => session.entity?.name || "Entity");

onMounted(async () => {
  // Check if user is authenticated
  if (!session.isAuthenticated) {
    router.push("/login");
    return;
  }

  await loadEntityDetails();
});

async function loadEntityDetails() {
  if (!entityId.value) {
    error.value = "No entity selected";
    return;
  }

  loading.value = true;
  error.value = "";

  try {
    // Fetch all areas
    const areasResponse = await getAllAreas(entityId.value);
    if (areasResponse.status === "success" && areasResponse.data) {
      areas.value = areasResponse.data;
    } else {
      throw new Error(areasResponse.message || "Failed to fetch areas");
    }

    // Fetch all merchants
    const merchantsResponse = await getAllMerchants(entityId.value);
    if (merchantsResponse.status === "success" && merchantsResponse.data) {
      merchants.value = merchantsResponse.data;
    }

    // Fetch all beacons
    const beaconsResponse = await getAllBeacons(entityId.value);
    if (beaconsResponse.status === "success" && beaconsResponse.data) {
      beacons.value = beaconsResponse.data;
    }
  } catch (err) {
    error.value =
      err instanceof Error ? err.message : "Failed to load entity details";
  } finally {
    loading.value = false;
  }
}

function navigateToArea(areaId: string) {
  // Update session area and navigate to home
  const area = areas.value.find((a) => a.id === areaId);
  if (area) {
    session.area = area;
    router.push("/");
  }
}

function getMerchantsInArea(areaId: string): Merchant[] {
  return merchants.value.filter((m) => String(m.area_id) === areaId);
}

function getBeaconsInArea(areaId: string): MapBeacon[] {
  return beacons.value.filter((b) => String(b.area) === areaId);
}
</script>

<template>
  <div class="container mx-auto p-4 max-w-7xl">
    <div class="mb-6">
      <div class="flex items-center gap-2 mb-2">
        <Button variant="ghost" size="icon" @click="router.push('/')">
          <Icon icon="mdi:arrow-left" class="h-5 w-5" />
        </Button>
        <h1 class="text-3xl font-bold">{{ entityName }}</h1>
      </div>
      <p class="text-muted-foreground">
        View all areas, merchants, and beacons within this entity
      </p>
    </div>

    <!-- Error Message -->
    <div v-if="error" class="mb-4">
      <Card class="border-destructive">
        <CardContent class="pt-6">
          <div class="flex items-center gap-2 text-destructive">
            <Icon icon="mdi:alert-circle" class="h-5 w-5" />
            <span>{{ error }}</span>
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="space-y-4">
      <Skeleton class="h-32 w-full" />
      <Skeleton class="h-32 w-full" />
      <Skeleton class="h-32 w-full" />
    </div>

    <!-- Statistics Cards -->
    <div
      v-if="!loading && !error"
      class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6"
    >
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Icon icon="mdi:vector-square" class="h-5 w-5" />
            Areas
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div class="text-3xl font-bold">{{ areas.length }}</div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Icon icon="mdi:store" class="h-5 w-5" />
            Merchants
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div class="text-3xl font-bold">{{ merchants.length }}</div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Icon icon="mdi:bluetooth" class="h-5 w-5" />
            Beacons
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div class="text-3xl font-bold">{{ beacons.length }}</div>
        </CardContent>
      </Card>
    </div>

    <!-- Areas List -->
    <div v-if="!loading && !error" class="space-y-4">
      <h2 class="text-2xl font-bold mb-4">Areas</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <Card
          v-for="area in areas"
          :key="area.id"
          class="hover:shadow-lg transition-shadow cursor-pointer"
          @click="navigateToArea(area.id)"
        >
          <CardHeader>
            <CardTitle class="flex items-center justify-between">
              <span>{{ area.name }}</span>
              <Badge v-if="area.floor_type" variant="secondary">
                {{ area.floor_type }} {{ area.floor_name }}
              </Badge>
            </CardTitle>
            <CardDescription>{{
              area.description || "No description"
            }}</CardDescription>
          </CardHeader>
          <CardContent>
            <div class="space-y-2">
              <div class="flex items-center gap-2 text-sm">
                <Icon
                  icon="mdi:identifier"
                  class="h-4 w-4 text-muted-foreground"
                />
                <span class="text-muted-foreground">Code:</span>
                <Badge variant="outline">{{ area.beacon_code }}</Badge>
              </div>

              <div class="flex items-center gap-2 text-sm">
                <Icon icon="mdi:store" class="h-4 w-4 text-muted-foreground" />
                <span class="text-muted-foreground">Merchants:</span>
                <Badge>{{ getMerchantsInArea(area.id).length }}</Badge>
              </div>

              <div class="flex items-center gap-2 text-sm">
                <Icon
                  icon="mdi:bluetooth"
                  class="h-4 w-4 text-muted-foreground"
                />
                <span class="text-muted-foreground">Beacons:</span>
                <Badge>{{ getBeaconsInArea(area.id).length }}</Badge>
              </div>

              <div class="flex items-center gap-2 text-sm">
                <Icon
                  icon="mdi:vector-polygon"
                  class="h-4 w-4 text-muted-foreground"
                />
                <span class="text-muted-foreground">Polygon Points:</span>
                <Badge>{{ area.polygon.length }}</Badge>
              </div>
            </div>

            <Button class="w-full mt-4" variant="outline">
              <Icon icon="mdi:map" class="h-4 w-4 mr-2" />
              View Area
            </Button>
          </CardContent>
        </Card>
      </div>

      <!-- Empty State -->
      <div v-if="areas.length === 0" class="text-center py-12">
        <Icon
          icon="mdi:map-marker-off"
          class="h-16 w-16 mx-auto text-muted-foreground mb-4"
        />
        <h3 class="text-xl font-semibold mb-2">No Areas Found</h3>
        <p class="text-muted-foreground">
          This entity doesn't have any areas configured yet.
        </p>
      </div>
    </div>
  </div>
</template>
