import { info, warn } from '@tauri-apps/plugin-log'

/**
 * Biometric authentication types
 */
export type BiometricType = 'fingerprint' | 'face' | 'iris' | 'none'

/**
 * Biometric authentication status
 */
export interface BiometricStatus {
  available: boolean
  enrolled: boolean
  type: BiometricType
}

/**
 * Check if biometric authentication is available on the device
 */
export async function checkBiometricAvailability(): Promise<BiometricStatus> {
  try {
    // TODO: Implement using tauri-plugin-biometric
    // const { authenticate } = await import('@tauri-apps/plugin-biometric')
    // const status = await authenticate.status()

    await info('Biometric: checking availability')

    // Placeholder implementation
    return {
      available: false,
      enrolled: false,
      type: 'none',
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    await warn(`Biometric: availability check failed: ${errorMessage}`)
    return {
      available: false,
      enrolled: false,
      type: 'none',
    }
  }
}

/**
 * Authenticate using biometric
 */
export async function authenticateWithBiometric(
  reason: string = 'Authenticate to continue',
): Promise<boolean> {
  try {
    // TODO: Implement using tauri-plugin-biometric
    // const { authenticate } = await import('@tauri-apps/plugin-biometric')
    // const result = await authenticate({
    //   reason,
    //   cancelTitle: 'Cancel',
    //   fallbackTitle: 'Use Password',
    // })

    await info(`Biometric: authentication requested - ${reason}`)

    // Placeholder implementation
    return false
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    await warn(`Biometric: authentication failed: ${errorMessage}`)
    return false
  }
}

/**
 * Store credentials with biometric protection
 */
export async function storeBiometricCredentials(
  key: string,
  _value: string,
): Promise<boolean> {
  try {
    // TODO: Implement using tauri-plugin-stronghold
    // const { Stronghold } = await import('@tauri-apps/plugin-stronghold')
    // const stronghold = await Stronghold.load('credentials.hold')
    // await stronghold.insert(key, _value)

    await info(`Biometric: stored credentials for ${key}`)
    return true
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    await warn(`Biometric: failed to store credentials: ${errorMessage}`)
    return false
  }
}

/**
 * Retrieve credentials with biometric authentication
 */
export async function retrieveBiometricCredentials(
  key: string,
): Promise<string | null> {
  try {
    // First authenticate
    const authenticated = await authenticateWithBiometric(
      'Access your saved credentials',
    )

    if (!authenticated) {
      await warn('Biometric: authentication required to retrieve credentials')
      return null
    }

    // TODO: Implement using tauri-plugin-stronghold
    // const { Stronghold } = await import('@tauri-apps/plugin-stronghold')
    // const stronghold = await Stronghold.load('credentials.hold')
    // const _value = await stronghold.get(key)

    await info(`Biometric: retrieved credentials for ${key}`)
    return null // Placeholder
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    await warn(`Biometric: failed to retrieve credentials: ${errorMessage}`)
    return null
  }
}

/**
 * Remove stored credentials
 */
export async function removeBiometricCredentials(
  key: string,
): Promise<boolean> {
  try {
    // TODO: Implement using tauri-plugin-stronghold
    // const { Stronghold } = await import('@tauri-apps/plugin-stronghold')
    // const stronghold = await Stronghold.load('credentials.hold')
    // await stronghold.remove(key)

    await info(`Biometric: removed credentials for ${key}`)
    return true
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    await warn(`Biometric: failed to remove credentials: ${errorMessage}`)
    return false
  }
}

/**
 * Enable biometric login for a user
 */
export async function enableBiometricLogin(
  username: string,
  token: string,
): Promise<boolean> {
  try {
    const status = await checkBiometricAvailability()

    if (!status.available) {
      await warn('Biometric: not available on this device')
      return false
    }

    if (!status.enrolled) {
      await warn('Biometric: no biometric credentials enrolled')
      return false
    }

    // Authenticate first
    const authenticated = await authenticateWithBiometric(
      'Enable biometric login',
    )

    if (!authenticated) {
      return false
    }

    // Store credentials
    const stored = await storeBiometricCredentials(`login:${username}`, token)

    if (stored) {
      await info(`Biometric: enabled biometric login for ${username}`)
      return true
    }

    return false
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    await warn(`Biometric: failed to enable biometric login: ${errorMessage}`)
    return false
  }
}

/**
 * Login using biometric authentication
 */
export async function loginWithBiometric(
  username: string,
): Promise<string | null> {
  try {
    const status = await checkBiometricAvailability()

    if (!status.available) {
      await warn('Biometric: not available on this device')
      return null
    }

    // Retrieve stored token with biometric auth
    const token = await retrieveBiometricCredentials(`login:${username}`)

    if (token) {
      await info(`Biometric: successful biometric login for ${username}`)
      return token
    }

    return null
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    await warn(`Biometric: login failed: ${errorMessage}`)
    return null
  }
}

/**
 * Disable biometric login
 */
export async function disableBiometricLogin(
  username: string,
): Promise<boolean> {
  try {
    const removed = await removeBiometricCredentials(`login:${username}`)

    if (removed) {
      await info(`Biometric: disabled biometric login for ${username}`)
      return true
    }

    return false
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    await warn(`Biometric: failed to disable biometric login: ${errorMessage}`)
    return false
  }
}
