// Auto-generated from Rust schema: merchant.rs

export interface ObjectId {
  $oid: string
}

export interface Merchant {
  _id: ObjectId
  name: string
  description: string | null
  chain: string | null // Name of the chain if part of a chain store series
  entity: ObjectId // Reference to the Entity
  beacon_code: string // Unique identifier for the merchant for displaying in the beacon name
  area: ObjectId
  type: MerchantType
  /** List of tags for categorization, e.g., "food", "electronics", "clothing"
   * Tags can be used for search and filtering
   */
  tags: string[]
  location: [number, number]
  style: MerchantStyle
  polygon: [number, number][]
  website?: string | null
  phone?: string
  email?: string | null
  opening_hours: ([number, number] | [])[] // milliseconds from midnight, e.g., [[36000000, 72000000]] for 10:00-20:00
  images: string[] // URLs to images
  social_media: {
    platform:
      | 'facebook'
      | 'instagram'
      | 'twitter'
      | 'linkedin'
      | 'tiktok'
      | 'wechat'
      | 'weibo'
      | 'rednote'
      | 'bluesky'
      | 'reddit'
      | 'discord'
      | 'whatsapp'
      | 'telegram'
      | string
    handle: string // e.g., "@merchant" or "merchantPage"
    url?: string // Full URL to the social media page
  }[]
}

export type MerchantType =
  | { food: { cuisine: FoodCuisine | null; type: FoodType } }
  | {
      electronics: {
        is_mobile: boolean
        is_computer: boolean
        is_accessories: boolean
      }
    }
  | {
      clothing: {
        is_menswear: boolean
        is_womenswear: boolean
        is_childrenswear: boolean
      }
    }
  | 'supermarket'
  | 'health'
  | 'entertainment'
  | 'service'
  | 'room' // The room is, for example, a hotel room, office room, or meeting room.
  | 'other' // For any other type not listed

export type FoodType =
  | { restaurant: FoodCuisine }
  | 'cafe'
  | 'fastFood'
  | 'bakery'
  | 'bar'
  | 'other' // For any other type not listed

export type FoodCuisine =
  | 'italian'
  | { chinese: { cuisine: ChineseFoodCuisine; specific: string | null } } // Specific dish or style, e.g., "Dim Sum", "Ningbo Cuisine"
  | 'indian'
  | 'american'
  | 'japanese'
  | 'korean'
  | 'french'
  | 'thai'
  | 'mexican'
  | 'mediterranean'
  | 'spanish'
  | 'vietnamese'
  | 'middleEastern'
  | 'african'
  | { other: string } // For any other type not listed

export type ChineseFoodCuisine =
  | 'cantonese'
  | 'sichuan'
  | 'hunan'
  | 'jiangxi'
  | 'shanghai'
  | 'hangzhou'
  | 'ningbo'
  | 'northern'
  | 'other' // For any other type not listed

export type MerchantStyle = 'store' | 'kiosk' | 'popUp' | 'foodTruck' | 'room'
