<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { useFavoritesStore } from '@/states/favorites'
import { useHistoryStore } from '@/states/history'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Badge } from '@/components/ui/badge'
import { Icon } from '@iconify/vue'
import { info } from '@tauri-apps/plugin-log'

const router = useRouter()
const session = useSessionStore()
const favorites = useFavoritesStore()
const history = useHistoryStore()

const isEditing = ref(false)
const newPassword = ref('')
const confirmNewPassword = ref('')
const currentPassword = ref('')
const errorMessage = ref('')
const successMessage = ref('')

const userStats = computed(() => ({
  totalNavigations: history.navigationHistory.length,
  completedNavigations: history.completedNavigations.length,
  totalFavorites: favorites.merchants.length + favorites.areas.length,
  favoritesMerchants: favorites.merchants.length,
  favoritesAreas: favorites.areas.length,
}))

onMounted(() => {
  if (!session.isAuthenticated) {
    router.push('/login')
  }
})

function handleLogout() {
  session.clearSession()
  localStorage.removeItem('auth_token')
  router.push('/login')
}

function toggleEditMode() {
  isEditing.value = !isEditing.value
  if (!isEditing.value) {
    // Clear password fields when canceling
    newPassword.value = ''
    confirmNewPassword.value = ''
    currentPassword.value = ''
    errorMessage.value = ''
    successMessage.value = ''
  }
}

async function handlePasswordChange() {
  errorMessage.value = ''
  successMessage.value = ''

  if (!currentPassword.value || !newPassword.value || !confirmNewPassword.value) {
    errorMessage.value = 'Please fill in all password fields'
    return
  }

  if (newPassword.value !== confirmNewPassword.value) {
    errorMessage.value = 'New passwords do not match'
    return
  }

  if (newPassword.value.length < 8) {
    errorMessage.value = 'New password must be at least 8 characters'
    return
  }

  try {
    // TODO: Implement password change API call
    await info('Password change request (API not implemented yet)')
    successMessage.value = 'Password changed successfully!'

    // Clear fields and exit edit mode
    setTimeout(() => {
      toggleEditMode()
    }, 2000)
  } catch (error) {
    errorMessage.value = `Failed to change password: ${error}`
  }
}

function navigateToSettings() {
  router.push('/settings')
}

function navigateToFavorites() {
  router.push('/favorites')
}

function navigateToHistory() {
  router.push('/history')
}
</script>

<template>
  <div class="min-h-screen bg-gradient-to-b from-blue-50 to-white dark:from-gray-900 dark:to-gray-800 p-6">
    <div class="max-w-4xl mx-auto">
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <Button variant="ghost" @click="router.push('/')">
          <Icon icon="mdi:arrow-left" class="w-5 h-5 mr-2" />
          Back to Home
        </Button>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Profile</h1>
        <Button variant="ghost" @click="navigateToSettings">
          <Icon icon="mdi:cog" class="w-5 h-5" />
        </Button>
      </div>

      <!-- User Info Card -->
      <Card class="mb-6">
        <CardHeader>
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-4">
              <div class="w-20 h-20 rounded-full bg-gradient-to-br from-blue-400 to-blue-600 flex items-center justify-center">
                <Icon icon="mdi:account" class="w-12 h-12 text-white" />
              </div>
              <div>
                <CardTitle class="text-2xl">{{ session.userId || 'User' }}</CardTitle>
                <CardDescription>Member since {{ new Date().toLocaleDateString() }}</CardDescription>
                <div class="flex gap-2 mt-2">
                  <Badge variant="secondary">
                    <Icon icon="mdi:check-circle" class="w-4 h-4 mr-1" />
                    Verified
                  </Badge>
                </div>
              </div>
            </div>
            <Button variant="outline" @click="handleLogout">
              <Icon icon="mdi:logout" class="w-4 h-4 mr-2" />
              Logout
            </Button>
          </div>
        </CardHeader>
      </Card>

      <!-- Stats Grid -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <Card>
          <CardContent class="pt-6">
            <div class="text-center">
              <Icon icon="mdi:navigation" class="w-10 h-10 mx-auto mb-2 text-blue-500" />
              <div class="text-3xl font-bold text-gray-900 dark:text-white">
                {{ userStats.completedNavigations }}
              </div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Completed Routes</div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent class="pt-6">
            <div class="text-center">
              <Icon icon="mdi:heart" class="w-10 h-10 mx-auto mb-2 text-red-500" />
              <div class="text-3xl font-bold text-gray-900 dark:text-white">
                {{ userStats.totalFavorites }}
              </div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Favorites</div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent class="pt-6">
            <div class="text-center">
              <Icon icon="mdi:history" class="w-10 h-10 mx-auto mb-2 text-green-500" />
              <div class="text-3xl font-bold text-gray-900 dark:text-white">
                {{ userStats.totalNavigations }}
              </div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Total Navigations</div>
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Quick Actions -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Quick Actions</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Button variant="outline" class="h-auto py-4" @click="navigateToFavorites">
              <div class="flex flex-col items-center">
                <Icon icon="mdi:heart-outline" class="w-8 h-8 mb-2" />
                <span>View Favorites</span>
              </div>
            </Button>
            <Button variant="outline" class="h-auto py-4" @click="navigateToHistory">
              <div class="flex flex-col items-center">
                <Icon icon="mdi:history" class="w-8 h-8 mb-2" />
                <span>View History</span>
              </div>
            </Button>
            <Button variant="outline" class="h-auto py-4" @click="navigateToSettings">
              <div class="flex flex-col items-center">
                <Icon icon="mdi:cog-outline" class="w-8 h-8 mb-2" />
                <span>Settings</span>
              </div>
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- Password Change Section -->
      <Card>
        <CardHeader>
          <div class="flex items-center justify-between">
            <div>
              <CardTitle>Security</CardTitle>
              <CardDescription>Manage your password and security settings</CardDescription>
            </div>
            <Button variant="outline" @click="toggleEditMode">
              <Icon :icon="isEditing ? 'mdi:close' : 'mdi:pencil'" class="w-4 h-4 mr-2" />
              {{ isEditing ? 'Cancel' : 'Change Password' }}
            </Button>
          </div>
        </CardHeader>
        <CardContent v-if="isEditing">
          <Separator class="mb-4" />
          <div class="space-y-4">
            <div>
              <Label for="current-password">Current Password</Label>
              <Input
                id="current-password"
                type="password"
                v-model="currentPassword"
                placeholder="Enter current password"
              />
            </div>
            <div>
              <Label for="new-password">New Password</Label>
              <Input
                id="new-password"
                type="password"
                v-model="newPassword"
                placeholder="Enter new password (min 8 characters)"
              />
            </div>
            <div>
              <Label for="confirm-password">Confirm New Password</Label>
              <Input
                id="confirm-password"
                type="password"
                v-model="confirmNewPassword"
                placeholder="Confirm new password"
              />
            </div>

            <div v-if="errorMessage" class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
              <p class="text-sm text-red-600 dark:text-red-400 flex items-center">
                <Icon icon="mdi:alert-circle" class="w-4 h-4 mr-2" />
                {{ errorMessage }}
              </p>
            </div>

            <div v-if="successMessage" class="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-md">
              <p class="text-sm text-green-600 dark:text-green-400 flex items-center">
                <Icon icon="mdi:check-circle" class="w-4 h-4 mr-2" />
                {{ successMessage }}
              </p>
            </div>

            <div class="flex justify-end gap-3">
              <Button variant="outline" @click="toggleEditMode">Cancel</Button>
              <Button @click="handlePasswordChange">
                <Icon icon="mdi:lock-reset" class="w-4 h-4 mr-2" />
                Update Password
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
