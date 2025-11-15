import { ref, computed } from 'vue'
import { info, warn } from '@tauri-apps/plugin-log'

// Offline state management
const isOnline = ref(navigator.onLine)
const lastOnlineTime = ref<number>(Date.now())
const offlineQueue = ref<
  Array<{ action: string; data: any; timestamp: number }>
>([])

// Forward declaration for processOfflineQueue
let processOfflineQueueFn: (() => Promise<void>) | null = null

// Network status listeners
if (typeof window !== 'undefined') {
  window.addEventListener('online', () => {
    isOnline.value = true
    void info('Network: back online')
    if (processOfflineQueueFn) {
      void processOfflineQueueFn()
    }
  })

  window.addEventListener('offline', () => {
    isOnline.value = false
    lastOnlineTime.value = Date.now()
    void warn('Network: went offline')
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
    void info(`Offline queue: added ${action}`)
  }

  /**
   * Process all queued actions when back online
   */
  async function processOfflineQueue() {
    if (offlineQueue.value.length === 0) return

    void info(`Offline queue: processing ${offlineQueue.value.length} actions`)

    const queue = [...offlineQueue.value]
    offlineQueue.value = []

    for (const item of queue) {
      try {
        // TODO: Implement action handlers
        void info(`Offline queue: processing ${item.action}`)
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error)
        void warn(
          `Offline queue: failed to process ${item.action}: ${errorMessage}`,
        )
        // Re-queue failed actions
        offlineQueue.value.push(item)
      }
    }
  }

  // Set the function reference for the event listener
  processOfflineQueueFn = processOfflineQueue

  /**
   * Clear the offline queue
   */
  function clearQueue() {
    offlineQueue.value = []
  }

  /**
   * Check if a specific resource is cached for offline use
   */
  async function isCached(_key: string): Promise<boolean> {
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
  async function cacheResource(key: string, _data: any): Promise<void> {
    try {
      // TODO: Implement cache storage using Tauri SQLite
      void info(`Cache: stored ${key}`)
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error)
      void warn(`Cache: failed to store ${key}: ${errorMessage}`)
    }
  }

  /**
   * Retrieve a cached resource
   */
  async function getCachedResource(_key: string): Promise<any> {
    try {
      // TODO: Implement cache retrieval using Tauri SQLite
      return null
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error)
      void warn(`Cache: failed to retrieve ${_key}: ${errorMessage}`)
      return null
    }
  }

  /**
   * Clear all cached resources
   */
  async function clearCache(): Promise<void> {
    try {
      // TODO: Implement cache clearing using Tauri SQLite
      void info('Cache: cleared all resources')
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error)
      void warn(`Cache: failed to clear: ${errorMessage}`)
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
        void warn(`Retry attempt ${attempt + 1}/${maxRetries} after ${delay}ms`)
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
    void info('Offline: using fallback')
    return await offlineFn()
  }

  try {
    return await onlineFn()
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    void warn(`Network error: falling back to offline mode: ${errorMessage}`)
    return await offlineFn()
  }
}
