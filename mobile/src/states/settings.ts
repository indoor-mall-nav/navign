import { defineStore } from 'pinia'

export interface AppSettings {
  theme: 'light' | 'dark' | 'system'
  language: string
  enableBiometric: boolean
  enableNotifications: boolean
  enableLocationTracking: boolean
  autoUpdatePosition: boolean
  positionUpdateInterval: number // in seconds
  showBeaconsOnMap: boolean
  showMerchantsOnMap: boolean
  navigationPreferences: {
    allowStairs: boolean
    allowElevators: boolean
    allowEscalators: boolean
    preferFastestRoute: boolean
  }
  privacy: {
    shareUsageData: boolean
    enableCrashReporting: boolean
  }
}

export const useSettingsStore = defineStore('settings', {
  state: (): AppSettings => ({
    theme: 'system',
    language: 'en',
    enableBiometric: false,
    enableNotifications: true,
    enableLocationTracking: true,
    autoUpdatePosition: true,
    positionUpdateInterval: 5,
    showBeaconsOnMap: true,
    showMerchantsOnMap: true,
    navigationPreferences: {
      allowStairs: true,
      allowElevators: true,
      allowEscalators: true,
      preferFastestRoute: true,
    },
    privacy: {
      shareUsageData: false,
      enableCrashReporting: true,
    },
  }),
  actions: {
    setTheme(theme: 'light' | 'dark' | 'system') {
      this.theme = theme
      this.applyTheme()
    },
    setLanguage(language: string) {
      this.language = language
    },
    toggleBiometric() {
      this.enableBiometric = !this.enableBiometric
    },
    toggleNotifications() {
      this.enableNotifications = !this.enableNotifications
    },
    toggleLocationTracking() {
      this.enableLocationTracking = !this.enableLocationTracking
    },
    toggleAutoUpdatePosition() {
      this.autoUpdatePosition = !this.autoUpdatePosition
    },
    setPositionUpdateInterval(interval: number) {
      this.positionUpdateInterval = interval
    },
    toggleBeaconsOnMap() {
      this.showBeaconsOnMap = !this.showBeaconsOnMap
    },
    toggleMerchantsOnMap() {
      this.showMerchantsOnMap = !this.showMerchantsOnMap
    },
    updateNavigationPreferences(prefs: Partial<AppSettings['navigationPreferences']>) {
      this.navigationPreferences = { ...this.navigationPreferences, ...prefs }
    },
    updatePrivacySettings(privacy: Partial<AppSettings['privacy']>) {
      this.privacy = { ...this.privacy, ...privacy }
    },
    applyTheme() {
      const root = document.documentElement

      if (this.theme === 'dark') {
        root.classList.add('dark')
      } else if (this.theme === 'light') {
        root.classList.remove('dark')
      } else {
        // System preference
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
        if (prefersDark) {
          root.classList.add('dark')
        } else {
          root.classList.remove('dark')
        }
      }
    },
    resetToDefaults() {
      this.$reset()
      this.applyTheme()
    },
  },
  persist: true,
})
