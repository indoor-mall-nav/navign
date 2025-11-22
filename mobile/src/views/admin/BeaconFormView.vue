<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { createBeacon, updateBeacon, getBeacon, listAreas } from '@/lib/api/client'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Beacon } from '@/schema'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const loading = ref(false)
const error = ref<string | null>(null)
const areas = ref<any[]>([])

const entityId = computed(() => route.query.entity as string || '')
const beaconId = computed(() => parseInt(route.query.id as string) || -1)
const isEditMode = computed(() => !!beaconId.value)

// Form data
const formData = ref({
  name: '',
  description: '',
  type: 'navigation' as 'navigation' | 'marketing',
  device: 'esp32c3' as 'esp32' | 'esp32c3' | 'esp32s3' | 'esp32c6',
  area: -1,
  merchant: null as number | null,
  connection: null as number | null,
  location: [0, 0] as [number, number],
})

onMounted(async () => {
  await loadAreas()
  if (isEditMode.value) {
    await loadBeacon()
  }
})

async function loadAreas() {
  if (!entityId.value) return

  try {
    const response = await listAreas(entityId.value, session.userToken || '')
    if (response.status === 'success' && response.data) {
      areas.value = response.data
    }
  } catch (err) {
  }
}

async function loadBeacon() {
  if (!entityId.value || !beaconId.value) return

  loading.value = true
  error.value = null

  try {
    const response = await getBeacon(entityId.value, beaconId.value, session.userToken || '')
    if (response.status === 'success' && response.data) {
      const beacon = response.data
      formData.value = {
        name: beacon.name,
        description: beacon.description || '',
        type: beacon.type,
        device: beacon.device,
        area: beacon.area,
        merchant: beacon.merchant || null,
        connection: beacon.connection || null,
        location: beacon.location,
      }
    } else {
      error.value = response.message || 'Failed to load beacon'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

async function handleSubmit() {
  if (!entityId.value) {
    error.value = 'No entity selected'
    return
  }

  if (!formData.value.name || !formData.value.area) {
    error.value = 'Name and area are required'
    return
  }

  loading.value = true
  error.value = null

  try {
    if (isEditMode.value) {
      const updateData: Beacon = {
        id: beaconId.value,
        entity_id: entityId.value,
        name: formData.value.name,
        description: formData.value.description || null,
        type: formData.value.type,
        device: formData.value.device,
        area_id: formData.value.area,
        merchant_id: formData.value.merchant,
        connection_id: formData.value.connection,
        location: formData.value.location,
        mac: '',
        created_at: '',
        updated_at: ''
      }
      const response = await updateBeacon(entityId.value, updateData, session.userToken || '')
      if (response.status === 'success') {
        router.push({ name: 'admin-beacons', query: { entity: entityId.value } })
      } else {
        error.value = response.message || 'Failed to update beacon'
      }
    } else {
      const createData: Beacon = {
        id: -1,
        entity_id: entityId.value,
        name: formData.value.name,
        description: formData.value.description || null,
        type: formData.value.type,
        device: formData.value.device,
        area_id: formData.value.area,
        merchant_id: formData.value.merchant,
        connection_id: formData.value.connection,
        location: formData.value.location,
        mac: '',
        created_at: '',
        updated_at: ''
      }
      const response = await createBeacon(entityId.value, createData, session.userToken || '')
      if (response.status === 'success') {
        router.push({ name: 'admin-beacons', query: { entity: entityId.value } })
      } else {
        error.value = response.message || 'Failed to create beacon'
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function handleCancel() {
  router.push({ name: 'admin-beacons', query: { entity: entityId.value } })
}
</script>

<template>
  <div class="container mx-auto p-6 max-w-2xl">
    <div class="mb-6">
      <h1 class="text-3xl font-bold">{{ isEditMode ? 'Edit' : 'Create' }} Beacon</h1>
      <p class="text-gray-600 mt-1">{{ isEditMode ? 'Update beacon information' : 'Add a new BLE beacon' }}</p>
    </div>

    <div v-if="error" class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md text-red-800">
      {{ error }}
    </div>

    <Card>
      <CardContent class="pt-6">
        <form @submit.prevent="handleSubmit" class="space-y-4">
          <div class="space-y-2">
            <Label for="name">Name *</Label>
            <Input
              id="name"
              v-model="formData.name"
              placeholder="BEACON-0001-0001"
              required
            />
          </div>

          <div class="space-y-2">
            <Label for="description">Description</Label>
            <Input
              id="description"
              v-model="formData.description"
              placeholder="Optional description"
            />
          </div>

          <div class="space-y-2">
            <Label for="type">Type *</Label>
            <select
              id="type"
              v-model="formData.type"
              class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              required
            >
              <option value="navigation">Navigation</option>
              <option value="marketing">Marketing</option>
            </select>
          </div>

          <div class="space-y-2">
            <Label for="device">Device Type *</Label>
            <select
              id="device"
              v-model="formData.device"
              class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              required
            >
              <option value="esp32">ESP32</option>
              <option value="esp32c3">ESP32-C3</option>
              <option value="esp32s3">ESP32-S3</option>
              <option value="esp32c6">ESP32-C6</option>
            </select>
          </div>

          <div class="space-y-2">
            <Label for="area">Area *</Label>
            <select
              id="area"
              v-model="formData.area"
              class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              required
            >
              <option value="" disabled>Select an area</option>
              <option v-for="area in areas" :key="area.id" :value="area.id">
                {{ area.name }}
              </option>
            </select>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <Label for="location-x">Location X *</Label>
              <Input
                id="location-x"
                type="number"
                step="0.01"
                v-model.number="formData.location[0]"
                required
              />
            </div>
            <div class="space-y-2">
              <Label for="location-y">Location Y *</Label>
              <Input
                id="location-y"
                type="number"
                step="0.01"
                v-model.number="formData.location[1]"
                required
              />
            </div>
          </div>

          <div class="flex gap-2 pt-4">
            <Button type="submit" :disabled="loading" class="flex-1">
              {{ loading ? 'Saving...' : (isEditMode ? 'Update' : 'Create') }}
            </Button>
            <Button type="button" @click="handleCancel" variant="outline" class="flex-1">
              Cancel
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
