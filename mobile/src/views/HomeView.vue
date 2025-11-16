<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useSessionStore } from '@/states/session'
import { useRouter } from 'vue-router'
import { locateDevice } from '@/lib/api/tauri'
import MapDisplay from '@/components/map/MapDisplay.vue'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Icon } from '@iconify/vue'
import { Badge } from '@/components/ui/badge'
import { fetch } from '@tauri-apps/plugin-http'
import {
  checkPermissions,
  getCurrentPosition,
  requestPermissions,
} from '@tauri-apps/plugin-geolocation'
import { baseUrl } from '@/lib/shared'
import type { Area, Entity } from '@/schema'
import { error as logError, info } from '@tauri-apps/plugin-log'

const session = useSessionStore()
const router = useRouter()

const locating = ref(false)
const locationError = ref('')
const currentPosition = ref<{ area: string; x: number; y: number } | null>(null)

// Entity finding state
const geolocation = ref<[number, number]>([0, 0])
const entityString = ref('')
const entities = ref<Entity[]>([])
const findingEntity = ref(false)
const geolocationError = ref('')

// Area finding state (using backend locate API)
const findingArea = ref(false)
const areaFindingError = ref('')

const entityId = ref(session.entity?.id || '')
const areaId = ref(session.area?.id || '')

session.$subscribe((mutation, state) => {
  info(
    `Session changed: ${mutation.storeId}, ` +
      `Mutation: ${mutation.type}, New Entity: ${
        state.entity ? state.entity.name : 'None'
      }, New Area: ${state.area ? state.area.name : 'None'}`,
  )
  areaId.value = state.area?.id || ''
  entityId.value = state.entity?.id || ''
})

onMounted(() => {
  // Check if user is authenticated
  if (!session.isAuthenticated) {
    router.push('/login')
  }
  info('HomeView mounted, session:' + JSON.stringify(session.$state))
})

// Get geolocation
async function getGeolocation() {
  geolocationError.value = ''

  try {
    await info('Checking permissions...')
    const permission = await checkPermissions()
    await info('Current permissions: ' + JSON.stringify(permission))

    if (permission.location !== 'granted') {
      await info('Requesting permissions...')
      const request = await requestPermissions(['location', 'coarseLocation'])
      await info('Permission request result: ' + JSON.stringify(request))

      if (request.location !== 'granted') {
        geolocationError.value = 'Location permission is required to proceed.'
        return
      }
    }

    await info('Requesting geolocation...')
    const position = await getCurrentPosition()
    await info('Position obtained: ' + JSON.stringify(position))
    geolocation.value = [position.coords.latitude, position.coords.longitude]
    geolocationError.value = ''
  } catch (error) {
    await logError('Geolocation error: ' + JSON.stringify(error))
    geolocationError.value =
      'Failed to obtain geolocation. Please ensure location services are enabled.'
  }
}

// Find entities based on location and name
async function findEntity() {
  findingEntity.value = true
  geolocationError.value = ''

  try {
    const query = new URLSearchParams({
      country: 'China',
      region: 'Zhejiang',
      city: 'Ningbo',
      entity: entityString.value || '',
    })

    // Add geolocation if available
    if (
      geolocation.value &&
      geolocation.value[0] !== 0.0 &&
      geolocation.value[1] !== 0.0
    ) {
      query.append('latitude', geolocation.value[0].toString())
      query.append('longitude', geolocation.value[1].toString())
    }

    await info(
      'Fetching entities: ' + baseUrl + '/api/entities/?' + query.toString(),
    )

    const response = await fetch(
      baseUrl + '/api/entities/?' + query.toString(),
      {
        method: 'GET',
      },
    )

    const data: Entity[] = await response.json()
    await info('Entities found: ' + JSON.stringify(data))

    entities.value = data

    if (data.length === 1) {
      // Auto-select if only one entity found
      session.setEntity(data[0])
      geolocationError.value = `Found: ${data[0].name}`
    } else if (data.length > 1) {
      geolocationError.value = `Found ${data.length} entities. Please select one below.`
    } else {
      geolocationError.value = 'No entities found matching your criteria.'
    }
  } catch (error) {
    await logError('Error finding entity: ' + JSON.stringify(error))
    geolocationError.value = `Error: ${error}`
  } finally {
    findingEntity.value = false
  }
}

// Select an entity
function selectEntity(entity: Entity) {
  session.setEntity(entity)
  geolocationError.value = `Selected: ${entity.name}`
  entities.value = [] // Clear the list after selection
}

// Find area using backend BLE scanning and location API
async function findAreaWithBackend() {
  if (!entityId.value) {
    areaFindingError.value = 'Please select an entity first'
    return
  }

  findingArea.value = true
  areaFindingError.value = ''

  try {
    // Call backend locate_device which handles BLE scanning internally
    // For initial area detection, we pass empty area string
    await info('Calling backend locate API for entity: ' + entityId.value)

    const result = await locateDevice('', entityId.value)

    if (result.status === 'success') {
      // Backend returns the detected area
      areaFindingError.value = `Found area: ${result.area}`

      // Fetch the full area details
      try {
        const areaResponse = await fetch(
          `${baseUrl}/api/entities/${entityId.value}/areas/${result.area}`,
        )

        const area: Area = await areaResponse.json()
        session.setArea(area)

        // Set initial position if provided
        if (result.x !== undefined && result.y !== undefined) {
          currentPosition.value = {
            area: result.area,
            x: result.x,
            y: result.y,
          }
          session.setCurrentLocation({ x: result.x, y: result.y })
        }

        areaFindingError.value = `Successfully detected: ${area.name}`
      } catch (error) {
        await logError('Error fetching area details: ' + JSON.stringify(error))
        areaFindingError.value = 'Area detected but failed to load details'
      }
    } else {
      areaFindingError.value =
        result.message || 'Failed to detect area. No beacons found nearby.'
    }
  } catch (error) {
    await logError('Area detection error: ' + JSON.stringify(error))
    areaFindingError.value = `Error: ${error}`
  } finally {
    findingArea.value = false
  }
}

async function handleLocateMe() {
  if (!entityId.value || !areaId.value) {
    locationError.value = 'Please select an entity and area first'
    return
  }

  locating.value = true
  locationError.value = ''

  try {
    const result = await locateDevice(areaId.value, entityId.value)
    if (result.status === 'success') {
      currentPosition.value = {
        area: result.area || areaId.value,
        x: result.x || 0,
        y: result.y || 0,
      }
      session.setCurrentLocation({ x: result.x || 0, y: result.y || 0 })
      locationError.value = `Located at (${result.x?.toFixed(2)}, ${result.y?.toFixed(2)})`
      if (result.area && result.area !== areaId.value) {
        // Area has changed
        try {
          const areaResponse = await fetch(
            `${baseUrl}/api/entities/${entityId.value}/areas/${result.area}`,
          )
          const area: Area = await areaResponse.json()
          session.setArea(area)
          locationError.value += `, Area changed to ${area.name}`
        } catch (error) {
          await logError(
            'Error fetching new area details: ' + JSON.stringify(error),
          )
          locationError.value += ', but failed to load new area details'
        }
      }
    } else {
      locationError.value = result.message || 'Failed to locate device'
    }
  } catch (error) {
    locationError.value = `Error: ${error}`
  } finally {
    locating.value = false
  }
}

async function handleBeaconClick(beaconId: string) {
  await info('Beacon clicked:' + beaconId)
}

async function handleMerchantClick(merchantId: string) {
  await info('Merchant clicked: ' + merchantId)
}

function handleStartNavigation() {
  if (!entityId.value || !areaId.value) {
    locationError.value = 'Please select an entity and area first'
    return
  }

  router.push({
    name: 'navigation',
    query: {
      entity: entityId.value,
      area: areaId.value,
    },
  })
}

function handleLogout() {
  session.clearSession()
  router.push('/login')
}

// Watch for entity changes
watch(
  () => session.entity,
  (newEntity) => {
    if (newEntity) {
      info('Entity selected: ' + newEntity.name)
    }
  },
)
</script>

<template>
  <div class="container mx-auto p-4">
    <div class="flex justify-between items-center mb-6">
      <div>
        <h1 class="text-3xl font-bold">Indoor Navigation</h1>
        <p class="text-muted-foreground" v-if="session.entity?.name">
          {{ session.entity.name }}
        </p>
      </div>
      <Button variant="outline" @click="handleLogout">
        <Icon icon="mdi:logout" class="w-4 h-4 mr-2" />
        Logout
      </Button>
    </div>

    <!-- Entity Finding Section -->
    <div v-if="!session.isEntitySet" class="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Find Your Location</CardTitle>
          <CardDescription>
            Use your device's location or search for a mall/building name
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <!-- Geolocation -->
          <div class="space-y-2">
            <Button
              @click="getGeolocation"
              variant="outline"
              class="w-full justify-start"
              :disabled="geolocation[0] !== 0"
            >
              <Icon
                icon="mdi:crosshairs-gps"
                class="w-4 h-4 mr-2"
                :class="{ 'text-green-500': geolocation[0] !== 0 }"
              />
              {{
                geolocation[0] !== 0
                  ? `Location: ${geolocation[0].toFixed(4)}, ${geolocation[1].toFixed(4)}`
                  : 'Get Current Location'
              }}
            </Button>
          </div>

          <!-- Search Input -->
          <div class="space-y-2">
            <Input
              v-model="entityString"
              placeholder="Enter mall or building name (e.g., Ningbo Mall)"
              @keyup.enter="findEntity"
            />
          </div>

          <!-- Find Button -->
          <Button @click="findEntity" class="w-full" :disabled="findingEntity">
            <Icon
              icon="mdi:magnify"
              class="w-4 h-4 mr-2"
              :class="{ 'animate-spin': findingEntity }"
            />
            {{ findingEntity ? 'Searching...' : 'Find Entity' }}
          </Button>

          <!-- Error/Success Messages -->
          <p
            v-if="geolocationError"
            class="text-sm"
            :class="
              geolocationError.includes('Found')
                ? 'text-green-600'
                : 'text-red-500'
            "
          >
            {{ geolocationError }}
          </p>
        </CardContent>
      </Card>

      <!-- Entity Selection List -->
      <Card v-if="entities.length > 1">
        <CardHeader>
          <CardTitle>Select Location</CardTitle>
          <CardDescription>
            Multiple locations found. Please select one:
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-2">
          <Card
            v-for="entity in entities"
            :key="entity.id"
            class="p-4 cursor-pointer hover:bg-accent transition-colors"
            @click="selectEntity(entity)"
          >
            <div class="flex items-start justify-between">
              <div>
                <h3 class="font-semibold">{{ entity.name }}</h3>
                <p
                  class="text-sm text-muted-foreground"
                  v-if="entity.description"
                >
                  {{ entity.description }}
                </p>
                <div class="flex gap-1 mt-2">
                  <Badge
                    v-for="tag in entity.tags.slice(0, 3)"
                    :key="tag"
                    variant="secondary"
                  >
                    {{ tag }}
                  </Badge>
                </div>
              </div>
              <Icon
                icon="mdi:chevron-right"
                class="w-5 h-5 text-muted-foreground"
              />
            </div>
          </Card>
        </CardContent>
      </Card>

      <!-- Area Detection using Backend API -->
      <Card v-if="session.isEntitySet && !session.isAreaSet">
        <CardHeader>
          <CardTitle>Detect Your Area</CardTitle>
          <CardDescription>
            Use BLE beacons to automatically detect your current area
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="p-4 bg-muted rounded-lg text-sm">
            <p class="font-medium mb-2">How it works:</p>
            <ul class="list-disc list-inside space-y-1 text-muted-foreground">
              <li>Scans for nearby BLE beacons</li>
              <li>Identifies your current area automatically</li>
              <li>Works on both mobile and desktop (with mock data)</li>
            </ul>
          </div>

          <Button
            @click="findAreaWithBackend"
            class="w-full"
            :disabled="findingArea"
          >
            <Icon
              icon="mdi:radar"
              class="w-4 h-4 mr-2"
              :class="{ 'animate-spin': findingArea }"
            />
            {{ findingArea ? 'Detecting Area...' : 'Detect Current Area' }}
          </Button>

          <p
            v-if="areaFindingError"
            class="text-sm"
            :class="
              areaFindingError.includes('Successfully') ||
              areaFindingError.includes('Found')
                ? 'text-green-600'
                : 'text-red-500'
            "
          >
            {{ areaFindingError }}
          </p>
        </CardContent>
      </Card>
    </div>

    <!-- Main Navigation View (when entity and area are selected) -->
    <div v-else class="grid grid-cols-1 lg:grid-cols-3 gap-4">
      <!-- Left sidebar - Location info -->
      <div class="lg:col-span-1 space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>Current Location</CardTitle>
            <CardDescription v-if="session.area?.name">
              {{ session.area.name }}
              ID: {{ session.area.id }}
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-4">
            <div v-if="currentPosition" class="p-3 bg-accent rounded-lg">
              <div class="flex items-center gap-2 mb-2">
                <Icon
                  icon="mdi:map-marker-check"
                  class="w-5 h-5 text-green-500"
                />
                <span class="font-semibold">Located</span>
              </div>
              <div class="text-sm text-muted-foreground">
                Position: ({{ currentPosition.x.toFixed(2) }},
                {{ currentPosition.y.toFixed(2) }})
              </div>
            </div>

            <Button class="w-full" @click="handleLocateMe" :disabled="locating">
              <Icon
                icon="mdi:crosshairs-gps"
                class="w-4 h-4 mr-2"
                :class="{ 'animate-spin': locating }"
              />
              {{ locating ? 'Locating...' : 'Locate Me' }}
            </Button>

            <p
              v-if="locationError"
              :class="
                locationError.includes('Located')
                  ? 'text-green-600 text-sm'
                  : 'text-red-500 text-sm'
              "
            >
              {{ locationError }}
            </p>
          </CardContent>
        </Card>

        <Card v-if="session.nearestMerchants.length > 0">
          <CardHeader>
            <CardTitle>Nearby Merchants</CardTitle>
          </CardHeader>
          <CardContent class="space-y-2">
            <div
              v-for="merchant in session.nearestMerchants.slice(0, 5)"
              :key="merchant.id"
              class="p-2 border rounded-lg hover:bg-accent cursor-pointer transition-colors"
              @click="handleMerchantClick(merchant.id)"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <Icon icon="mdi:store" class="w-4 h-4 text-blue-500" />
                  <span class="text-sm font-medium">{{ merchant.name }}</span>
                </div>
                <Icon
                  icon="mdi:chevron-right"
                  class="w-4 h-4 text-muted-foreground"
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
          </CardHeader>
          <CardContent class="space-y-2">
            <Button
              variant="outline"
              class="w-full justify-start"
              @click="handleStartNavigation"
            >
              <Icon icon="mdi:navigation" class="w-4 h-4 mr-2" />
              Start Navigation
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              @click="router.push('/entity-details')"
            >
              <Icon icon="mdi:office-building" class="w-4 h-4 mr-2" />
              View All Areas
            </Button>
            <Button variant="outline" class="w-full justify-start">
              <Icon icon="mdi:magnify" class="w-4 h-4 mr-2" />
              Search Merchants
            </Button>
            <Button
              variant="outline"
              class="w-full justify-start"
              @click="handleLocateMe"
            >
              <Icon icon="mdi:target" class="w-4 h-4 mr-2" />
              Update Location
            </Button>
          </CardContent>
        </Card>
      </div>

      <!-- Main content - Map -->
      <div class="lg:col-span-2">
        <MapDisplay
          v-if="entityId && areaId"
          :entity-id="entityId"
          :area-id="areaId"
          :width="800"
          :height="600"
          :user-location="
            currentPosition
              ? { x: currentPosition.x, y: currentPosition.y }
              : null
          "
          @beacon-click="handleBeaconClick"
          @merchant-click="handleMerchantClick"
        />
        <Card v-else>
          <CardContent class="py-12 text-center">
            <Icon
              icon="mdi:map-off"
              class="w-16 h-16 mx-auto mb-4 text-muted-foreground"
            />
            <p class="text-muted-foreground">No map data available</p>
            <Button class="mt-4" @click="findAreaWithBackend">
              <Icon icon="mdi:radar" class="w-4 h-4 mr-2" />
              Detect Area
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1s linear infinite;
}
</style>
