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
  imageUrl: '',
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
        area: merchant.area,
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
        imageUrl: merchant.image_url || '',
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
        image_url: formData.value.imageUrl || null,
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
        image_url: formData.value.imageUrl || null,
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

// Real-time validation
const nameError = ref<string | null>(null)
const beaconCodeError = ref<string | null>(null)
const emailError = ref<string | null>(null)
const websiteError = ref<string | null>(null)

function validateName() {
  if (!formData.value.name || formData.value.name.trim().length === 0) {
    nameError.value = 'Name is required'
  } else if (formData.value.name.length < 2) {
    nameError.value = 'Name must be at least 2 characters'
  } else {
    nameError.value = null
  }
}

function validateBeaconCode() {
  if (!formData.value.beacon_code || formData.value.beacon_code.trim().length === 0) {
    beaconCodeError.value = 'Beacon code is required'
  } else if (!/^[A-Z0-9-]+$/.test(formData.value.beacon_code)) {
    beaconCodeError.value = 'Beacon code must contain only uppercase letters, numbers, and hyphens'
  } else {
    beaconCodeError.value = null
  }
}

function validateEmail() {
  if (formData.value.email && formData.value.email.length > 0) {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (!emailRegex.test(formData.value.email)) {
      emailError.value = 'Invalid email format'
    } else {
      emailError.value = null
    }
  } else {
    emailError.value = null
  }
}

function validateWebsite() {
  if (formData.value.website && formData.value.website.length > 0) {
    try {
      new URL(formData.value.website)
      websiteError.value = null
    } catch {
      websiteError.value = 'Invalid URL format (must include https://)'
    }
  } else {
    websiteError.value = null
  }
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
              @blur="validateName"
              @input="validateName"
              placeholder="Merchant name"
              required
              :class="nameError ? 'border-red-500' : ''"
            />
            <p v-if="nameError" class="text-xs text-red-600">{{ nameError }}</p>
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
              <option v-for="area in areas" :key="area.id" :value="area.id">
                {{ area.name }}
              </option>
            </select>
          </div>

          <div class="space-y-2">
            <Label for="beacon_code">Beacon Code *</Label>
            <Input
              id="beacon_code"
              v-model="formData.beacon_code"
              @blur="validateBeaconCode"
              @input="validateBeaconCode"
              placeholder="Unique identifier (e.g., M001)"
              required
              :class="beaconCodeError ? 'border-red-500' : ''"
            />
            <p v-if="beaconCodeError" class="text-xs text-red-600">{{ beaconCodeError }}</p>
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
              <Label for="image-url">Merchant Image URL</Label>
              <Input
                id="image-url"
                v-model="formData.imageUrl"
                type="url"
                placeholder="https://example.com/image.jpg"
              />
              <div v-if="formData.imageUrl" class="mt-2">
                <img
                  :src="formData.imageUrl"
                  alt="Merchant preview"
                  class="w-full max-w-xs h-48 object-cover rounded-md border"
                  @error="() => formData.imageUrl = ''"
                />
              </div>
            </div>

            <div class="space-y-2">
              <Label for="website">Website</Label>
              <Input
                id="website"
                v-model="formData.website"
                @blur="validateWebsite"
                type="url"
                placeholder="https://example.com"
                :class="websiteError ? 'border-red-500' : ''"
              />
              <p v-if="websiteError" class="text-xs text-red-600">{{ websiteError }}</p>
            </div>
            <div class="space-y-2">
              <Label for="email">Email</Label>
              <Input
                id="email"
                v-model="formData.email"
                @blur="validateEmail"
                type="email"
                placeholder="contact@example.com"
                :class="emailError ? 'border-red-500' : ''"
              />
              <p v-if="emailError" class="text-xs text-red-600">{{ emailError }}</p>
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
