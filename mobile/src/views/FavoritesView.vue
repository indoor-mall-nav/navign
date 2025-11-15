<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { useFavoritesStore } from '@/states/favorites'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import { Badge } from '@/components/ui/badge'
import { Icon } from '@iconify/vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

const router = useRouter()
const session = useSessionStore()
const favorites = useFavoritesStore()

const activeTab = ref<'merchants' | 'areas'>('merchants')
const searchQuery = ref('')
const selectedItem = ref<any>(null)
const showDeleteDialog = ref(false)
const showClearAllDialog = ref(false)

onMounted(() => {
  if (!session.isAuthenticated) {
    router.push('/login')
  }
})

const filteredMerchants = computed(() => {
  if (!searchQuery.value) return favorites.sortedMerchants

  const query = searchQuery.value.toLowerCase()
  return favorites.sortedMerchants.filter((fav) =>
    fav.merchant.name.toLowerCase().includes(query) ||
    (fav.merchant.description || '').toLowerCase().includes(query) ||
    (fav.notes || '').toLowerCase().includes(query)
  )
})

const filteredAreas = computed(() => {
  if (!searchQuery.value) return favorites.sortedAreas

  const query = searchQuery.value.toLowerCase()
  return favorites.sortedAreas.filter((fav) =>
    fav.area.name.toLowerCase().includes(query) ||
    (fav.area.description || '').toLowerCase().includes(query) ||
    (fav.label || '').toLowerCase().includes(query)
  )
})

function formatDate(timestamp: number) {
  const date = new Date(timestamp)
  const now = new Date()
  const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24))

  if (diffDays === 0) return 'Today'
  if (diffDays === 1) return 'Yesterday'
  if (diffDays < 7) return `${diffDays} days ago`
  return date.toLocaleDateString()
}

function navigateToMerchant(merchantId: string) {
  // TODO: Navigate to merchant details
  router.push(`/merchant/${merchantId}`)
}

function navigateToArea(areaId: string) {
  // TODO: Navigate to area
  router.push(`/area/${areaId}`)
}

function confirmDelete(item: any, type: 'merchant' | 'area') {
  selectedItem.value = { item, type }
  showDeleteDialog.value = true
}

function handleDelete() {
  if (!selectedItem.value) return

  const { item, type } = selectedItem.value
  if (type === 'merchant') {
    favorites.removeMerchantFavorite(item.merchantId)
  } else {
    favorites.removeAreaFavorite(item.areaId)
  }

  showDeleteDialog.value = false
  selectedItem.value = null
}

function clearAllFavorites() {
  showClearAllDialog.value = true
}

function confirmClearAll() {
  favorites.clearAllFavorites()
  showClearAllDialog.value = false
}
</script>

<template>
  <div class="min-h-screen bg-gradient-to-b from-blue-50 to-white dark:from-gray-900 dark:to-gray-800 p-6">
    <div class="max-w-6xl mx-auto">
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <Button variant="ghost" @click="router.push('/profile')">
          <Icon icon="mdi:arrow-left" class="w-5 h-5 mr-2" />
          Back to Profile
        </Button>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Favorites</h1>
        <Button variant="ghost" @click="clearAllFavorites" v-if="favorites.merchants.length > 0 || favorites.areas.length > 0">
          <Icon icon="mdi:delete-outline" class="w-5 h-5" />
        </Button>
      </div>

      <!-- Tabs -->
      <div class="flex gap-2 mb-6">
        <Button
          :variant="activeTab === 'merchants' ? 'default' : 'outline'"
          @click="activeTab = 'merchants'"
          class="flex-1"
        >
          <Icon icon="mdi:store" class="w-4 h-4 mr-2" />
          Merchants ({{ favorites.merchants.length }})
        </Button>
        <Button
          :variant="activeTab === 'areas' ? 'default' : 'outline'"
          @click="activeTab = 'areas'"
          class="flex-1"
        >
          <Icon icon="mdi:map-marker" class="w-4 h-4 mr-2" />
          Areas ({{ favorites.areas.length }})
        </Button>
      </div>

      <!-- Search -->
      <div class="mb-6">
        <div class="relative">
          <Icon icon="mdi:magnify" class="absolute left-3 top-3 w-5 h-5 text-gray-400" />
          <Input
            v-model="searchQuery"
            placeholder="Search favorites..."
            class="pl-10"
          />
        </div>
      </div>

      <!-- Merchants Tab -->
      <div v-if="activeTab === 'merchants'">
        <div v-if="filteredMerchants.length === 0" class="text-center py-12">
          <Icon icon="mdi:heart-outline" class="w-16 h-16 mx-auto mb-4 text-gray-300 dark:text-gray-600" />
          <p class="text-gray-500 dark:text-gray-400">
            {{ searchQuery ? 'No merchants found' : 'No favorite merchants yet' }}
          </p>
          <Button variant="outline" class="mt-4" @click="router.push('/')">
            <Icon icon="mdi:compass-outline" class="w-4 h-4 mr-2" />
            Explore Merchants
          </Button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card v-for="fav in filteredMerchants" :key="fav.merchantId">
            <CardHeader>
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <CardTitle class="text-lg">{{ fav.merchant.name }}</CardTitle>
                  <CardDescription>{{ fav.merchant.description }}</CardDescription>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  @click="confirmDelete(fav, 'merchant')"
                  class="text-red-500 hover:text-red-700"
                >
                  <Icon icon="mdi:heart" class="w-5 h-5" />
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div class="space-y-3">
                <div class="flex items-center text-sm text-gray-500 dark:text-gray-400">
                  <Icon icon="mdi:clock-outline" class="w-4 h-4 mr-2" />
                  Saved {{ formatDate(fav.savedAt) }}
                </div>
                <div v-if="fav.merchant.tags && fav.merchant.tags.length > 0" class="flex flex-wrap gap-2">
                  <Badge v-for="tag in fav.merchant.tags.slice(0, 3)" :key="tag" variant="secondary">
                    {{ tag }}
                  </Badge>
                </div>
                <div v-if="fav.notes" class="p-2 bg-yellow-50 dark:bg-yellow-900/20 rounded text-sm">
                  <Icon icon="mdi:note-text-outline" class="w-4 h-4 inline mr-1" />
                  {{ fav.notes }}
                </div>
                <Separator />
                <Button class="w-full" @click="navigateToMerchant(fav.merchantId)">
                  <Icon icon="mdi:navigation" class="w-4 h-4 mr-2" />
                  Navigate Here
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <!-- Areas Tab -->
      <div v-if="activeTab === 'areas'">
        <div v-if="filteredAreas.length === 0" class="text-center py-12">
          <Icon icon="mdi:heart-outline" class="w-16 h-16 mx-auto mb-4 text-gray-300 dark:text-gray-600" />
          <p class="text-gray-500 dark:text-gray-400">
            {{ searchQuery ? 'No areas found' : 'No favorite areas yet' }}
          </p>
          <Button variant="outline" class="mt-4" @click="router.push('/')">
            <Icon icon="mdi:compass-outline" class="w-4 h-4 mr-2" />
            Explore Areas
          </Button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card v-for="fav in filteredAreas" :key="fav.areaId">
            <CardHeader>
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <CardTitle class="text-lg">
                    {{ fav.label || fav.area.name }}
                  </CardTitle>
                  <CardDescription>{{ fav.area.description }}</CardDescription>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  @click="confirmDelete(fav, 'area')"
                  class="text-red-500 hover:text-red-700"
                >
                  <Icon icon="mdi:heart" class="w-5 h-5" />
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div class="space-y-3">
                <div class="flex items-center text-sm text-gray-500 dark:text-gray-400">
                  <Icon icon="mdi:clock-outline" class="w-4 h-4 mr-2" />
                  Saved {{ formatDate(fav.savedAt) }}
                </div>
                <div class="flex items-center text-sm">
                  <Icon icon="mdi:floor-plan" class="w-4 h-4 mr-2" />
                  Floor: {{ fav.area.floor?.name ?? 'Ground' }}
                </div>
                <Separator />
                <Button class="w-full" @click="navigateToArea(fav.areaId)">
                  <Icon icon="mdi:map-marker" class="w-4 h-4 mr-2" />
                  View on Map
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <!-- Delete Confirmation Dialog -->
      <Dialog v-model:open="showDeleteDialog">
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Remove Favorite?</DialogTitle>
            <DialogDescription>
              This will remove this item from your favorites. You can add it back anytime.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" @click="showDeleteDialog = false">Cancel</Button>
            <Button variant="destructive" @click="handleDelete">Remove</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <!-- Clear All Confirmation Dialog -->
      <Dialog v-model:open="showClearAllDialog">
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Clear All Favorites?</DialogTitle>
            <DialogDescription>
              This will remove all your favorite merchants and areas. This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" @click="showClearAllDialog = false">Cancel</Button>
            <Button variant="destructive" @click="confirmClearAll">Clear All</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  </div>
</template>
