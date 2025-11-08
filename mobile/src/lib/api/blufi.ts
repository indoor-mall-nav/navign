/**
 * BluFi provisioning API - TypeScript wrapper for Rust commands
 *
 * This module provides minimal TypeScript wrappers for BluFi provisioning.
 * All business logic is implemented in Rust (mobile/src-tauri/src/blufi/).
 * TypeScript only handles UI concerns.
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * WiFi security mode
 */
export type WiFiSecurityMode =
  | 'open'
  | 'wep'
  | 'wpa_psk'
  | 'wpa2_psk'
  | 'wpa_wpa2_psk'
  | 'wpa2_enterprise'
  | 'wpa3_psk'
  | 'wpa2_wpa3_psk'

/**
 * WiFi network information
 */
export interface WiFiNetwork {
  ssid: string
  bssid?: string
  rssi: number
  channel: number
  security: WiFiSecurityMode
  hidden?: boolean
}

/**
 * BluFi configuration
 */
export interface BluFiConfig {
  ssid: string
  password: string
  security: WiFiSecurityMode
  orchestrator_url?: string
  orchestrator_port?: number
  entity_id?: string
  beacon_name?: string
  beacon_location?: {
    x: number
    y: number
    floor: string
  }
}

/**
 * BluFi provisioning state
 */
export type BluFiState =
  | 'idle'
  | 'scanning'
  | 'connecting'
  | 'negotiating'
  | 'provisioning'
  | 'verifying'
  | 'connected'
  | 'failed'

/**
 * BluFi provisioning result
 */
export interface BluFiProvisioningResult {
  success: boolean
  state: BluFiState
  message?: string
  ip_address?: string
  mac_address?: string
  connected_ssid?: string
  error?: {
    error_type: string
    message: string
    details?: string
    code?: number
  }
}

/**
 * Provisioning beacon discovered during scan
 */
export interface ProvisioningBeacon {
  mac_address: string
  name: string
  rssi: number
  is_provisioned: boolean
}

/**
 * API response wrapper
 */
interface ApiResponse<T> {
  status: 'success' | 'error'
  message?: string
  beacons?: T
  networks?: T
  result?: T
}

/**
 * Scan for beacons in provisioning mode
 *
 * @returns List of discovered beacons
 */
export async function scanProvisioningBeacons(): Promise<ProvisioningBeacon[]> {
  const response = await invoke<string>('blufi_scan_beacons')
  const parsed: ApiResponse<ProvisioningBeacon[]> = JSON.parse(response)

  if (parsed.status === 'error') {
    throw new Error(parsed.message || 'Failed to scan beacons')
  }

  return parsed.beacons || []
}

/**
 * Connect to a beacon for provisioning
 *
 * @param macAddress - MAC address of the beacon
 */
export async function connectBeacon(macAddress: string): Promise<void> {
  const response = await invoke<string>('blufi_connect', { macAddress })
  const parsed: ApiResponse<void> = JSON.parse(response)

  if (parsed.status === 'error') {
    throw new Error(parsed.message || 'Failed to connect to beacon')
  }
}

/**
 * Scan WiFi networks through connected beacon
 *
 * @returns List of available WiFi networks
 */
export async function scanWifiNetworks(): Promise<WiFiNetwork[]> {
  const response = await invoke<string>('blufi_scan_wifi')
  const parsed: ApiResponse<WiFiNetwork[]> = JSON.parse(response)

  if (parsed.status === 'error') {
    throw new Error(parsed.message || 'Failed to scan WiFi networks')
  }

  return parsed.networks || []
}

/**
 * Provision WiFi credentials to beacon
 *
 * @param config - BluFi configuration
 * @returns Provisioning result
 */
export async function provisionBeacon(
  config: BluFiConfig,
): Promise<BluFiProvisioningResult> {
  const response = await invoke<string>('blufi_provision', {
    config: JSON.stringify(config),
  })
  const parsed: ApiResponse<BluFiProvisioningResult> = JSON.parse(response)

  if (parsed.status === 'error') {
    throw new Error(parsed.message || 'Failed to provision beacon')
  }

  if (!parsed.result) {
    throw new Error('No result returned from provisioning')
  }

  return parsed.result
}

/**
 * Disconnect from beacon
 */
export async function disconnectBeacon(): Promise<void> {
  const response = await invoke<string>('blufi_disconnect')
  const parsed: ApiResponse<void> = JSON.parse(response)

  if (parsed.status === 'error') {
    throw new Error(parsed.message || 'Failed to disconnect from beacon')
  }
}
