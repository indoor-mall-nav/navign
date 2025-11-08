import { vi } from 'vitest'

// Mock Tauri API
vi.mock('@tauri-apps/api', () => ({
  invoke: vi.fn(),
}))

// Mock Tauri plugins
vi.mock('@tauri-apps/plugin-log', () => ({
  info: vi.fn(),
  warn: vi.fn(),
  error: vi.fn(),
  debug: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-sql', () => ({
  default: vi.fn(() => ({
    execute: vi.fn(),
    select: vi.fn(),
  })),
}))

vi.mock('@mnlphlp/plugin-blec', () => ({
  scan: vi.fn(),
  connect: vi.fn(),
  disconnect: vi.fn(),
}))

// Mock window.matchMedia for responsive components
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})
