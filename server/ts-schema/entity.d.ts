// Auto-generated from Rust schema: entity.rs

export interface ObjectId {
  $oid: string
}

export interface Entity {
  _id: ObjectId
  type: EntityType
  name: string
  description: string | null
  longitude_range: [number, number] // [min_longitude, max_longitude]
  latitude_range: [number, number] // [min_latitude, max_latitude]
  altitude_range: [number, number] | null // [min_altitude, max_altitude]
  nation: string | null
  region: string | null
  city: string | null
  tags: string[]
  created_at: number // Timestamp in milliseconds
  updated_at: number // Timestamp in milliseconds
}

export type EntityType = 'mall' | 'transportation' | 'school' | 'hospital'
