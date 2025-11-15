<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { createMerchant, updateMerchant, getMerchant, listAreas } from '@/lib/api/client'
import type { MerchantCreateRequest, MerchantUpdateRequest } from '@/lib/api/client'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const loading = ref(false)
const error = ref<string | null>(null)
const areas = ref<any[]>([])

const entityId = computed(() => route.query.entity as string || '')
const merchantId = computed(() => route.query.id as string || '')
const isEditMode = computed(() => !!merchantId.value)

// Form data
const formData = ref({
  name: '',
  description: '',
  chain: '',
  area: '',
  beacon_code: '',
  type: 'other' as any,
  tags: [] as string[],
  tagsInput: '',
  location: [0, 0] as [number, number],
  style: 'store' as 'store' | 'kiosk' | 'popUp' | 'foodTruck' | 'room',
  polygon: [] as [number, number][],
  polygonInput: '',
  website: '',
  phone: '',
  email: '',
})

onMounted(async () => {
  await loadAreas()
  if (isEditMode.value) {
    await loadMerchant()
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

async function loadMerchant() {
  if (!entityId.value || !merchantId.value) return

  loading.value = true
  error.value = null

  try {
    const response = await getMerchant(entityId.value, merchantId.value, session.userToken || '')
    if (response.status === 'success' && response.data) {
      const merchant = response.data
      formData.value = {
        name: merchant.name,
        description: merchant.description || '',
        chain: merchant.chain || '',
        area: merchant.area.$oid,
        beacon_code: merchant.beacon_code,
        type: merchant.type,
        tags: merchant.tags,
        tagsInput: merchant.tags.join(', '),
        location: merchant.location,
        style: merchant.style,
        polygon: merchant.polygon || [],
        polygonInput: JSON.stringify(merchant.polygon || []),
        website: merchant.website || '',
        phone: merchant.phone || '',
        email: merchant.email || '',
      }
    } else {
      error.value = response.message || 'Failed to load merchant'
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

function parsePolygon() {
  try {
    if (!formData.value.polygonInput.trim()) {
      formData.value.polygon = []
      return
    }
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

  if (!formData.value.name || !formData.value.area || !formData.value.beacon_code) {
    error.value = 'Name, area, and beacon code are required'
    return
  }

  // Parse tags and polygon
  parseTags()
  if (formData.value.polygonInput) {
    parsePolygon()
    if (error.value) return
  }

  loading.value = true
  error.value = null

  try {
    if (isEditMode.value) {
      const updateData: MerchantUpdateRequest = {
        _id: merchantId.value,
        name: formData.value.name,
        description: formData.value.description || null,
        chain: formData.value.chain || null,
        area: formData.value.area,
        beacon_code: formData.value.beacon_code,
        type: formData.value.type,
        tags: formData.value.tags,
        location: formData.value.location,
        style: formData.value.style,
        polygon: formData.value.polygon,
        website: formData.value.website || null,
        phone: formData.value.phone || undefined,
        email: formData.value.email || null,
      }
      const response = await updateMerchant(entityId.value, updateData, session.userToken || '')
      if (response.status === 'success') {
        router.push({ name: 'admin-merchants', query: { entity: entityId.value } })
      } else {
        error.value = response.message || 'Failed to update merchant'
      }
    } else {
      const createData: MerchantCreateRequest = {
        entity: entityId.value,
        name: formData.value.name,
        description: formData.value.description || null,
        chain: formData.value.chain || null,
        area: formData.value.area,
        beacon_code: formData.value.beacon_code,
        type: formData.value.type,
        tags: formData.value.tags,
        location: formData.value.location,
        style: formData.value.style,
        polygon: formData.value.polygon,
        website: formData.value.website || null,
        phone: formData.value.phone || undefined,
        email: formData.value.email || null,
      }
      const response = await createMerchant(entityId.value, createData, session.userToken || '')
      if (response.status === 'success') {
        router.push({ name: 'admin-merchants', query: { entity: entityId.value } })
      } else {
        error.value = response.message || 'Failed to create merchant'
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function handleCancel() {
  router.push({ name: 'admin-merchants', query: { entity: entityId.value } })
}
</script>

<template>
  <div class="container mx-auto p-6 max-w-2xl">
    <div class="mb-6">
      <h1 class="text-3xl font-bold">{{ isEditMode ? 'Edit' : 'Create' }} Merchant</h1>
      <p class="text-gray-600 mt-1">{{ isEditMode ? 'Update merchant information' : 'Add a new merchant' }}</p>
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
              placeholder="Merchant name"
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
            <Label for="chain">Chain</Label>
            <Input
              id="chain"
              v-model="formData.chain"
              placeholder="Chain name (if applicable)"
            />
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
              <option v-for="area in areas" :key="area._id.$oid" :value="area._id.$oid">
                {{ area.name }}
              </option>
            </select>
          </div>

          <div class="space-y-2">
            <Label for="beacon_code">Beacon Code *</Label>
            <Input
              id="beacon_code"
              v-model="formData.beacon_code"
              placeholder="Unique identifier (e.g., M001)"
              required
            />
          </div>

          <div class="space-y-2">
            <Label for="style">Style *</Label>
            <select
              id="style"
              v-model="formData.style"
              class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              required
            >
              <option value="store">Store</option>
              <option value="kiosk">Kiosk</option>
              <option value="popUp">Pop-up</option>
              <option value="foodTruck">Food Truck</option>
              <option value="room">Room</option>
            </select>
          </div>

          <div class="space-y-2">
            <Label for="tags">Tags (comma-separated)</Label>
            <Input
              id="tags"
              v-model="formData.tagsInput"
              @blur="parseTags"
              placeholder="food, restaurant, italian"
            />
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

          <div class="space-y-2">
            <Label for="polygon">Polygon (JSON, optional)</Label>
            <textarea
              id="polygon"
              v-model="formData.polygonInput"
              @blur="parsePolygon"
              placeholder='[[x1, y1], [x2, y2], ...]'
              class="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono"
            />
          </div>

          <div class="grid grid-cols-1 gap-4">
            <div class="space-y-2">
              <Label for="website">Website</Label>
              <Input
                id="website"
                v-model="formData.website"
                type="url"
                placeholder="https://example.com"
              />
            </div>
            <div class="space-y-2">
              <Label for="email">Email</Label>
              <Input
                id="email"
                v-model="formData.email"
                type="email"
                placeholder="contact@example.com"
              />
            </div>
            <div class="space-y-2">
              <Label for="phone">Phone</Label>
              <Input
                id="phone"
                v-model="formData.phone"
                type="tel"
                placeholder="+1234567890"
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
