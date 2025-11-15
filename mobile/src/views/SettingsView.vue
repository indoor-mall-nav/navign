<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useSessionStore } from '@/states/session'
import { useSettingsStore } from '@/states/settings'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
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
const settings = useSettingsStore()

const showResetDialog = ref(false)
const biometricAvailable = ref(false)

onMounted(async () => {
  if (!session.isAuthenticated) {
    router.push('/login')
  }

  // Apply theme on mount
  settings.applyTheme()

  // Check if biometric is available
  try {
    // TODO: Check biometric availability using tauri-plugin-biometric
    biometricAvailable.value = false // Placeholder
  } catch (error) {
    biometricAvailable.value = false
  }
})

// Watch theme changes and apply
watch(() => settings.theme, () => {
  settings.applyTheme()
})

function handleResetSettings() {
  settings.resetToDefaults()
  showResetDialog.value = false
}
</script>

<template>
  <div class="min-h-screen bg-gradient-to-b from-blue-50 to-white dark:from-gray-900 dark:to-gray-800 p-6">
    <div class="max-w-4xl mx-auto">
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <Button variant="ghost" @click="router.push('/profile')">
          <Icon icon="mdi:arrow-left" class="w-5 h-5 mr-2" />
          Back to Profile
        </Button>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Settings</h1>
        <div class="w-24"></div>
      </div>

      <!-- Appearance Settings -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Appearance</CardTitle>
          <CardDescription>Customize the app's look and feel</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <Label>Theme</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Choose your preferred theme</p>
            </div>
            <div class="flex gap-2">
              <Button
                :variant="settings.theme === 'light' ? 'default' : 'outline'"
                @click="settings.setTheme('light')"
                class="w-20"
              >
                <Icon icon="mdi:white-balance-sunny" class="w-4 h-4 mr-1" />
                Light
              </Button>
              <Button
                :variant="settings.theme === 'dark' ? 'default' : 'outline'"
                @click="settings.setTheme('dark')"
                class="w-20"
              >
                <Icon icon="mdi:moon-waning-crescent" class="w-4 h-4 mr-1" />
                Dark
              </Button>
              <Button
                :variant="settings.theme === 'system' ? 'default' : 'outline'"
                @click="settings.setTheme('system')"
                class="w-20"
              >
                <Icon icon="mdi:laptop" class="w-4 h-4 mr-1" />
                Auto
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Location & Tracking -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Location & Tracking</CardTitle>
          <CardDescription>Manage location services and tracking preferences</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <Label>Enable Location Tracking</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Allow app to track your indoor position</p>
            </div>
            <Button
              :variant="settings.enableLocationTracking ? 'default' : 'outline'"
              @click="settings.toggleLocationTracking()"
            >
              <Icon :icon="settings.enableLocationTracking ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
          <Separator />
          <div class="flex items-center justify-between">
            <div>
              <Label>Auto-Update Position</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Automatically update your position while navigating</p>
            </div>
            <Button
              :variant="settings.autoUpdatePosition ? 'default' : 'outline'"
              @click="settings.toggleAutoUpdatePosition()"
              :disabled="!settings.enableLocationTracking"
            >
              <Icon :icon="settings.autoUpdatePosition ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
          <Separator />
          <div class="flex items-center justify-between">
            <div>
              <Label>Update Interval</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">How often to update position (seconds)</p>
            </div>
            <div class="flex gap-2">
              <Button
                v-for="interval in [3, 5, 10, 15]"
                :key="interval"
                :variant="settings.positionUpdateInterval === interval ? 'default' : 'outline'"
                @click="settings.setPositionUpdateInterval(interval)"
                :disabled="!settings.autoUpdatePosition"
                class="w-12"
              >
                {{ interval }}s
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Map Display -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Map Display</CardTitle>
          <CardDescription>Control what appears on the map</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <Label>Show Beacons</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Display beacon locations on the map</p>
            </div>
            <Button
              :variant="settings.showBeaconsOnMap ? 'default' : 'outline'"
              @click="settings.toggleBeaconsOnMap()"
            >
              <Icon :icon="settings.showBeaconsOnMap ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
          <Separator />
          <div class="flex items-center justify-between">
            <div>
              <Label>Show Merchants</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Display merchant locations on the map</p>
            </div>
            <Button
              :variant="settings.showMerchantsOnMap ? 'default' : 'outline'"
              @click="settings.toggleMerchantsOnMap()"
            >
              <Icon :icon="settings.showMerchantsOnMap ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- Navigation Preferences -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Navigation Preferences</CardTitle>
          <CardDescription>Customize your navigation experience</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <Label>Allow Stairs</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Include stairs in route calculations</p>
            </div>
            <Button
              :variant="settings.navigationPreferences.allowStairs ? 'default' : 'outline'"
              @click="settings.updateNavigationPreferences({ allowStairs: !settings.navigationPreferences.allowStairs })"
            >
              <Icon :icon="settings.navigationPreferences.allowStairs ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
          <Separator />
          <div class="flex items-center justify-between">
            <div>
              <Label>Allow Elevators</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Include elevators in route calculations</p>
            </div>
            <Button
              :variant="settings.navigationPreferences.allowElevators ? 'default' : 'outline'"
              @click="settings.updateNavigationPreferences({ allowElevators: !settings.navigationPreferences.allowElevators })"
            >
              <Icon :icon="settings.navigationPreferences.allowElevators ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
          <Separator />
          <div class="flex items-center justify-between">
            <div>
              <Label>Allow Escalators</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Include escalators in route calculations</p>
            </div>
            <Button
              :variant="settings.navigationPreferences.allowEscalators ? 'default' : 'outline'"
              @click="settings.updateNavigationPreferences({ allowEscalators: !settings.navigationPreferences.allowEscalators })"
            >
              <Icon :icon="settings.navigationPreferences.allowEscalators ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
          <Separator />
          <div class="flex items-center justify-between">
            <div>
              <Label>Prefer Fastest Route</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Optimize for speed over distance</p>
            </div>
            <Button
              :variant="settings.navigationPreferences.preferFastestRoute ? 'default' : 'outline'"
              @click="settings.updateNavigationPreferences({ preferFastestRoute: !settings.navigationPreferences.preferFastestRoute })"
            >
              <Icon :icon="settings.navigationPreferences.preferFastestRoute ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- Security -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Security</CardTitle>
          <CardDescription>Manage security and authentication settings</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <Label>Biometric Authentication</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">
                {{ biometricAvailable ? 'Use fingerprint/face recognition to unlock' : 'Not available on this device' }}
              </p>
            </div>
            <Button
              :variant="settings.enableBiometric ? 'default' : 'outline'"
              @click="settings.toggleBiometric()"
              :disabled="!biometricAvailable"
            >
              <Icon :icon="settings.enableBiometric ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- Notifications -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Notifications</CardTitle>
          <CardDescription>Control notification preferences</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <Label>Enable Notifications</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Receive navigation and promotional alerts</p>
            </div>
            <Button
              :variant="settings.enableNotifications ? 'default' : 'outline'"
              @click="settings.toggleNotifications()"
            >
              <Icon :icon="settings.enableNotifications ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- Privacy -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Privacy</CardTitle>
          <CardDescription>Manage your privacy and data preferences</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <Label>Share Usage Data</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Help improve the app by sharing anonymous usage data</p>
            </div>
            <Button
              :variant="settings.privacy.shareUsageData ? 'default' : 'outline'"
              @click="settings.updatePrivacySettings({ shareUsageData: !settings.privacy.shareUsageData })"
            >
              <Icon :icon="settings.privacy.shareUsageData ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
          <Separator />
          <div class="flex items-center justify-between">
            <div>
              <Label>Crash Reporting</Label>
              <p class="text-sm text-gray-500 dark:text-gray-400">Automatically send crash reports to help fix bugs</p>
            </div>
            <Button
              :variant="settings.privacy.enableCrashReporting ? 'default' : 'outline'"
              @click="settings.updatePrivacySettings({ enableCrashReporting: !settings.privacy.enableCrashReporting })"
            >
              <Icon :icon="settings.privacy.enableCrashReporting ? 'mdi:check' : 'mdi:close'" class="w-4 h-4" />
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- Reset Settings -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Reset</CardTitle>
          <CardDescription>Restore default settings</CardDescription>
        </CardHeader>
        <CardContent>
          <Button variant="destructive" @click="showResetDialog = true">
            <Icon icon="mdi:restore" class="w-4 h-4 mr-2" />
            Reset to Defaults
          </Button>
        </CardContent>
      </Card>

      <!-- Reset Confirmation Dialog -->
      <Dialog v-model:open="showResetDialog">
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Reset Settings?</DialogTitle>
            <DialogDescription>
              This will restore all settings to their default values. This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" @click="showResetDialog = false">Cancel</Button>
            <Button variant="destructive" @click="handleResetSettings">Reset</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  </div>
</template>
