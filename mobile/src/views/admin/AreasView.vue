<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { listAreas, deleteArea } from '@/lib/api/client'
import type { Area } from '@/schema/area'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const areas = ref<Area[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const deleteDialogOpen = ref(false)
const areaToDelete = ref<Area | null>(null)

const entityId = computed(() => route.query.entity as string || session.entity?._id || '')

onMounted(async () => {
  await loadAreas()
})

async function loadAreas() {
  if (!entityId.value) {
    error.value = 'No entity selected'
    return
  }

  loading.value = true
  error.value = null

  try {
    const response = await listAreas(entityId.value, session.userToken || '')
    if (response.status === 'success' && response.data) {
      areas.value = response.data
    } else {
      error.value = response.message || 'Failed to load areas'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function navigateToCreate() {
  router.push({ name: 'admin-areas-form', query: { entity: entityId.value } })
}

function navigateToEdit(area: Area) {
  router.push({
    name: 'admin-areas-form',
    query: {
      entity: entityId.value,
      id: area._id
    }
  })
}

function openDeleteDialog(area: Area) {
  areaToDelete.value = area
  deleteDialogOpen.value = true
}

async function confirmDelete() {
  if (!areaToDelete.value || !entityId.value) return

  loading.value = true
  error.value = null

  try {
    const response = await deleteArea(
      entityId.value,
      areaToDelete.value._id,
      session.userToken || ''
    )
    if (response.status === 'success') {
      await loadAreas()
      deleteDialogOpen.value = false
      areaToDelete.value = null
    } else {
      error.value = response.message || 'Failed to delete area'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function formatFloor(floor: Area['floor']) {
  if (!floor) return 'No floor'
  return `${floor.type} ${floor.name}`
}

function getFloorBadgeVariant(floorType: string | undefined) {
  switch (floorType) {
    case 'floor':
      return 'default'
    case 'basement':
      return 'secondary'
    case 'level':
      return 'outline'
    default:
      return 'outline'
  }
}
</script>

<template>
  <div class="container mx-auto p-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-3xl font-bold">Areas</h1>
        <p class="text-gray-600 mt-1">Manage areas and zones within your entity</p>
      </div>
      <Button @click="navigateToCreate">
        Create Area
      </Button>
    </div>

    <div v-if="error" class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md text-red-800">
      {{ error }}
    </div>

    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
      <p class="mt-2 text-gray-600">Loading areas...</p>
    </div>

    <div v-else-if="areas.length === 0" class="text-center py-12">
      <p class="text-gray-600">No areas found. Create your first area to get started.</p>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <Card v-for="area in areas" :key="area._id" class="hover:shadow-lg transition-shadow">
        <CardHeader>
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <CardTitle class="text-lg">{{ area.name }}</CardTitle>
              <CardDescription v-if="area.description" class="mt-1">
                {{ area.description }}
              </CardDescription>
            </div>
            <Badge :variant="getFloorBadgeVariant(area.floor?.type)">
              {{ formatFloor(area.floor) }}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div class="space-y-2 text-sm">
            <div class="flex justify-between">
              <span class="text-gray-600">Beacon Code:</span>
              <span class="font-medium font-mono">{{ area.beacon_code }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-gray-600">Polygon Points:</span>
              <span class="font-medium">{{ area.polygon.length }}</span>
            </div>
          </div>

          <div class="flex gap-2 mt-4">
            <Button @click="navigateToEdit(area)" variant="outline" class="flex-1">
              Edit
            </Button>
            <Button
              @click="openDeleteDialog(area)"
              variant="outline"
              class="flex-1 text-red-600 hover:bg-red-50"
            >
              Delete
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>

    <Dialog v-model:open="deleteDialogOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete Area</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{{ areaToDelete?.name }}"? This action cannot be undone.
          </DialogDescription>
        </DialogHeader>
        <div class="flex justify-end gap-2 mt-4">
          <Button @click="deleteDialogOpen = false" variant="outline">
            Cancel
          </Button>
          <Button @click="confirmDelete" class="bg-red-600 hover:bg-red-700">
            Delete
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  </div>
</template>
