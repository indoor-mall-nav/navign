<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { createArea, updateArea, getArea } from '@/lib/api/client'
import type { AreaCreateRequest, AreaUpdateRequest } from '@/lib/api/client'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const loading = ref(false)
const error = ref<string | null>(null)

const entityId = computed(() => route.query.entity as string || '')
const areaId = computed(() => route.query.id as string || '')
const isEditMode = computed(() => !!areaId.value)

// Form data
const formData = ref({
  name: '',
  description: '',
  beacon_code: '',
  floor: null as { type: 'level' | 'floor' | 'basement', name: number } | null,
  polygon: [] as [number, number][],
  polygonInput: '', // JSON string input for polygon
})

onMounted(async () => {
  if (isEditMode.value) {
    await loadArea()
  }
})

async function loadArea() {
  if (!entityId.value || !areaId.value) return

  loading.value = true
  error.value = null

  try {
    const response = await getArea(entityId.value, areaId.value, session.userToken || '')
    if (response.status === 'success' && response.data) {
      const area = response.data
      formData.value = {
        name: area.name,
        description: area.description || '',
        beacon_code: area.beacon_code,
        floor: area.floor,
        polygon: area.polygon,
        polygonInput: JSON.stringify(area.polygon),
      }
    } else {
      error.value = response.message || 'Failed to load area'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function parsePolygon() {
  try {
    const parsed = JSON.parse(formData.value.polygonInput)
    if (Array.isArray(parsed) && parsed.every(p => Array.isArray(p) && p.length === 2)) {
      formData.value.polygon = parsed
      error.value = null
    } else {
      error.value = 'Invalid polygon format. Expected: [[x1, y1], [x2, y2], ...]'
    }
  } catch (err) {
    error.value = 'Invalid JSON format for polygon'
  }
}

async function handleSubmit() {
  if (!entityId.value) {
    error.value = 'No entity selected'
    return
  }

  if (!formData.value.name || !formData.value.beacon_code) {
    error.value = 'Name and beacon code are required'
    return
  }

  // Parse polygon before submitting
  if (formData.value.polygonInput) {
    parsePolygon()
    if (error.value) return
  }

  if (formData.value.polygon.length < 3) {
    error.value = 'Polygon must have at least 3 points'
    return
  }

  loading.value = true
  error.value = null

  try {
    if (isEditMode.value) {
      const updateData: AreaUpdateRequest = {
        _id: areaId.value,
        name: formData.value.name,
        description: formData.value.description || null,
        beacon_code: formData.value.beacon_code,
        floor: formData.value.floor,
        polygon: formData.value.polygon,
      }
      const response = await updateArea(entityId.value, updateData, session.userToken || '')
      if (response.status === 'success') {
        router.push({ name: 'admin-areas', query: { entity: entityId.value } })
      } else {
        error.value = response.message || 'Failed to update area'
      }
    } else {
      const createData: AreaCreateRequest = {
        entity: entityId.value,
        name: formData.value.name,
        description: formData.value.description || null,
        beacon_code: formData.value.beacon_code,
        floor: formData.value.floor,
        polygon: formData.value.polygon,
      }
      const response = await createArea(entityId.value, createData, session.userToken || '')
      if (response.status === 'success') {
        router.push({ name: 'admin-areas', query: { entity: entityId.value } })
      } else {
        error.value = response.message || 'Failed to create area'
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function handleCancel() {
  router.push({ name: 'admin-areas', query: { entity: entityId.value } })
}

function addFloor() {
  formData.value.floor = { type: 'floor', name: 1 }
}

function removeFloor() {
  formData.value.floor = null
}
</script>

<template>
  <div class="container mx-auto p-6 max-w-2xl">
    <div class="mb-6">
      <h1 class="text-3xl font-bold">{{ isEditMode ? 'Edit' : 'Create' }} Area</h1>
      <p class="text-gray-600 mt-1">{{ isEditMode ? 'Update area information' : 'Add a new area/zone' }}</p>
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
              placeholder="Area name"
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
            <Label for="beacon_code">Beacon Code *</Label>
            <Input
              id="beacon_code"
              v-model="formData.beacon_code"
              placeholder="Unique identifier (e.g., A001)"
              required
            />
          </div>

          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <Label>Floor</Label>
              <Button
                v-if="!formData.floor"
                type="button"
                @click="addFloor"
                variant="outline"
                size="sm"
              >
                Add Floor
              </Button>
              <Button
                v-else
                type="button"
                @click="removeFloor"
                variant="outline"
                size="sm"
              >
                Remove Floor
              </Button>
            </div>

            <div v-if="formData.floor" class="grid grid-cols-2 gap-4 p-4 border rounded">
              <div class="space-y-2">
                <Label for="floor-type">Type</Label>
                <select
                  id="floor-type"
                  v-model="formData.floor.type"
                  class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                >
                  <option value="floor">Floor</option>
                  <option value="level">Level</option>
                  <option value="basement">Basement</option>
                </select>
              </div>
              <div class="space-y-2">
                <Label for="floor-name">Number</Label>
                <Input
                  id="floor-name"
                  type="number"
                  v-model.number="formData.floor.name"
                />
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <Label for="polygon">Polygon (JSON) *</Label>
            <textarea
              id="polygon"
              v-model="formData.polygonInput"
              @blur="parsePolygon"
              placeholder='[[x1, y1], [x2, y2], [x3, y3], ...]'
              class="flex min-h-[120px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono"
              required
            />
            <p class="text-xs text-gray-500">
              Enter polygon coordinates as JSON array. Example: [[0, 0], [100, 0], [100, 100], [0, 100]]
            </p>
            <div v-if="formData.polygon.length > 0" class="text-xs text-green-600">
              Valid polygon with {{ formData.polygon.length }} points
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
