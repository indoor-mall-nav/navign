<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { listMerchants, deleteMerchant } from '@/lib/api/client'
import type { Merchant } from '@/schema'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const merchants = ref<Merchant[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const deleteDialogOpen = ref(false)
const merchantToDelete = ref<Merchant | null>(null)

const entityId = computed(() => route.query.entity as string || session.entity?.id || '')

onMounted(async () => {
  await loadMerchants()
})

async function loadMerchants() {
  if (!entityId.value) {
    error.value = 'No entity selected'
    return
  }

  loading.value = true
  error.value = null

  try {
    const response = await listMerchants(entityId.value, session.userToken || '')
    if (response.status === 'success' && response.data) {
      merchants.value = response.data
    } else {
      error.value = response.message || 'Failed to load merchants'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function navigateToCreate() {
  router.push({ name: 'admin-merchants-form', query: { entity: entityId.value } })
}

function navigateToEdit(merchant: Merchant) {
  router.push({
    name: 'admin-merchants-form',
    query: {
      entity: entityId.value,
      id: merchant.id
    }
  })
}

function openDeleteDialog(merchant: Merchant) {
  merchantToDelete.value = merchant
  deleteDialogOpen.value = true
}

async function confirmDelete() {
  if (!merchantToDelete.value || !entityId.value) return

  loading.value = true
  error.value = null

  try {
    const response = await deleteMerchant(
      entityId.value,
      String(merchantToDelete.value.id),
      session.userToken || ''
    )
    if (response.status === 'success') {
      await loadMerchants()
      deleteDialogOpen.value = false
      merchantToDelete.value = null
    } else {
      error.value = response.message || 'Failed to delete merchant'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function getMerchantTypeLabel(type: Merchant['type']): string {
  if (typeof type === 'string') return type
  if ('food' in type) return 'Food'
  if ('electronics' in type) return 'Electronics'
  if ('clothing' in type) return 'Clothing'
  return 'Other'
}

function getStyleBadgeVariant(style: string) {
  const variants: Record<string, any> = {
    store: 'default',
    kiosk: 'secondary',
    popUp: 'outline',
    foodTruck: 'outline',
    room: 'outline',
  }
  return variants[style] || 'outline'
}
</script>

<template>
  <div class="container mx-auto p-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-3xl font-bold">Merchants</h1>
        <p class="text-gray-600 mt-1">Manage stores, restaurants, and other merchants</p>
      </div>
      <Button @click="navigateToCreate">
        Create Merchant
      </Button>
    </div>

    <div v-if="error" class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md text-red-800">
      {{ error }}
    </div>

    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
      <p class="mt-2 text-gray-600">Loading merchants...</p>
    </div>

    <div v-else-if="merchants.length === 0" class="text-center py-12">
      <p class="text-gray-600">No merchants found. Create your first merchant to get started.</p>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <Card v-for="merchant in merchants" :key="merchant.id" class="hover:shadow-lg transition-shadow">
        <CardHeader>
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <CardTitle class="text-lg">{{ merchant.name }}</CardTitle>
              <CardDescription v-if="merchant.description" class="mt-1">
                {{ merchant.description }}
              </CardDescription>
              <p v-if="merchant.chain" class="text-xs text-gray-500 mt-1">
                Chain: {{ merchant.chain }}
              </p>
            </div>
            <Badge :variant="getStyleBadgeVariant(merchant.style)">
              {{ merchant.style }}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div class="space-y-2 text-sm">
            <div class="flex justify-between">
              <span class="text-gray-600">Type:</span>
              <span class="font-medium">{{ getMerchantTypeLabel(merchant.type) }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-gray-600">Beacon Code:</span>
              <span class="font-medium font-mono">{{ merchant.beacon_code }}</span>
            </div>
            <div v-if="merchant.tags.length > 0" class="pt-2">
              <div class="flex flex-wrap gap-1">
                <Badge v-for="tag in merchant.tags" :key="tag" variant="outline" class="text-xs">
                  {{ tag }}
                </Badge>
              </div>
            </div>
          </div>

          <div class="flex gap-2 mt-4">
            <Button @click="navigateToEdit(merchant)" variant="outline" class="flex-1">
              Edit
            </Button>
            <Button
              @click="openDeleteDialog(merchant)"
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
          <DialogTitle>Delete Merchant</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{{ merchantToDelete?.name }}"? This action cannot be undone.
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
