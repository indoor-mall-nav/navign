<script setup lang="ts">
import { computed, ref } from 'vue'
import { Icon } from '@iconify/vue'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Checkbox } from '@/components/ui/checkbox'
import { Separator } from '@/components/ui/separator'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import type { MapMerchant } from '@/lib/api/tauri'
import type { RoutePreferences } from '../types'

const props = defineProps<{
  merchants: MapMerchant[]
  preferences: RoutePreferences
  loading?: boolean
  error?: string
  entityId: string
}>()

const emit = defineEmits<{
  'select-destination': [merchant: MapMerchant]
  'calculate-route': [merchant: MapMerchant]
  'update:preferences': [preferences: RoutePreferences]
}>()

const searchQuery = ref('')
const selectedMerchant = ref<MapMerchant | null>(null)

const filteredMerchants = computed(() => {
  if (!searchQuery.value) return props.merchants
  const query = searchQuery.value.toLowerCase()
  return props.merchants.filter(
    (m) =>
      m.name.toLowerCase().includes(query) ||
      m.tags.some((tag) => tag.toLowerCase().includes(query)),
  )
})

function selectMerchant(merchant: MapMerchant) {
  selectedMerchant.value = merchant
  emit('select-destination', merchant)
}

function handleCalculateRoute() {
  if (selectedMerchant.value) {
    emit('calculate-route', selectedMerchant.value)
  }
}

function updatePreference(key: keyof RoutePreferences, value: boolean) {
  const newPreferences = { ...props.preferences, [key]: value }
  emit('update:preferences', newPreferences)
}
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Icon icon="mdi:map-marker" class="w-5 h-5" />
        Select Destination
      </CardTitle>
      <CardDescription> Search and select where you want to go </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <!-- Search Input -->
      <div class="space-y-2">
        <Input
          v-model="searchQuery"
          placeholder="Search merchants, stores, or tags..."
          class="w-full"
        >
          <template #prefix>
            <Icon icon="mdi:magnify" class="w-4 h-4" />
          </template>
        </Input>

        <!-- Merchant List -->
        <div class="max-h-64 overflow-y-auto space-y-2">
          <Card
            v-for="merchant in filteredMerchants"
            :key="merchant.id"
            class="p-3 cursor-pointer transition-colors"
            :class="{
              'bg-accent border-primary': selectedMerchant?.id === merchant.id,
              'hover:bg-accent/50': selectedMerchant?.id !== merchant.id,
            }"
            @click="selectMerchant(merchant)"
          >
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <Icon icon="mdi:store" class="w-4 h-4 text-blue-500" />
                  <span class="font-medium">{{ merchant.name }}</span>
                </div>
                <div class="flex flex-wrap gap-1 mt-2">
                  <Badge
                    v-for="tag in merchant.tags"
                    :key="tag"
                    variant="secondary"
                    class="text-xs"
                  >
                    {{ tag }}
                  </Badge>
                </div>
              </div>
              <Icon
                v-if="selectedMerchant?.id === merchant.id"
                icon="mdi:check-circle"
                class="w-5 h-5 text-primary flex-shrink-0"
              />
            </div>
          </Card>
        </div>
      </div>

      <Separator />

      <!-- Route Preferences -->
      <div class="space-y-3">
        <Label class="text-sm font-medium">Route Preferences</Label>
        <div class="flex flex-col gap-2">
          <div class="flex items-center space-x-2">
            <Checkbox
              :id="`elevator-${entityId}`"
              :checked="preferences.elevator"
              @update:checked="(val: boolean) => updatePreference('elevator', !!val)"
            />
            <label
              :for="`elevator-${entityId}`"
              class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer flex items-center gap-2"
            >
              <Icon icon="mdi:elevator" class="w-4 h-4 text-purple-500" />
              Allow Elevators
            </label>
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              :id="`stairs-${entityId}`"
              :checked="preferences.stairs"
              @update:checked="(val: boolean) => updatePreference('stairs', !!val)"
            />
            <label
              :for="`stairs-${entityId}`"
              class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer flex items-center gap-2"
            >
              <Icon icon="mdi:stairs" class="w-4 h-4 text-orange-500" />
              Allow Stairs
            </label>
          </div>
          <div class="flex items-center space-x-2">
            <Checkbox
              :id="`escalator-${entityId}`"
              :checked="preferences.escalator"
              @update:checked="(val: boolean) => updatePreference('escalator', !!val)"
            />
            <label
              :for="`escalator-${entityId}`"
              class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer flex items-center gap-2"
            >
              <Icon icon="mdi:escalator" class="w-4 h-4 text-green-500" />
              Allow Escalators
            </label>
          </div>
        </div>
      </div>

      <!-- Error Message -->
      <div v-if="error" class="text-sm text-red-500">
        {{ error }}
      </div>

      <!-- Calculate Route Button -->
      <Button
        class="w-full"
        :disabled="!selectedMerchant || loading"
        @click="handleCalculateRoute"
      >
        <Icon
          v-if="loading"
          icon="mdi:loading"
          class="w-4 h-4 mr-2 animate-spin"
        />
        <Icon v-else icon="mdi:routes" class="w-4 h-4 mr-2" />
        {{ loading ? 'Calculating...' : 'Calculate Route' }}
      </Button>
    </CardContent>
  </Card>
</template>
