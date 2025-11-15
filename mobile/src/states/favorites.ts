import { defineStore } from 'pinia'
import type { Merchant, Area } from '@/schema'

export interface FavoriteMerchant {
  merchantId: string
  merchant: Merchant
  savedAt: number
  notes?: string
}

export interface FavoriteArea {
  areaId: string
  area: Area
  savedAt: number
  label?: string
}

export const useFavoritesStore = defineStore('favorites', {
  state: () => ({
    merchants: [] as FavoriteMerchant[],
    areas: [] as FavoriteArea[],
  }),
  getters: {
    isMerchantFavorited: (state) => (merchantId: string) => {
      return state.merchants.some((fav) => fav.merchantId === merchantId)
    },
    isAreaFavorited: (state) => (areaId: string) => {
      return state.areas.some((fav) => fav.areaId === areaId)
    },
    sortedMerchants: (state) => {
      return [...state.merchants].sort((a, b) => b.savedAt - a.savedAt)
    },
    sortedAreas: (state) => {
      return [...state.areas].sort((a, b) => b.savedAt - a.savedAt)
    },
  },
  actions: {
    addMerchantFavorite(merchant: Merchant, notes?: string) {
      const merchantId = merchant._id?.$oid || ''
      if (!merchantId || this.isMerchantFavorited(merchantId)) {
        return
      }
      this.merchants.push({
        merchantId,
        merchant,
        savedAt: Date.now(),
        notes,
      })
    },
    removeMerchantFavorite(merchantId: string) {
      this.merchants = this.merchants.filter((fav) => fav.merchantId !== merchantId)
    },
    updateMerchantNotes(merchantId: string, notes: string) {
      const favorite = this.merchants.find((fav) => fav.merchantId === merchantId)
      if (favorite) {
        favorite.notes = notes
      }
    },
    addAreaFavorite(area: Area, label?: string) {
      const areaId = area._id?.$oid || ''
      if (!areaId || this.isAreaFavorited(areaId)) {
        return
      }
      this.areas.push({
        areaId,
        area,
        savedAt: Date.now(),
        label,
      })
    },
    removeAreaFavorite(areaId: string) {
      this.areas = this.areas.filter((fav) => fav.areaId !== areaId)
    },
    updateAreaLabel(areaId: string, label: string) {
      const favorite = this.areas.find((fav) => fav.areaId === areaId)
      if (favorite) {
        favorite.label = label
      }
    },
    clearAllFavorites() {
      this.merchants = []
      this.areas = []
    },
  },
  persist: true,
})
