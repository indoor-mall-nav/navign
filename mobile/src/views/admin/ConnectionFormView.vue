<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { createConnection, updateConnection, getConnection, listAreas } from '@/lib/api/client'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Connection } from '@/schema'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const loading = ref(false)
const error = ref<string | null>(null)
const areas = ref<any[]>([])

const entityId = computed(() => route.query.entity as string || '')
const connectionId = computed(() => parseInt(route.query.id as string) || -1)
const isEditMode = computed(() => !!connectionId.value)

// Form data
const formData = ref({
  name: '',
  description: '',
  type: 'gate' as 'gate' | 'escalator' | 'elevator' | 'stairs' | 'rail' | 'shuttle',
  connected_areas: [] as [number, number, number, boolean][],
  connected_areas_input: '',
  available_period: [] as [number, number][],
  available_period_input: '',
  tags: [] as string[],
  tagsInput: '',
})

onMounted(async () => {
  await loadAreas()
  if (isEditMode.value) {
    await loadConnection()
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

async function loadConnection() {
  if (!entityId.value || !connectionId.value) return

  loading.value = true
  error.value = null

  try {
    const response = await getConnection(entityId.value, connectionId.value, session.userToken || '')
    if (response.status === 'success' && response.data) {
      const connection = response.data
      formData.value = {
        name: connection.name,
        description: connection.description || '',
        type: connection.type,
        connected_areas: connection.connected_areas.map((ca: any) => [
          ca[0],
          ca[1],
          ca[2],
          ca[3]
        ]) as [number, number, number, boolean][],
        connected_areas_input: JSON.stringify(connection.connected_areas.map((ca: any) => [
          ca[0],
          ca[1],
          ca[2],
          ca[3]
        ])),
        available_period: connection.available_period,
        available_period_input: JSON.stringify(connection.available_period),
        tags: connection.tags,
        tagsInput: connection.tags.join(', '),
      }
    } else {
      error.value = response.message || 'Failed to load connection'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function parseTags() {
  formData.value.tags = formData.value.tagsInput
    .split(',')
    .map(t => t.trim())
    .filter(t => t.length > 0)
}

function parseConnectedAreas() {
  try {
    if (!formData.value.connected_areas_input.trim()) {
      formData.value.connected_areas = []
      return
    }
    const parsed = JSON.parse(formData.value.connected_areas_input)
    if (Array.isArray(parsed) && parsed.every(ca => Array.isArray(ca) && ca.length === 3)) {
      formData.value.connected_areas = parsed
      error.value = null
    } else {
      error.value = 'Invalid connected areas format. Expected: [["area_id", x, y], ...]'
    }
  } catch (err) {
    error.value = 'Invalid JSON format for connected areas'
  }
}

function parseAvailablePeriod() {
  try {
    if (!formData.value.available_period_input.trim()) {
      formData.value.available_period = []
      return
    }
    const parsed = JSON.parse(formData.value.available_period_input)
    if (Array.isArray(parsed) && parsed.every(p => Array.isArray(p) && p.length === 2)) {
      formData.value.available_period = parsed
      error.value = null
    } else {
      error.value = 'Invalid available period format. Expected: [[start, end], ...]'
    }
  } catch (err) {
    error.value = 'Invalid JSON format for available period'
  }
}

async function handleSubmit() {
  if (!entityId.value) {
    error.value = 'No entity selected'
    return
  }

  if (!formData.value.name) {
    error.value = 'Name is required'
    return
  }

  // Parse all inputs
  parseTags()
  if (formData.value.connected_areas_input) {
    parseConnectedAreas()
    if (error.value) return
  }
  if (formData.value.available_period_input) {
    parseAvailablePeriod()
    if (error.value) return
  }

  if (formData.value.connected_areas.length < 2) {
    error.value = 'Connection must connect at least 2 areas'
    return
  }

  loading.value = true
  error.value = null

  try {
    if (isEditMode.value) {
      const updateData: Connection = {
        id: connectionId.value,
        entity_id: entityId.value,
        name: formData.value.name,
        description: formData.value.description || null,
        type: formData.value.type,
        connected_areas: formData.value.connected_areas,
        available_period: formData.value.available_period,
        tags: formData.value.tags,
        gnd: [0.0, 0.0],
        created_at: '',
        updated_at: ''
      }
      const response = await updateConnection(entityId.value, updateData, session.userToken || '')
      if (response.status === 'success') {
        router.push({ name: 'admin-connections', query: { entity: entityId.value } })
      } else {
        error.value = response.message || 'Failed to update connection'
      }
    } else {
      const createData: Connection = {
        id: -1,
        entity_id: entityId.value,
        name: formData.value.name,
        description: formData.value.description || null,
        type: formData.value.type,
        connected_areas: formData.value.connected_areas,
        available_period: formData.value.available_period,
        tags: formData.value.tags,
        gnd: [0.0, 0.0],
        created_at: '',
        updated_at: ''
      }
      const response = await createConnection(entityId.value, createData, session.userToken || '')
      if (response.status === 'success') {
        router.push({ name: 'admin-connections', query: { entity: entityId.value } })
      } else {
        error.value = response.message || 'Failed to create connection'
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function handleCancel() {
  router.push({ name: 'admin-connections', query: { entity: entityId.value } })
}
</script>

<template>
  <div class="container mx-auto p-6 max-w-2xl">
    <div class="mb-6">
      <h1 class="text-3xl font-bold">{{ isEditMode ? 'Edit' : 'Create' }} Connection</h1>
      <p class="text-gray-600 mt-1">{{ isEditMode ? 'Update connection information' : 'Add a new connection between areas' }}</p>
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
              placeholder="Connection name"
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
              <option value="gate">Gate 🚪</option>
              <option value="elevator">Elevator 🛗</option>
              <option value="escalator">Escalator ↗️</option>
              <option value="stairs">Stairs 🪜</option>
              <option value="rail">Rail 🚇</option>
              <option value="shuttle">Shuttle 🚐</option>
            </select>
          </div>

          <div class="space-y-2">
            <Label for="connected_areas">Connected Areas (JSON) *</Label>
            <textarea
              id="connected_areas"
              v-model="formData.connected_areas_input"
              @blur="parseConnectedAreas"
              placeholder='[["area_id_1", x1, y1], ["area_id_2", x2, y2]]'
              class="flex min-h-[100px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono"
              required
            />
            <p class="text-xs text-gray-500">
              Format: [["area_id", x, y], ...] where area_id is the area's ObjectId, and x, y are coordinates.
            </p>
            <div v-if="formData.connected_areas.length > 0" class="text-xs text-green-600">
              Valid: {{ formData.connected_areas.length }} connected areas
            </div>
          </div>

          <div class="space-y-2">
            <Label for="available_period">Available Period (JSON, optional)</Label>
            <textarea
              id="available_period"
              v-model="formData.available_period_input"
              @blur="parseAvailablePeriod"
              placeholder='[[36000000, 72000000]] (10:00-20:00)'
              class="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono"
            />
            <p class="text-xs text-gray-500">
              Format: [[start_ms, end_ms], ...] - milliseconds from midnight.
            </p>
          </div>

          <div class="space-y-2">
            <Label for="tags">Tags (comma-separated)</Label>
            <Input
              id="tags"
              v-model="formData.tagsInput"
              @blur="parseTags"
              placeholder="accessible, express, fast"
            />
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
