<script setup lang="ts">
import { ref, watch } from 'vue'
import { getAreaDetails, type AreaDetails } from '@/lib/api/tauri'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { Icon } from '@iconify/vue'
import { error as logError } from '@tauri-apps/plugin-log'

const props = defineProps<{
  open: boolean
  entityId: string
  areaId: string | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const areaDetails = ref<AreaDetails | null>(null)
const loading = ref(false)
const error = ref<string>('')

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen && props.areaId) {
      await loadAreaDetails()
    }
  },
)

async function loadAreaDetails() {
  if (!props.areaId) return

  loading.value = true
  error.value = ''

  try {
    const result = await getAreaDetails(props.entityId, props.areaId)
    if (result.status === 'success' && result.data) {
      areaDetails.value = result.data
    } else {
      error.value = result.message || 'Failed to load area details'
    }
  } catch (err) {
    error.value = `Error: ${err}`
    await logError('Failed to load area details: ' + JSON.stringify(err))
  } finally {
    loading.value = false
  }
}

function formatFloor(floor: { type: string; name: number } | null): string {
  if (!floor) return 'N/A'

  switch (floor.type) {
    case 'level':
      return `Level ${floor.name}`
    case 'floor':
      return `Floor ${floor.name}`
    case 'basement':
      return `Basement ${floor.name}`
    default:
      return `Floor ${floor.name}`
  }
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="max-w-2xl max-h-[80vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Icon icon="mdi:map-marker" class="w-6 h-6 text-primary" />
          {{ areaDetails?.name || 'Area Details' }}
        </DialogTitle>
        <DialogDescription v-if="areaDetails?.beacon_code">
          Beacon Code: {{ areaDetails.beacon_code }}
        </DialogDescription>
      </DialogHeader>

      <div v-if="loading" class="space-y-4">
        <Skeleton class="h-20 w-full" />
        <Skeleton class="h-32 w-full" />
      </div>

      <div v-else-if="error" class="text-center text-destructive py-8">
        <Icon icon="mdi:alert-circle" class="w-12 h-12 mx-auto mb-2" />
        {{ error }}
      </div>

      <div v-else-if="areaDetails" class="space-y-4">
        <!-- Description -->
        <Card v-if="areaDetails.description">
          <CardContent class="pt-6">
            <div class="flex items-start gap-3">
              <Icon
                icon="mdi:information-outline"
                class="w-5 h-5 text-muted-foreground mt-1"
              />
              <div>
                <h3 class="font-semibold mb-2">Description</h3>
                <p class="text-sm text-muted-foreground">
                  {{ areaDetails.description }}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Floor Information -->
        <Card v-if="areaDetails.floor">
          <CardContent class="pt-6">
            <div class="flex items-center gap-3">
              <Icon
                icon="mdi:floor-plan"
                class="w-5 h-5 text-muted-foreground"
              />
              <div>
                <h3 class="font-semibold mb-1">Floor</h3>
                <p class="text-sm text-muted-foreground">
                  {{ formatFloor(areaDetails.floor) }}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Area Boundary -->
        <Card>
          <CardContent class="pt-6">
            <div class="flex items-start gap-3">
              <Icon
                icon="mdi:vector-polygon"
                class="w-5 h-5 text-muted-foreground mt-1"
              />
              <div class="flex-1">
                <h3 class="font-semibold mb-2">Boundary Points</h3>
                <p class="text-sm text-muted-foreground">
                  {{ areaDetails.polygon.length }} coordinate points define this
                  area
                </p>
                <div
                  v-if="areaDetails.polygon.length > 0"
                  class="mt-2 text-xs text-muted-foreground bg-muted/30 p-2 rounded max-h-32 overflow-y-auto"
                >
                  <div
                    v-for="(point, idx) in areaDetails.polygon.slice(0, 5)"
                    :key="idx"
                  >
                    Point {{ idx + 1 }}: ({{ point[0].toFixed(2) }},
                    {{ point[1].toFixed(2) }})
                  </div>
                  <div
                    v-if="areaDetails.polygon.length > 5"
                    class="text-xs italic"
                  >
                    ... and {{ areaDetails.polygon.length - 5 }} more points
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Entity ID -->
        <Card>
          <CardContent class="pt-6">
            <div class="flex items-center gap-3">
              <Icon icon="mdi:key" class="w-5 h-5 text-muted-foreground" />
              <div class="flex-1 overflow-hidden">
                <h3 class="font-semibold mb-1">Entity ID</h3>
                <p class="text-xs text-muted-foreground font-mono truncate">
                  {{ areaDetails.entity.$oid }}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </DialogContent>
  </Dialog>
</template>
