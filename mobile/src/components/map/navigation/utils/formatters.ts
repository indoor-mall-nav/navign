/**
 * Utility functions for formatting navigation data
 */

/**
 * Format distance in meters to human-readable string
 * @param meters - Distance in meters
 * @returns Formatted distance string (e.g., "50 cm", "150 m", "1.5 km")
 */
export function formatDistance(meters: number): string {
  if (meters < 1) {
    return `${Math.round(meters * 100)} cm`
  } else if (meters < 1000) {
    return `${Math.round(meters)} m`
  } else {
    return `${(meters / 1000).toFixed(2)} km`
  }
}

/**
 * Format time in seconds to human-readable string
 * @param seconds - Time in seconds
 * @returns Formatted time string (e.g., "30 sec", "5 min", "1h 30m")
 */
export function formatTime(seconds: number): string {
  if (seconds < 60) {
    return `${Math.round(seconds)} sec`
  } else if (seconds < 3600) {
    const minutes = Math.floor(seconds / 60)
    const remainingSeconds = Math.round(seconds % 60)
    return remainingSeconds > 0 ? `${minutes} min ${remainingSeconds} sec` : `${minutes} min`
  } else {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`
  }
}

/**
 * Calculate estimated time based on distance (assuming average walking speed)
 * @param meters - Distance in meters
 * @param walkingSpeedMps - Walking speed in meters per second (default: 1.4 m/s = 5 km/h)
 * @returns Estimated time in seconds
 */
export function estimateWalkingTime(meters: number, walkingSpeedMps: number = 1.4): number {
  return meters / walkingSpeedMps
}
