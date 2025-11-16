<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Merchant } from '@/schema'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Icon } from '@iconify/vue'
import { useFavoritesStore } from '@/states/favorites'
import { useHistoryStore } from '@/states/history'

interface Props {
  merchants: Merchant[]
  loading?: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  navigate: [merchant: Merchant]
  select: [merchant: Merchant]
}>()

const favorites = useFavoritesStore()
const history = useHistoryStore()

const searchQuery = ref('')
const selectedTypes = ref<string[]>([])
const selectedTags = ref<string[]>([])
const sortBy = ref<'name' | 'distance' | 'recent'>('name')

// Get all unique types and tags from merchants
const availableTypes = computed(() => {
  const types = new Set<string>()
  props.merchants.forEach((m) => {
    const type = m.type
    if (typeof type === 'string') {
      types.add(type)
    } else if ('food' in type) {
      types.add('food')
    } else if ('electronics' in type) {
      types.add('electronics')
    } else if ('clothing' in type) {
      types.add('clothing')
    }
  })
  return Array.from(types)
})

const availableTags = computed(() => {
  const tags = new Set<string>()
  props.merchants.forEach((m) => {
    if (m.tags) {
      m.tags.forEach((tag) => tags.add(tag))
    }
  })
  return Array.from(tags).slice(0, 10) // Limit to 10 most common tags
})

// Filtered merchants based on search and filters
const filteredMerchants = computed(() => {
  let results = [...props.merchants]

  // Text search
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    results = results.filter(
      (m) =>
        m.name.toLowerCase().includes(query) ||
        (m.description || '').toLowerCase().includes(query) ||
        (m.tags && m.tags.some((tag) => tag.toLowerCase().includes(query)))
    )
  }

  // Type filter
  if (selectedTypes.value.length > 0) {
    results = results.filter((m) => {
      const type = m.type
      if (typeof type === 'string') {
        return selectedTypes.value.includes(type)
      } else if ('food' in type) {
        return selectedTypes.value.includes('food')
      } else if ('electronics' in type) {
        return selectedTypes.value.includes('electronics')
      } else if ('clothing' in type) {
        return selectedTypes.value.includes('clothing')
      }
      return false
    })
  }

  // Tags filter
  if (selectedTags.value.length > 0) {
    results = results.filter((m) => m.tags && m.tags.some((tag) => selectedTags.value.includes(tag)))
  }

  // Sort
  if (sortBy.value === 'name') {
    results.sort((a, b) => a.name.localeCompare(b.name))
  } else if (sortBy.value === 'recent') {
    // TODO: Sort by recent visits from history
  } else if (sortBy.value === 'distance') {
    // TODO: Sort by distance from current location
  }

  return results
})

function toggleType(type: string) {
  const index = selectedTypes.value.indexOf(type)
  if (index > -1) {
    selectedTypes.value.splice(index, 1)
  } else {
    selectedTypes.value.push(type)
  }
}

function toggleTag(tag: string) {
  const index = selectedTags.value.indexOf(tag)
  if (index > -1) {
    selectedTags.value.splice(index, 1)
  } else {
    selectedTags.value.push(tag)
  }
}

function clearFilters() {
  searchQuery.value = ''
  selectedTypes.value = []
  selectedTags.value = []
}

function toggleFavorite(merchant: Merchant) {
  const merchantId = merchant.id || ''
  if (favorites.isMerchantFavorited(merchantId)) {
    favorites.removeMerchantFavorite(merchantId)
  } else {
    favorites.addMerchantFavorite(merchant)
  }
}

function handleNavigate(merchant: Merchant) {
  // Add to search history
  if (searchQuery.value) {
    history.addSearchEntry(searchQuery.value, filteredMerchants.value.length)
  }
  emit('navigate', merchant)
}

function getMerchantTypeLabel(merchant: Merchant): string {
  const type = merchant.type
  if (typeof type === 'string') {
    return type
  } else if ('food' in type) {
    return 'Food'
  } else if ('electronics' in type) {
    return 'Electronics'
  } else if ('clothing' in type) {
    return 'Clothing'
  }
  return 'Other'
}
</script>

<template>
  <div class="space-y-4">
    <!-- Search Bar -->
    <div class="relative">
      <Icon icon="mdi:magnify" class="absolute left-3 top-3 w-5 h-5 text-gray-400" />
      <Input
        v-model="searchQuery"
        placeholder="Search merchants by name, description, or tags..."
        class="pl-10 pr-10"
      />
      <Button
        v-if="searchQuery"
        variant="ghost"
        size="sm"
        class="absolute right-1 top-1"
        @click="searchQuery = ''"
      >
        <Icon icon="mdi:close" class="w-4 h-4" />
      </Button>
    </div>

    <!-- Filter Bar -->
    <div class="flex items-center gap-2 flex-wrap">
      <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Filters:</span>

      <!-- Sort -->
      <select
        v-model="sortBy"
        class="text-sm border border-gray-300 dark:border-gray-600 rounded-md px-2 py-1 bg-white dark:bg-gray-800"
      >
        <option value="name">Sort by Name</option>
        <option value="distance">Sort by Distance</option>
        <option value="recent">Sort by Recent</option>
      </select>

      <!-- Clear Filters -->
      <Button
        v-if="selectedTypes.length > 0 || selectedTags.length > 0 || searchQuery"
        variant="outline"
        size="sm"
        @click="clearFilters"
      >
        <Icon icon="mdi:filter-off" class="w-4 h-4 mr-1" />
        Clear
      </Button>

      <div class="flex-1"></div>

      <!-- Results count -->
      <span class="text-sm text-gray-500 dark:text-gray-400">
        {{ filteredMerchants.length }} result{{ filteredMerchants.length !== 1 ? 's' : '' }}
      </span>
    </div>

    <!-- Type Filters -->
    <div v-if="availableTypes.length > 0" class="space-y-2">
      <p class="text-sm font-medium text-gray-700 dark:text-gray-300">Types:</p>
      <div class="flex flex-wrap gap-2">
        <Badge
          v-for="type in availableTypes"
          :key="type"
          :variant="selectedTypes.includes(type) ? 'default' : 'outline'"
          class="cursor-pointer"
          @click="toggleType(type)"
        >
          <Icon
            :icon="selectedTypes.includes(type) ? 'mdi:check-circle' : 'mdi:circle-outline'"
            class="w-3 h-3 mr-1"
          />
          {{ type }}
        </Badge>
      </div>
    </div>

    <!-- Tag Filters -->
    <div v-if="availableTags.length > 0" class="space-y-2">
      <p class="text-sm font-medium text-gray-700 dark:text-gray-300">Tags:</p>
      <div class="flex flex-wrap gap-2">
        <Badge
          v-for="tag in availableTags"
          :key="tag"
          :variant="selectedTags.includes(tag) ? 'default' : 'secondary'"
          class="cursor-pointer"
          @click="toggleTag(tag)"
        >
          <Icon
            :icon="selectedTags.includes(tag) ? 'mdi:check' : 'mdi:tag-outline'"
            class="w-3 h-3 mr-1"
          />
          {{ tag }}
        </Badge>
      </div>
    </div>

    <Separator />

    <!-- Results -->
    <div v-if="loading" class="text-center py-8">
      <Icon icon="mdi:loading" class="w-8 h-8 animate-spin mx-auto text-blue-500" />
      <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">Loading merchants...</p>
    </div>

    <div v-else-if="filteredMerchants.length === 0" class="text-center py-8">
      <Icon icon="mdi:store-off-outline" class="w-16 h-16 mx-auto mb-4 text-gray-300 dark:text-gray-600" />
      <p class="text-gray-500 dark:text-gray-400">No merchants found</p>
      <Button variant="outline" class="mt-4" @click="clearFilters" v-if="searchQuery || selectedTypes.length > 0 || selectedTags.length > 0">
        <Icon icon="mdi:filter-off" class="w-4 h-4 mr-2" />
        Clear Filters
      </Button>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <Card v-for="merchant in filteredMerchants" :key="merchant.id || merchant.name">
        <CardHeader>
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <CardTitle class="text-lg">{{ merchant.name }}</CardTitle>
              <p class="text-sm text-gray-500 dark:text-gray-400">{{ merchant.description }}</p>
            </div>
            <Button
              variant="ghost"
              size="sm"
              @click="toggleFavorite(merchant)"
              :class="favorites.isMerchantFavorited(merchant.id || '') ? 'text-red-500' : ''"
            >
              <Icon
                :icon="favorites.isMerchantFavorited(merchant.id || '') ? 'mdi:heart' : 'mdi:heart-outline'"
                class="w-5 h-5"
              />
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div class="space-y-3">
            <!-- Type Badge -->
            <div>
              <Badge variant="secondary">
                <Icon icon="mdi:store" class="w-3 h-3 mr-1" />
                {{ getMerchantTypeLabel(merchant) }}
              </Badge>
            </div>

            <!-- Tags -->
            <div v-if="merchant.tags && merchant.tags.length > 0" class="flex flex-wrap gap-2">
              <Badge v-for="tag in merchant.tags.slice(0, 3)" :key="tag" variant="outline" class="text-xs">
                {{ tag }}
              </Badge>
            </div>

            <!-- Contact Info -->
            <div class="flex flex-wrap gap-4 text-xs text-gray-600 dark:text-gray-400">
              <div v-if="merchant.phone" class="flex items-center">
                <Icon icon="mdi:phone" class="w-3 h-3 mr-1" />
                {{ merchant.phone }}
              </div>
              <div v-if="merchant.website" class="flex items-center">
                <Icon icon="mdi:web" class="w-3 h-3 mr-1" />
                Website
              </div>
            </div>

            <Separator />

            <!-- Actions -->
            <div class="flex gap-2">
              <Button class="flex-1" @click="handleNavigate(merchant)">
                <Icon icon="mdi:navigation" class="w-4 h-4 mr-2" />
                Navigate
              </Button>
              <Button variant="outline" @click="emit('select', merchant)">
                <Icon icon="mdi:information-outline" class="w-4 h-4" />
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
