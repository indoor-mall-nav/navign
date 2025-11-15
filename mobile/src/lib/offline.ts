import { ref, computed } from 'vue'
import { info, warn } from '@tauri-apps/plugin-log'

// Offline state management
const isOnline = ref(navigator.onLine)
const lastOnlineTime = ref<number>(Date.now())
const offlineQueue = ref<
  Array<{ action: string; data: any; timestamp: number }>
>([])

// Network status listeners
if (typeof window !== 'undefined') {
  window.addEventListener('online', () => {
    isOnline.value = true
    info('Network: back online')
    processOfflineQueue()
  })

  window.addEventListener('offline', () => {
    isOnline.value = false
    lastOnlineTime.value = Date.now()
    warn('Network: went offline')
  })
}

export function useOffline() {
  const isConnected = computed(() => isOnline.value)
  const offlineDuration = computed(() => {
    if (isOnline.value) return 0
    return Date.now() - lastOnlineTime.value
  })

  /**
   * Add an action to the offline queue to be executed when back online
   */
  function queueAction(action: string, data: any) {
    offlineQueue.value.push({
      action,
      data,
      timestamp: Date.now(),
    })
    info(`Offline queue: added ${action}`)
  }

  /**
   * Process all queued actions when back online
   */
  async function processOfflineQueue() {
    if (offlineQueue.value.length === 0) return

    info(`Offline queue: processing ${offlineQueue.value.length} actions`)

    const queue = [...offlineQueue.value]
    offlineQueue.value = []

    for (const item of queue) {
      try {
        // TODO: Implement action handlers
        info(`Offline queue: processing ${item.action}`)
      } catch (error) {
        warn(`Offline queue: failed to process ${item.action}: ${error}`)
        // Re-queue failed actions
        offlineQueue.value.push(item)
      }
    }
  }

  /**
   * Clear the offline queue
   */
  function clearQueue() {
    offlineQueue.value = []
  }

  /**
   * Check if a specific resource is cached for offline use
   */
  async function isCached(key: string): Promise<boolean> {
    try {
      // TODO: Implement cache check using Tauri SQLite
      return false
    } catch {
      return false
    }
  }

  /**
   * Cache a resource for offline use
   */
  async function cacheResource(key: string, data: any): Promise<void> {
    try {
      // TODO: Implement cache storage using Tauri SQLite
      info(`Cache: stored ${key}`)
    } catch (error) {
      warn(`Cache: failed to store ${key}: ${error}`)
    }
  }

  /**
   * Retrieve a cached resource
   */
  async function getCachedResource(key: string): Promise<any> {
    try {
      // TODO: Implement cache retrieval using Tauri SQLite
      return null
    } catch (error) {
      warn(`Cache: failed to retrieve ${key}: ${error}`)
      return null
    }
  }

  /**
   * Clear all cached resources
   */
  async function clearCache(): Promise<void> {
    try {
      // TODO: Implement cache clearing using Tauri SQLite
      info('Cache: cleared all resources')
    } catch (error) {
      warn(`Cache: failed to clear: ${error}`)
    }
  }

  return {
    isOnline: isConnected,
    offlineDuration,
    queuedActions: computed(() => offlineQueue.value.length),
    queueAction,
    processOfflineQueue,
    clearQueue,
    isCached,
    cacheResource,
    getCachedResource,
    clearCache,
  }
}

/**
 * Retry a network request with exponential backoff
 */
export async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  maxRetries = 3,
  baseDelay = 1000,
): Promise<T> {
  let lastError: Error | null = null

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error as Error
      if (attempt < maxRetries) {
        const delay = baseDelay * Math.pow(2, attempt)
        warn(`Retry attempt ${attempt + 1}/${maxRetries} after ${delay}ms`)
        await new Promise((resolve) => setTimeout(resolve, delay))
      }
    }
  }

  throw lastError
}

/**
 * Execute a function with offline fallback
 */
export async function withOfflineFallback<T>(
  onlineFn: () => Promise<T>,
  offlineFn: () => Promise<T>,
): Promise<T> {
  if (!navigator.onLine) {
    info('Offline: using fallback')
    return await offlineFn()
  }

  try {
    return await onlineFn()
  } catch (error) {
    warn(`Network error: falling back to offline mode: ${error}`)
    return await offlineFn()
  }
}
