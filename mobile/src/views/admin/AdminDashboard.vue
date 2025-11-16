<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const entityId = computed(() => route.query.entity as string || session.entity?.id || '')

const adminSections = [
  {
    name: 'Beacons',
    description: 'Manage BLE beacons for indoor positioning and access control',
    icon: '📡',
    route: 'admin-beacons',
    color: 'bg-blue-50 hover:bg-blue-100 border-blue-200'
  },
  {
    name: 'Areas',
    description: 'Manage areas and zones within your entity',
    icon: '🗺️',
    route: 'admin-areas',
    color: 'bg-green-50 hover:bg-green-100 border-green-200'
  },
  {
    name: 'Merchants',
    description: 'Manage stores, restaurants, and other merchants',
    icon: '🏪',
    route: 'admin-merchants',
    color: 'bg-purple-50 hover:bg-purple-100 border-purple-200'
  },
  {
    name: 'Connections',
    description: 'Manage connections between areas (elevators, stairs, gates)',
    icon: '🔗',
    route: 'admin-connections',
    color: 'bg-orange-50 hover:bg-orange-100 border-orange-200'
  }
]

function navigateTo(routeName: string) {
  router.push({ name: routeName, query: { entity: entityId.value } })
}
</script>

<template>
  <div class="container mx-auto p-6">
    <div class="mb-8">
      <h1 class="text-4xl font-bold mb-2">Admin Dashboard</h1>
      <p class="text-gray-600">Manage your indoor navigation system</p>
    </div>

    <div v-if="!entityId" class="mb-6 p-4 bg-yellow-50 border border-yellow-200 rounded-md text-yellow-800">
      <p class="font-medium">No entity selected</p>
      <p class="text-sm mt-1">Please select an entity to manage its data.</p>
    </div>

    <div class="grid gap-6 md:grid-cols-2">
      <Card
        v-for="section in adminSections"
        :key="section.name"
        :class="['cursor-pointer transition-all border-2', section.color]"
        @click="navigateTo(section.route)"
      >
        <CardHeader>
          <div class="flex items-center gap-3">
            <span class="text-4xl">{{ section.icon }}</span>
            <div>
              <CardTitle class="text-2xl">{{ section.name }}</CardTitle>
              <CardDescription class="mt-1">{{ section.description }}</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <Button variant="outline" class="w-full">
            Manage {{ section.name }}
          </Button>
        </CardContent>
      </Card>
    </div>

    <div class="mt-8 p-6 bg-gray-50 rounded-lg border border-gray-200">
      <h2 class="text-xl font-semibold mb-2">Quick Tips</h2>
      <ul class="space-y-2 text-sm text-gray-700">
        <li class="flex items-start gap-2">
          <span class="text-blue-600 font-bold">•</span>
          <span><strong>Beacons:</strong> Create beacons for indoor positioning. Each beacon must be associated with an area.</span>
        </li>
        <li class="flex items-start gap-2">
          <span class="text-green-600 font-bold">•</span>
          <span><strong>Areas:</strong> Define zones using polygon coordinates. Areas represent physical spaces like floors or rooms.</span>
        </li>
        <li class="flex items-start gap-2">
          <span class="text-purple-600 font-bold">•</span>
          <span><strong>Merchants:</strong> Add businesses or points of interest. Merchants can have tags for easy search and filtering.</span>
        </li>
        <li class="flex items-start gap-2">
          <span class="text-orange-600 font-bold">•</span>
          <span><strong>Connections:</strong> Link areas together using elevators, stairs, gates, or other connection types for pathfinding.</span>
        </li>
      </ul>
    </div>
  </div>
</template>
