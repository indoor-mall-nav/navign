// API service layer for Tauri commands
import { invoke } from '@tauri-apps/api/core'
import { info } from '@tauri-apps/plugin-log'
import { Merchant } from '@/schema'

export interface ApiResponse<T = any> {
  status: 'success' | 'error'
  data?: T
  message?: string
  [key: string]: any
}

// Login/Authentication APIs
export async function login(
  email: string,
  password: string,
): Promise<ApiResponse> {
  const response = await invoke<string>('login_handler', { email, password })
  return JSON.parse(response)
}

export async function register(
  email: string,
  username: string,
  password: string,
): Promise<ApiResponse> {
  const response = await invoke<string>('register_handler', {
    email,
    username,
    password,
  })
  return JSON.parse(response)
}

export async function logout(token: string): Promise<ApiResponse> {
  const response = await invoke<string>('logout_handler', { token })
  return JSON.parse(response)
}

export async function guestLogin(): Promise<ApiResponse> {
  const response = await invoke<string>('guest_login_handler')
  await info('Message received from guest_login:' + JSON.stringify(response))
  return JSON.parse(response)
}

export async function validateToken(token: string): Promise<ApiResponse> {
  const response = await invoke<string>('validate_token_handler', { token })
  return JSON.parse(response)
}

// Map Display APIs
export interface MapData {
  id: string
  name: string
  polygon: [number, number][]
  beacons: MapBeacon[]
  merchants: MapMerchant[]
}

export interface MapBeacon {
  id: string
  name: string
  location: [number, number]
  type: string
}

export interface MapMerchant {
  id: string
  name: string
  location: [number, number]
  polygon: [number, number][]
  tags: string[]
}

// Route/Navigation APIs
export type RouteInstruction =
  | {
      move: [number, number]
    }
  | {
      transport: [
        string,
        string,
        'stairs' | 'elevator' | 'escalator' | 'gate' | 'turnstile',
      ]
    }

export interface RouteResponse {
  instructions: RouteInstruction[]
  total_distance: number
  areas: string[]
}

export interface ConnectivityLimits {
  elevator: boolean
  stairs: boolean
  escalator: boolean
}

export async function getMapData(
  entity: string,
  area: string,
): Promise<ApiResponse<MapData>> {
  const response = await invoke<string>('get_map_data_handler', {
    entity,
    area,
  })
  return JSON.parse(response)
}

export async function getAllMerchants(
  entity: string,
): Promise<ApiResponse<Merchant[]>> {
  const response = await invoke<string>('get_all_merchants_handler', {
    entity,
  })
  return JSON.parse(response)
}

export async function generateSvgMap(
  entity: string,
  area: string,
  width: number,
  height: number,
): Promise<ApiResponse<{ svg: string }>> {
  const response = await invoke<string>('generate_svg_map_handler', {
    entity,
    area,
    width,
    height,
  })
  return JSON.parse(response)
}

export async function searchMerchants(
  entity: string,
  area: string,
  query: string,
): Promise<ApiResponse> {
  const response = await invoke<string>('search_merchants_handler', {
    entity,
    area,
    query,
  })
  return JSON.parse(response)
}

// Location APIs
export async function locateDevice(
  area: string,
  entity: string,
): Promise<ApiResponse<{ area: string; x: number; y: number }>> {
  const response = await invoke<string>('locate_handler', { area, entity })
  await info('Message received from locate_device:' + JSON.stringify(response))
  return JSON.parse(response)
}

// Unlocker APIs
export async function unlockDevice(
  entity: string,
  target: string,
): Promise<ApiResponse> {
  const response = await invoke<string>('unlock_handler', {
    entity,
    target,
  })
  await info('Message received from unlock_device:' + JSON.stringify(response))
  return JSON.parse(response)
}

export async function bindWithServer(): Promise<ApiResponse> {
  const response = await invoke<string>('bind_with_server')
  return JSON.parse(response)
}

export async function getRoute(
  entity: string,
  from: string,
  to: string,
  limits?: ConnectivityLimits,
): Promise<ApiResponse<RouteResponse>> {
  await info(
    `Requesting route from ${from} to ${to} with limits: ${JSON.stringify(
      limits,
    )}`,
  )
  const response = await invoke<string>('get_route_handler', {
    entity,
    from,
    to,
    allowElevator: limits?.elevator ?? true,
    allowStairs: limits?.stairs ?? true,
    allowEscalator: limits?.escalator ?? true,
  })
  return JSON.parse(response)
}

// Area Details API
export interface AreaDetails {
  _id: string
  entity: string
  name: string
  description: string | null
  beacon_code: string
  floor: {
    type: string
    name: number
  } | null
  polygon: [number, number][]
}

export async function getAreaDetails(
  entity: string,
  area: string,
): Promise<ApiResponse<AreaDetails>> {
  const response = await invoke<string>('get_area_details_handler', {
    entity,
    area,
  })
  return JSON.parse(response)
}

// Merchant Details API
export interface MerchantDetails {
  _id: string
  name: string
  description: string | null
  chain: string | null
  entity: string
  beacon_code: string
  area: string
  location: [number, number]
  polygon: [number, number][] | null
  tags: string[]
  type: any
  style: string
  email: string | null
  phone: string | null
  website: string | null
  social_media: Array<{
    platform: string
    handle: string
    url?: string
  }> | null
}

export async function getMerchantDetails(
  entity: string,
  merchant: string,
): Promise<ApiResponse<MerchantDetails>> {
  const response = await invoke<string>('get_merchant_details_handler', {
    entity,
    merchant,
  })
  return JSON.parse(response)
}
