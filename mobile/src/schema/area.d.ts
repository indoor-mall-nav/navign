// Auto-generated from Rust schema: area.rs

export interface Area {
  id: string
  entity: string
  name: string
  description: string | null
  /** Unique identifier for the area for displaying in the beacon name. */
  beacon_code: string
  floor: Floor | null
  polygon: [number, number][]
  created_at: number // Timestamp in milliseconds
  updated_at: number // Timestamp in milliseconds
}

export interface Floor {
  type: FloorType
  name: number
}

export type FloorType = 'level' | 'floor' | 'basement'
