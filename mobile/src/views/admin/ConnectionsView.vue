<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { listConnections, deleteConnection } from '@/lib/api/client'
import type { Connection } from '@/schema'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const connections = ref<Connection[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const deleteDialogOpen = ref(false)
const connectionToDelete = ref<Connection | null>(null)

const entityId = computed(() => route.query.entity as string || session.entity?.id || '')

onMounted(async () => {
  await loadConnections()
})

async function loadConnections() {
  if (!entityId.value) {
    error.value = 'No entity selected'
    return
  }

  loading.value = true
  error.value = null

  try {
    const response = await listConnections(entityId.value, session.userToken || '')
    if (response.status === 'success' && response.data) {
      connections.value = response.data
    } else {
      error.value = response.message || 'Failed to load connections'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function navigateToCreate() {
  router.push({ name: 'admin-connections-form', query: { entity: entityId.value } })
}

function navigateToEdit(connection: Connection) {
  router.push({
    name: 'admin-connections-form',
    query: {
      entity: entityId.value,
      id: connection.id
    }
  })
}

function openDeleteDialog(connection: Connection) {
  connectionToDelete.value = connection
  deleteDialogOpen.value = true
}

async function confirmDelete() {
  if (!connectionToDelete.value || !entityId.value) return

  loading.value = true
  error.value = null

  try {
    const response = await deleteConnection(
      entityId.value,
      String(connectionToDelete.value.id),
      session.userToken || ''
    )
    if (response.status === 'success') {
      await loadConnections()
      deleteDialogOpen.value = false
      connectionToDelete.value = null
    } else {
      error.value = response.message || 'Failed to delete connection'
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
  } finally {
    loading.value = false
  }
}

function getTypeBadgeVariant(type: string) {
  const variants: Record<string, any> = {
    gate: 'default',
    elevator: 'default',
    escalator: 'secondary',
    stairs: 'secondary',
    rail: 'outline',
    shuttle: 'outline',
  }
  return variants[type] || 'outline'
}

function getTypeIcon(type: string): string {
  const icons: Record<string, string> = {
    gate: '🚪',
    elevator: '🛗',
    escalator: '↗️',
    stairs: '🪜',
    rail: '🚇',
    shuttle: '🚐',
  }
  return icons[type] || '🔗'
}
</script>

<template>
  <div class="container mx-auto p-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-3xl font-bold">Connections</h1>
        <p class="text-gray-600 mt-1">Manage connections between areas (elevators, stairs, gates, etc.)</p>
      </div>
      <Button @click="navigateToCreate">
        Create Connection
      </Button>
    </div>

    <div v-if="error" class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md text-red-800">
      {{ error }}
    </div>

    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
      <p class="mt-2 text-gray-600">Loading connections...</p>
    </div>

    <div v-else-if="connections.length === 0" class="text-center py-12">
      <p class="text-gray-600">No connections found. Create your first connection to get started.</p>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <Card v-for="connection in connections" :key="connection.id" class="hover:shadow-lg transition-shadow">
        <CardHeader>
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-2">
                <span class="text-2xl">{{ getTypeIcon(connection.type) }}</span>
                <CardTitle class="text-lg">{{ connection.name }}</CardTitle>
              </div>
              <CardDescription v-if="connection.description" class="mt-1">
                {{ connection.description }}
              </CardDescription>
            </div>
            <Badge :variant="getTypeBadgeVariant(connection.type)">
              {{ connection.type }}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div class="space-y-2 text-sm">
            <div class="flex justify-between">
              <span class="text-gray-600">Connected Areas:</span>
              <span class="font-medium">{{ connection.connected_areas.length }}</span>
            </div>
            <div v-if="connection.tags.length > 0" class="pt-2">
              <div class="flex flex-wrap gap-1">
                <Badge v-for="tag in connection.tags" :key="tag" variant="outline" class="text-xs">
                  {{ tag }}
                </Badge>
              </div>
            </div>
          </div>

          <div class="flex gap-2 mt-4">
            <Button @click="navigateToEdit(connection)" variant="outline" class="flex-1">
              Edit
            </Button>
            <Button
              @click="openDeleteDialog(connection)"
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
          <DialogTitle>Delete Connection</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{{ connectionToDelete?.name }}"? This action cannot be undone.
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
