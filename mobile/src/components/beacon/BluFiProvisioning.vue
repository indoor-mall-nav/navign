<script setup lang="ts">
/**
 * BluFi Provisioning Component - Placeholder
 *
 * This is a minimal placeholder for future BluFi provisioning UI.
 * All business logic is in Rust (mobile/src-tauri/src/blufi/).
 *
 * TODO:
 * - Implement beacon scanning UI
 * - Add beacon connection flow
 * - Implement WiFi network selection
 * - Add WiFi credential input form
 * - Implement orchestrator configuration
 * - Add provisioning progress indicator
 * - Add connection verification display
 */

import { ref } from 'vue'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Icon } from '@iconify/vue'
import {
  scanProvisioningBeacons,
  connectBeacon,
  scanWifiNetworks,
  provisionBeacon,
  disconnectBeacon,
  type ProvisioningBeacon,
  type WiFiNetwork,
  type BluFiConfig,
} from '@/lib/api/blufi'

// State
const scanning = ref(false)
const connecting = ref(false)
const provisioning = ref(false)
const beacons = ref<ProvisioningBeacon[]>([])
const networks = ref<WiFiNetwork[]>([])
const error = ref('')

// Handlers (placeholders)
async function handleScanBeacons() {
  scanning.value = true
  error.value = ''
  try {
    beacons.value = await scanProvisioningBeacons()
  } catch (err) {
    error.value = `Failed to scan beacons: ${err}`
  } finally {
    scanning.value = false
  }
}

async function handleConnect(macAddress: string) {
  connecting.value = true
  error.value = ''
  try {
    await connectBeacon(macAddress)
    // TODO: Navigate to WiFi setup step
  } catch (err) {
    error.value = `Failed to connect: ${err}`
  } finally {
    connecting.value = false
  }
}

async function handleScanWifi() {
  try {
    networks.value = await scanWifiNetworks()
  } catch (err) {
    error.value = `Failed to scan WiFi: ${err}`
  }
}

async function handleProvision(config: BluFiConfig) {
  provisioning.value = true
  error.value = ''
  try {
    const result = await provisionBeacon(config)
    // TODO: Show success/failure result
    console.log('Provisioning result:', result)
  } catch (err) {
    error.value = `Failed to provision: ${err}`
  } finally {
    provisioning.value = false
  }
}

async function handleDisconnect() {
  try {
    await disconnectBeacon()
  } catch (err) {
    error.value = `Failed to disconnect: ${err}`
  }
}
</script>

<template>
  <div class="blufi-provisioning">
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Icon icon="mdi:wifi-cog" class="w-5 h-5" />
          Beacon WiFi Configuration
        </CardTitle>
        <CardDescription>
          Configure WiFi credentials for ESP32 beacons using BluFi
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <!-- Placeholder Content -->
        <div class="text-center p-8 border-2 border-dashed rounded-lg">
          <Icon icon="mdi:wrench" class="w-16 h-16 mx-auto mb-4 text-muted-foreground" />
          <h3 class="text-lg font-semibold mb-2">Under Construction</h3>
          <p class="text-sm text-muted-foreground mb-4">
            BluFi provisioning UI is coming soon.
          </p>
          <p class="text-xs text-muted-foreground mb-4">
            This feature will allow you to configure WiFi credentials on ESP32 beacons
            over Bluetooth, enabling them to connect to your WiFi network and the
            orchestrator server.
          </p>

          <!-- Test Button (for development) -->
          <Button
            variant="outline"
            :disabled="scanning"
            @click="handleScanBeacons"
          >
            <Icon
              v-if="scanning"
              icon="mdi:loading"
              class="w-4 h-4 mr-2 animate-spin"
            />
            <Icon v-else icon="mdi:bluetooth-search" class="w-4 h-4 mr-2" />
            {{ scanning ? 'Scanning...' : 'Test: Scan Beacons' }}
          </Button>

          <!-- Error Display -->
          <div
            v-if="error"
            class="mt-4 p-3 bg-destructive/10 text-destructive text-sm rounded-md"
          >
            <Icon icon="mdi:alert-circle" class="w-4 h-4 inline mr-2" />
            {{ error }}
          </div>

          <!-- Development Info -->
          <div
            v-if="beacons.length > 0"
            class="mt-4 p-3 bg-muted text-sm rounded-md text-left"
          >
            <p class="font-semibold mb-2">Found {{ beacons.length }} beacon(s):</p>
            <pre class="text-xs">{{ JSON.stringify(beacons, null, 2) }}</pre>
          </div>
        </div>

        <!-- TODO: Implement actual UI -->
        <div class="text-xs text-muted-foreground space-y-1">
          <p><strong>TODO:</strong> Implement UI for:</p>
          <ul class="list-disc list-inside ml-4 space-y-1">
            <li>Beacon scanning and selection</li>
            <li>WiFi network scanning and selection</li>
            <li>WiFi credential input (SSID, password, security)</li>
            <li>Orchestrator configuration (URL, port, entity ID)</li>
            <li>Beacon metadata (name, location)</li>
            <li>Provisioning progress indicator</li>
            <li>Connection verification</li>
          </ul>
        </div>
      </CardContent>
    </Card>
  </div>
</template>
