import { defineStore } from 'pinia'
import type { Merchant, Area } from '@/schema'

export interface NavigationHistoryEntry {
  id: string
  from: {
    areaId: string
    areaName: string
    position: { x: number; y: number }
  }
  to: {
    merchantId?: string
    merchantName?: string
    areaId?: string
    areaName?: string
    position: { x: number; y: number }
  }
  timestamp: number
  duration?: number // in seconds
  distance?: number // in meters
  completed: boolean
}

export interface SearchHistoryEntry {
  query: string
  timestamp: number
  resultCount: number
}

export const useHistoryStore = defineStore('history', {
  state: () => ({
    navigationHistory: [] as NavigationHistoryEntry[],
    searchHistory: [] as SearchHistoryEntry[],
    maxNavigationHistory: 50,
    maxSearchHistory: 20,
  }),
  getters: {
    recentNavigations: (state) => {
      return [...state.navigationHistory]
        .sort((a, b) => b.timestamp - a.timestamp)
        .slice(0, 10)
    },
    recentSearches: (state) => {
      return [...state.searchHistory]
        .sort((a, b) => b.timestamp - a.timestamp)
        .slice(0, 10)
    },
    completedNavigations: (state) => {
      return state.navigationHistory.filter((entry) => entry.completed)
    },
    frequentDestinations: (state) => {
      const destinations = new Map<string, { count: number; lastVisit: number; name: string }>()

      state.navigationHistory.forEach((entry) => {
        if (entry.completed && entry.to.merchantId) {
          const key = entry.to.merchantId
          const existing = destinations.get(key)
          if (existing) {
            existing.count++
            existing.lastVisit = Math.max(existing.lastVisit, entry.timestamp)
          } else {
            destinations.set(key, {
              count: 1,
              lastVisit: entry.timestamp,
              name: entry.to.merchantName || '',
            })
          }
        }
      })

      return Array.from(destinations.entries())
        .map(([id, data]) => ({ id, ...data }))
        .sort((a, b) => b.count - a.count)
        .slice(0, 5)
    },
  },
  actions: {
    addNavigationEntry(entry: Omit<NavigationHistoryEntry, 'id'>) {
      const id = `nav_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
      this.navigationHistory.push({ id, ...entry })

      // Keep only the most recent entries
      if (this.navigationHistory.length > this.maxNavigationHistory) {
        this.navigationHistory = this.navigationHistory
          .sort((a, b) => b.timestamp - a.timestamp)
          .slice(0, this.maxNavigationHistory)
      }
    },
    updateNavigationEntry(id: string, updates: Partial<NavigationHistoryEntry>) {
      const entry = this.navigationHistory.find((e) => e.id === id)
      if (entry) {
        Object.assign(entry, updates)
      }
    },
    markNavigationCompleted(id: string, duration?: number, distance?: number) {
      const entry = this.navigationHistory.find((e) => e.id === id)
      if (entry) {
        entry.completed = true
        if (duration) entry.duration = duration
        if (distance) entry.distance = distance
      }
    },
    addSearchEntry(query: string, resultCount: number) {
      // Avoid duplicates within the last hour
      const oneHourAgo = Date.now() - 60 * 60 * 1000
      const existing = this.searchHistory.find(
        (entry) => entry.query === query && entry.timestamp > oneHourAgo
      )

      if (!existing) {
        this.searchHistory.push({
          query,
          timestamp: Date.now(),
          resultCount,
        })

        // Keep only the most recent entries
        if (this.searchHistory.length > this.maxSearchHistory) {
          this.searchHistory = this.searchHistory
            .sort((a, b) => b.timestamp - a.timestamp)
            .slice(0, this.maxSearchHistory)
        }
      }
    },
    clearNavigationHistory() {
      this.navigationHistory = []
    },
    clearSearchHistory() {
      this.searchHistory = []
    },
    clearAllHistory() {
      this.navigationHistory = []
      this.searchHistory = []
    },
    deleteNavigationEntry(id: string) {
      this.navigationHistory = this.navigationHistory.filter((entry) => entry.id !== id)
    },
  },
  persist: true,
})
