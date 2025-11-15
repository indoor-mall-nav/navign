// API client abstraction layer that supports both Tauri and direct HTTP modes
import type { ApiResponse } from './tauri'
import * as tauriApi from './tauri'

// Detect if we're running in Tauri mode
const isTauriMode = (): boolean => {
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    return true
  }
  return false
}

// Get the base URL for HTTP API calls (when not in Tauri mode)
const getBaseUrl = (): string => {
  // Use environment variable or default to localhost
  return import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000'
}

// Get the orchestrator URL for admin CRUD operations
// The orchestrator is the only service allowed to modify the central database
const getOrchestratorUrl = (): string => {
  return import.meta.env.VITE_ORCHESTRATOR_URL || 'http://localhost:8081'
}

// Helper to make HTTP requests to the server
async function httpRequest<T>(
  method: string,
  path: string,
  body?: any,
  token?: string,
): Promise<ApiResponse<T>> {
  const url = `${getBaseUrl()}${path}`
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  }

  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  try {
    const response = await fetch(url, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    })

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      return {
        status: 'error',
        message: errorData.message || `HTTP ${response.status}: ${response.statusText}`,
      }
    }

    const data = await response.json()
    return {
      status: 'success',
      data,
    }
  } catch (error) {
    return {
      status: 'error',
      message: error instanceof Error ? error.message : 'Network error',
    }
  }
}

// Helper to make HTTP requests to the orchestrator (for admin CRUD operations)
async function orchestratorRequest<T>(
  method: string,
  path: string,
  body?: any,
  token?: string,
): Promise<ApiResponse<T>> {
  const url = `${getOrchestratorUrl()}${path}`
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  }

  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  try {
    const response = await fetch(url, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    })

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      return {
        status: 'error',
        message: errorData.message || `HTTP ${response.status}: ${response.statusText}`,
      }
    }

    const data = await response.json()
    return {
      status: 'success',
      data,
    }
  } catch (error) {
    return {
      status: 'error',
      message: error instanceof Error ? error.message : 'Network error',
    }
  }
}

// ============================================================
// Authentication APIs
// ============================================================

export async function login(
  email: string,
  password: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.login(email, password)
  }
  return httpRequest('POST', '/api/auth/login', { username: email, password })
}

export async function register(
  email: string,
  username: string,
  password: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.register(email, username, password)
  }
  return httpRequest('POST', '/api/auth/register', { email, username, password })
}

export async function logout(token: string): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.logout(token)
  }
  return httpRequest('POST', '/api/auth/logout', {}, token)
}

export async function guestLogin(): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.guestLogin()
  }
  return httpRequest('POST', '/api/auth/guest')
}

export async function validateToken(token: string): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.validateToken(token)
  }
  return httpRequest('GET', '/api/auth/validate', undefined, token)
}

// ============================================================
// Beacon CRUD APIs
// ============================================================

export interface BeaconCreateRequest {
  entity: string
  area: string
  merchant?: string | null
  connection?: string | null
  name: string
  description?: string | null
  type: 'navigation' | 'marketing'
  location: [number, number]
  device: 'esp32' | 'esp32c3' | 'esp32s3' | 'esp32c6'
}

export interface BeaconUpdateRequest {
  _id: string
  area?: string
  merchant?: string | null
  connection?: string | null
  name?: string
  description?: string | null
  type?: 'navigation' | 'marketing'
  location?: [number, number]
  device?: 'esp32' | 'esp32c3' | 'esp32s3' | 'esp32c6'
}

export async function listBeacons(
  entityId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.getAllBeacons(entityId)
  }
  return orchestratorRequest('GET', `/api/entities/${entityId}/beacons`, undefined, token)
}

export async function getBeacon(
  entityId: string,
  beaconId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command for getting single beacon
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('GET', `/api/entities/${entityId}/beacons/${beaconId}`, undefined, token)
}

export async function createBeacon(
  entityId: string,
  beacon: BeaconCreateRequest,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('POST', `/api/entities/${entityId}/beacons`, beacon, token)
}

export async function updateBeacon(
  entityId: string,
  beacon: BeaconUpdateRequest,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('PUT', `/api/entities/${entityId}/beacons`, beacon, token)
}

export async function deleteBeacon(
  entityId: string,
  beaconId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('DELETE', `/api/entities/${entityId}/beacons/${beaconId}`, undefined, token)
}

// ============================================================
// Area CRUD APIs
// ============================================================

export interface AreaCreateRequest {
  entity: string
  name: string
  description?: string | null
  beacon_code: string
  floor?: {
    type: 'level' | 'floor' | 'basement'
    name: number
  } | null
  polygon: [number, number][]
}

export interface AreaUpdateRequest {
  _id: string
  name?: string
  description?: string | null
  beacon_code?: string
  floor?: {
    type: 'level' | 'floor' | 'basement'
    name: number
  } | null
  polygon?: [number, number][]
}

export async function listAreas(
  entityId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.getAllAreas(entityId)
  }
  return orchestratorRequest('GET', `/api/entities/${entityId}/areas`, undefined, token)
}

export async function getArea(
  entityId: string,
  areaId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.getAreaDetails(entityId, areaId)
  }
  return orchestratorRequest('GET', `/api/entities/${entityId}/areas/${areaId}`, undefined, token)
}

export async function createArea(
  entityId: string,
  area: AreaCreateRequest,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('POST', `/api/entities/${entityId}/areas`, area, token)
}

export async function updateArea(
  entityId: string,
  area: AreaUpdateRequest,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('PUT', `/api/entities/${entityId}/areas`, area, token)
}

export async function deleteArea(
  entityId: string,
  areaId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('DELETE', `/api/entities/${entityId}/areas/${areaId}`, undefined, token)
}

// ============================================================
// Merchant CRUD APIs
// ============================================================

export interface MerchantCreateRequest {
  entity: string
  area: string
  name: string
  description?: string | null
  chain?: string | null
  beacon_code: string
  type: any // MerchantType from schema
  tags: string[]
  location: [number, number]
  style: 'store' | 'kiosk' | 'popUp' | 'foodTruck' | 'room'
  polygon: [number, number][]
  website?: string | null
  phone?: string
  email?: string | null
  opening_hours?: ([number, number] | [])[]
  images?: string[]
  social_media?: {
    platform: string
    handle: string
    url?: string
  }[]
}

export interface MerchantUpdateRequest {
  _id: string
  area?: string
  name?: string
  description?: string | null
  chain?: string | null
  beacon_code?: string
  type?: any
  tags?: string[]
  location?: [number, number]
  style?: 'store' | 'kiosk' | 'popUp' | 'foodTruck' | 'room'
  polygon?: [number, number][]
  website?: string | null
  phone?: string
  email?: string | null
  opening_hours?: ([number, number] | [])[]
  images?: string[]
  social_media?: {
    platform: string
    handle: string
    url?: string
  }[]
}

export async function listMerchants(
  entityId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.getAllMerchants(entityId)
  }
  return orchestratorRequest('GET', `/api/entities/${entityId}/merchants`, undefined, token)
}

export async function getMerchant(
  entityId: string,
  merchantId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    return tauriApi.getMerchantDetails(entityId, merchantId)
  }
  return orchestratorRequest('GET', `/api/entities/${entityId}/merchants/${merchantId}`, undefined, token)
}

export async function createMerchant(
  entityId: string,
  merchant: MerchantCreateRequest,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('POST', `/api/entities/${entityId}/merchants`, merchant, token)
}

export async function updateMerchant(
  entityId: string,
  merchant: MerchantUpdateRequest,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('PUT', `/api/entities/${entityId}/merchants`, merchant, token)
}

export async function deleteMerchant(
  entityId: string,
  merchantId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('DELETE', `/api/entities/${entityId}/merchants/${merchantId}`, undefined, token)
}

// ============================================================
// Connection CRUD APIs
// ============================================================

export interface ConnectionCreateRequest {
  entity: string
  name: string
  description?: string | null
  type: 'gate' | 'escalator' | 'elevator' | 'stairs' | 'rail' | 'shuttle'
  connected_areas: [string, number, number][]
  available_period: [number, number][]
  tags: string[]
}

export interface ConnectionUpdateRequest {
  _id: string
  name?: string
  description?: string | null
  type?: 'gate' | 'escalator' | 'elevator' | 'stairs' | 'rail' | 'shuttle'
  connected_areas?: [string, number, number][]
  available_period?: [number, number][]
  tags?: string[]
}

export async function listConnections(
  entityId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('GET', `/api/entities/${entityId}/connections`, undefined, token)
}

export async function getConnection(
  entityId: string,
  connectionId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('GET', `/api/entities/${entityId}/connections/${connectionId}`, undefined, token)
}

export async function createConnection(
  entityId: string,
  connection: ConnectionCreateRequest,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('POST', `/api/entities/${entityId}/connections`, connection, token)
}

export async function updateConnection(
  entityId: string,
  connection: ConnectionUpdateRequest,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('PUT', `/api/entities/${entityId}/connections`, connection, token)
}

export async function deleteConnection(
  entityId: string,
  connectionId: string,
  token: string,
): Promise<ApiResponse> {
  if (isTauriMode()) {
    // TODO: Add Tauri command
    return { status: 'error', message: 'Not implemented in Tauri mode' }
  }
  return orchestratorRequest('DELETE', `/api/entities/${entityId}/connections/${connectionId}`, undefined, token)
}

// Re-export existing Tauri API functions that don't need abstraction
export * from './tauri'
