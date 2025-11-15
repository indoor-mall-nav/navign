<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { useHistoryStore } from '@/states/history'
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
const history = useHistoryStore()

const activeTab = ref<'navigation' | 'search' | 'frequent'>('navigation')
const searchQuery = ref('')
const showClearDialog = ref(false)

onMounted(() => {
  if (!session.isAuthenticated) {
    router.push('/login')
  }
})

const filteredNavigations = computed(() => {
  if (!searchQuery.value) return history.recentNavigations

  const query = searchQuery.value.toLowerCase()
  return history.recentNavigations.filter((nav) =>
    nav.from.areaName.toLowerCase().includes(query) ||
    (nav.to.merchantName || '').toLowerCase().includes(query) ||
    (nav.to.areaName || '').toLowerCase().includes(query)
  )
})

const filteredSearches = computed(() => {
  if (!searchQuery.value) return history.recentSearches

  const query = searchQuery.value.toLowerCase()
  return history.recentSearches.filter((search) =>
    search.query.toLowerCase().includes(query)
  )
})

function formatDate(timestamp: number) {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins} min ago`
  if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`
  if (diffDays === 1) return 'Yesterday'
  if (diffDays < 7) return `${diffDays} days ago`
  return date.toLocaleDateString()
}

function formatDuration(seconds: number | undefined) {
  if (!seconds) return 'N/A'
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  return `${mins}m ${secs}s`
}

function formatDistance(meters: number | undefined) {
  if (!meters) return 'N/A'
  if (meters < 1000) return `${Math.round(meters)}m`
  return `${(meters / 1000).toFixed(2)}km`
}

function repeatNavigation(nav: any) {
  // TODO: Start navigation to the same destination
  console.log('Repeat navigation:', nav)
}

function deleteNavigation(id: string) {
  history.deleteNavigationEntry(id)
}

function handleClearHistory() {
  if (activeTab.value === 'navigation') {
    history.clearNavigationHistory()
  } else if (activeTab.value === 'search') {
    history.clearSearchHistory()
  }
  showClearDialog.value = false
}

function navigateToDestination(dest: any) {
  // TODO: Navigate to frequent destination
  console.log('Navigate to:', dest)
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
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">History</h1>
        <Button
          variant="ghost"
          @click="showClearDialog = true"
          v-if="(activeTab === 'navigation' && history.navigationHistory.length > 0) ||
                (activeTab === 'search' && history.searchHistory.length > 0)"
        >
          <Icon icon="mdi:delete-outline" class="w-5 h-5" />
        </Button>
      </div>

      <!-- Tabs -->
      <div class="flex gap-2 mb-6">
        <Button
          :variant="activeTab === 'navigation' ? 'default' : 'outline'"
          @click="activeTab = 'navigation'"
          class="flex-1"
        >
          <Icon icon="mdi:navigation" class="w-4 h-4 mr-2" />
          Navigation ({{ history.navigationHistory.length }})
        </Button>
        <Button
          :variant="activeTab === 'search' ? 'default' : 'outline'"
          @click="activeTab = 'search'"
          class="flex-1"
        >
          <Icon icon="mdi:magnify" class="w-4 h-4 mr-2" />
          Search ({{ history.searchHistory.length }})
        </Button>
        <Button
          :variant="activeTab === 'frequent' ? 'default' : 'outline'"
          @click="activeTab = 'frequent'"
          class="flex-1"
        >
          <Icon icon="mdi:star" class="w-4 h-4 mr-2" />
          Frequent
        </Button>
      </div>

      <!-- Search -->
      <div class="mb-6" v-if="activeTab !== 'frequent'">
        <div class="relative">
          <Icon icon="mdi:magnify" class="absolute left-3 top-3 w-5 h-5 text-gray-400" />
          <Input
            v-model="searchQuery"
            :placeholder="`Search ${activeTab}...`"
            class="pl-10"
          />
        </div>
      </div>

      <!-- Navigation History Tab -->
      <div v-if="activeTab === 'navigation'">
        <div v-if="filteredNavigations.length === 0" class="text-center py-12">
          <Icon icon="mdi:history" class="w-16 h-16 mx-auto mb-4 text-gray-300 dark:text-gray-600" />
          <p class="text-gray-500 dark:text-gray-400">
            {{ searchQuery ? 'No navigation history found' : 'No navigation history yet' }}
          </p>
          <Button variant="outline" class="mt-4" @click="router.push('/')">
            <Icon icon="mdi:compass-outline" class="w-4 h-4 mr-2" />
            Start Navigating
          </Button>
        </div>

        <div class="space-y-4">
          <Card v-for="nav in filteredNavigations" :key="nav.id">
            <CardHeader>
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-2 mb-2">
                    <Badge :variant="nav.completed ? 'default' : 'secondary'">
                      <Icon :icon="nav.completed ? 'mdi:check-circle' : 'mdi:clock-outline'" class="w-3 h-3 mr-1" />
                      {{ nav.completed ? 'Completed' : 'Incomplete' }}
                    </Badge>
                    <span class="text-sm text-gray-500 dark:text-gray-400">{{ formatDate(nav.timestamp) }}</span>
                  </div>
                  <CardTitle class="text-lg mb-1">
                    {{ nav.to.merchantName || nav.to.areaName || 'Unknown Destination' }}
                  </CardTitle>
                  <CardDescription class="flex items-center">
                    <Icon icon="mdi:map-marker-outline" class="w-4 h-4 mr-1" />
                    From {{ nav.from.areaName }}
                  </CardDescription>
                </div>
                <div class="flex gap-2">
                  <Button variant="ghost" size="sm" @click="repeatNavigation(nav)">
                    <Icon icon="mdi:repeat" class="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm" @click="deleteNavigation(nav.id)" class="text-red-500">
                    <Icon icon="mdi:delete-outline" class="w-4 h-4" />
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent v-if="nav.completed && (nav.duration || nav.distance)">
              <div class="flex gap-6 text-sm">
                <div class="flex items-center text-gray-600 dark:text-gray-300">
                  <Icon icon="mdi:clock-outline" class="w-4 h-4 mr-2" />
                  Duration: {{ formatDuration(nav.duration) }}
                </div>
                <div class="flex items-center text-gray-600 dark:text-gray-300">
                  <Icon icon="mdi:map-marker-distance" class="w-4 h-4 mr-2" />
                  Distance: {{ formatDistance(nav.distance) }}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <!-- Search History Tab -->
      <div v-if="activeTab === 'search'">
        <div v-if="filteredSearches.length === 0" class="text-center py-12">
          <Icon icon="mdi:history" class="w-16 h-16 mx-auto mb-4 text-gray-300 dark:text-gray-600" />
          <p class="text-gray-500 dark:text-gray-400">
            {{ searchQuery ? 'No search history found' : 'No search history yet' }}
          </p>
        </div>

        <div class="space-y-3">
          <Card v-for="search in filteredSearches" :key="search.timestamp">
            <CardContent class="pt-4">
              <div class="flex items-center justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-3">
                    <Icon icon="mdi:magnify" class="w-5 h-5 text-gray-400" />
                    <div>
                      <p class="font-medium">{{ search.query }}</p>
                      <p class="text-sm text-gray-500 dark:text-gray-400">
                        {{ search.resultCount }} result{{ search.resultCount !== 1 ? 's' : '' }} • {{ formatDate(search.timestamp) }}
                      </p>
                    </div>
                  </div>
                </div>
                <Button variant="ghost" size="sm">
                  <Icon icon="mdi:arrow-right" class="w-4 h-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <!-- Frequent Destinations Tab -->
      <div v-if="activeTab === 'frequent'">
        <div v-if="history.frequentDestinations.length === 0" class="text-center py-12">
          <Icon icon="mdi:star-outline" class="w-16 h-16 mx-auto mb-4 text-gray-300 dark:text-gray-600" />
          <p class="text-gray-500 dark:text-gray-400">No frequent destinations yet</p>
          <p class="text-sm text-gray-400 dark:text-gray-500 mt-2">
            Navigate to places multiple times to see them here
          </p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card v-for="dest in history.frequentDestinations" :key="dest.id">
            <CardHeader>
              <div class="flex items-center justify-between">
                <div class="flex-1">
                  <CardTitle class="text-lg">{{ dest.name }}</CardTitle>
                  <CardDescription>Visited {{ dest.count }} time{{ dest.count > 1 ? 's' : '' }}</CardDescription>
                </div>
                <Badge variant="default">
                  <Icon icon="mdi:star" class="w-4 h-4 mr-1" />
                  {{ dest.count }}
                </Badge>
              </div>
            </CardHeader>
            <CardContent>
              <div class="space-y-3">
                <div class="text-sm text-gray-500 dark:text-gray-400">
                  Last visit: {{ formatDate(dest.lastVisit) }}
                </div>
                <Separator />
                <Button class="w-full" @click="navigateToDestination(dest)">
                  <Icon icon="mdi:navigation" class="w-4 h-4 mr-2" />
                  Navigate Again
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <!-- Clear History Confirmation Dialog -->
      <Dialog v-model:open="showClearDialog">
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Clear {{ activeTab === 'navigation' ? 'Navigation' : 'Search' }} History?</DialogTitle>
            <DialogDescription>
              This will permanently delete all your {{ activeTab }} history. This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" @click="showClearDialog = false">Cancel</Button>
            <Button variant="destructive" @click="handleClearHistory">Clear History</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  </div>
</template>
