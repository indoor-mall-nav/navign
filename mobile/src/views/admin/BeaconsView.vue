<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { listBeacons, deleteBeacon } from '@/lib/api/client'
import type { Beacon } from '@/schema/beacon'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const beacons = ref<Beacon[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const deleteDialogOpen = ref(false)
const beaconToDelete = ref<Beacon | null>(null)

const entityId = computed(() => route.query.entity as string || session.entity?._id?.$oid || '')

onMounted(async () => {
  await loadBeacons()
})

async function loadBeacons() {
  if (!entityId.value) {
    error.value = 'No entity selected'
    return
  }

  loading.value = true
  error.value = null

  try {
    const response = await listBeacons(entityId.value, session.userToken || '')
    if (response.status === 'success' && response.data) {
      beacons.value = response.data
    } else {
      error.value = response.message || 'Failed to load beacons'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function navigateToCreate() {
  router.push({ name: 'admin-beacons-form', query: { entity: entityId.value } })
}

function navigateToEdit(beacon: Beacon) {
  router.push({
    name: 'admin-beacons-form',
    query: {
      entity: entityId.value,
      id: beacon._id.$oid
    }
  })
}

function openDeleteDialog(beacon: Beacon) {
  beaconToDelete.value = beacon
  deleteDialogOpen.value = true
}

async function confirmDelete() {
  if (!beaconToDelete.value || !entityId.value) return

  loading.value = true
  error.value = null

  try {
    const response = await deleteBeacon(
      entityId.value,
      beaconToDelete.value._id.$oid,
      session.userToken || ''
    )
    if (response.status === 'success') {
      await loadBeacons()
      deleteDialogOpen.value = false
      beaconToDelete.value = null
    } else {
      error.value = response.message || 'Failed to delete beacon'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function getBeaconTypeBadgeClass(type: string) {
  return type === 'navigation' ? 'bg-blue-100 text-blue-800' : 'bg-purple-100 text-purple-800'
}
</script>

<template>
  <div class="container mx-auto p-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-3xl font-bold">Beacons</h1>
        <p class="text-gray-600 mt-1">Manage BLE beacons for indoor positioning and access control</p>
      </div>
      <Button @click="navigateToCreate">
        Create Beacon
      </Button>
    </div>

    <div v-if="error" class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md text-red-800">
      {{ error }}
    </div>

    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
      <p class="mt-2 text-gray-600">Loading beacons...</p>
    </div>

    <div v-else-if="beacons.length === 0" class="text-center py-12">
      <p class="text-gray-600">No beacons found. Create your first beacon to get started.</p>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <Card v-for="beacon in beacons" :key="beacon._id.$oid" class="hover:shadow-lg transition-shadow">
        <CardHeader>
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <CardTitle class="text-lg">{{ beacon.name }}</CardTitle>
              <CardDescription v-if="beacon.description" class="mt-1">
                {{ beacon.description }}
              </CardDescription>
            </div>
            <span
              :class="getBeaconTypeBadgeClass(beacon.type)"
              class="px-2 py-1 rounded text-xs font-medium"
            >
              {{ beacon.type }}
            </span>
          </div>
        </CardHeader>
        <CardContent>
          <div class="space-y-2 text-sm">
            <div class="flex justify-between">
              <span class="text-gray-600">Device:</span>
              <span class="font-medium">{{ beacon.device }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-gray-600">Location:</span>
              <span class="font-medium font-mono text-xs">
                [{{ beacon.location[0].toFixed(2) }}, {{ beacon.location[1].toFixed(2) }}]
              </span>
            </div>
          </div>

          <div class="flex gap-2 mt-4">
            <Button @click="navigateToEdit(beacon)" variant="outline" class="flex-1">
              Edit
            </Button>
            <Button
              @click="openDeleteDialog(beacon)"
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
          <DialogTitle>Delete Beacon</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{{ beaconToDelete?.name }}"? This action cannot be undone.
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
