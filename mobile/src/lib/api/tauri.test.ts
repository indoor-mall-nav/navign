// Comprehensive unit tests for Tauri API integration
import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  login,
  register,
  guestLogin,
  logout,
  getMapData,
  generateSvgMap,
  searchMerchants,
  locateDevice,
  getRoute,
  type ApiResponse,
  type ConnectivityLimits,
} from './tauri'
import { invoke } from '@tauri-apps/api/core'
import { error, info } from '@tauri-apps/plugin-log'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-log', () => ({
  info: vi.fn(),
  error: vi.fn(),
}))

describe('Authentication APIs', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should login successfully with valid credentials', async () => {
    const mockResponse: ApiResponse = {
      status: 'success',
      token: 'test_token_123',
      message: 'Login successful',
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockResponse))
    ;(info as any).mockResolvedValue(undefined)
    ;(error as any).mockResolvedValue(undefined)

    const result = await login('test@example.com', 'password123')

    expect(result.status).toBe('success')
    expect(result.token).toBe('test_token_123')
    expect(invoke).toHaveBeenCalledWith('login_handler', {
      email: 'test@example.com',
      password: 'password123',
    })
  })

  it('should handle login failure', async () => {
    const mockResponse: ApiResponse = {
      status: 'error',
      message: 'Invalid credentials',
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockResponse))

    const result = await login('wrong@example.com', 'wrongpass')

    expect(result.status).toBe('error')
    expect(result.message).toBe('Invalid credentials')
  })

  it('should register new user successfully', async () => {
    const mockResponse: ApiResponse = {
      status: 'success',
      user_id: 'new_user_456',
      message: 'Registration successful',
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockResponse))

    const result = await register('new@example.com', 'newuser', 'securepass')

    expect(result.status).toBe('success')
    expect(result.user_id).toBe('new_user_456')
  })

  it('should allow guest login', async () => {
    const mockResponse: ApiResponse = {
      status: 'success',
      user_id: 'guest_789',
      message: 'Logged in as guest',
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockResponse))

    const result = await guestLogin()

    expect(result.status).toBe('success')
    expect(result.user_id).toContain('guest')
  })

  it('should logout user', async () => {
    const mockResponse: ApiResponse = {
      status: 'success',
      message: 'Logged out successfully',
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockResponse))

    const result = await logout('test_token')

    expect(result.status).toBe('success')
    expect(invoke).toHaveBeenCalledWith('logout_handler', {
      token: 'test_token',
    })
  })
})

describe('Map APIs', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should fetch map data successfully', async () => {
    const mockMapData = {
      status: 'success',
      data: {
        id: 'area_123',
        name: 'Main Hall',
        polygon: [
          [0, 0],
          [100, 0],
          [100, 100],
          [0, 100],
        ],
        beacons: [
          {
            id: 'beacon_1',
            name: 'Beacon A',
            location: [50, 50],
            type: 'navigation',
          },
        ],
        merchants: [],
      },
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockMapData))

    const result = await getMapData('entity_1', 'area_123')

    expect(result.status).toBe('success')
    expect(result.data?.name).toBe('Main Hall')
    expect(result.data?.beacons).toHaveLength(1)
  })

  it('should generate SVG map', async () => {
    const mockSvgResponse = {
      status: 'success',
      svg: '<svg width="800" height="600">...</svg>',
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockSvgResponse))

    const result = await generateSvgMap('entity_1', 'area_123', 800, 600)

    expect(result.status).toBe('success')
    expect(result.svg).toContain('<svg')
    expect(invoke).toHaveBeenCalledWith('generate_svg_map_handler', {
      entity: 'entity_1',
      area: 'area_123',
      width: 800,
      height: 600,
    })
  })

  it('should search merchants by query', async () => {
    const mockSearchResponse = {
      status: 'success',
      data: [
        { id: 'm1', name: 'Coffee Shop', tags: ['food', 'cafe'] },
        { id: 'm2', name: 'Bookstore', tags: ['retail', 'books'] },
      ],
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockSearchResponse))

    const result = await searchMerchants('entity_1', 'area_123', 'coffee')

    expect(result.status).toBe('success')
    expect(result.data).toHaveLength(2)
  })
})

describe('Location APIs', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should locate device successfully', async () => {
    const mockLocationResponse = {
      status: 'success',
      area: 'area_123',
      x: 45.5,
      y: 67.8,
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockLocationResponse))
    ;(info as any).mockResolvedValue(undefined)
    ;(error as any).mockResolvedValue(undefined)

    const result = await locateDevice('area_123', 'entity_1')

    expect(result.status).toBe('success')
    expect(result.x).toBe(45.5)
    expect(result.y).toBe(67.8)
  })

  it('should handle location failure', async () => {
    const mockErrorResponse = {
      status: 'error',
      message: 'No beacons found',
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockErrorResponse))
    ;(info as any).mockResolvedValue(undefined)
    ;(error as any).mockResolvedValue(undefined)

    const result = await locateDevice('area_123', 'entity_1')

    expect(result.status).toBe('error')
    expect(result.message).toContain('beacons')
  })
})

describe('Route/Navigation APIs', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should handle route not found', async () => {
    const mockErrorResponse: ApiResponse = {
      status: 'error',
      message: 'No route found between merchants',
    }

    ;(invoke as any).mockResolvedValue(JSON.stringify(mockErrorResponse))

    const result = await getRoute('entity_1', 'merchant_a', 'merchant_z')

    expect(result.status).toBe('error')
    expect(result.message).toContain('No route found')
  })
})

describe('API Error Handling', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should handle network errors gracefully', async () => {
    ;(invoke as any).mockRejectedValue(new Error('Network error'))

    await expect(login('test@test.com', 'pass')).rejects.toThrow(
      'Network error',
    )
  })

  it('should handle malformed JSON responses', async () => {
    ;(invoke as any).mockResolvedValue('invalid json {')

    await expect(getMapData('e1', 'a1')).rejects.toThrow()
  })

  it('should handle empty responses', async () => {
    ;(invoke as any).mockResolvedValue('')

    await expect(locateDevice('a1', 'e1')).rejects.toThrow()
  })
})

describe('Type Safety', () => {
  it('should have correct types for ConnectivityLimits', () => {
    const limits: ConnectivityLimits = {
      elevator: true,
      stairs: false,
      escalator: true,
    }

    expect(typeof limits.elevator).toBe('boolean')
    expect(typeof limits.stairs).toBe('boolean')
    expect(typeof limits.escalator).toBe('boolean')
  })
})
