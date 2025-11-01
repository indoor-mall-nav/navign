// Auto-generated from Rust schema: area.rs

export interface ObjectId {
  $oid: string
}

export interface Area {
  _id: ObjectId
  entity: ObjectId
  name: string
  description: string | null
  /** Unique identifier for the area for displaying in the beacon name. */
  beacon_code: string
  floor: Floor | null
  polygon: [number, number][]
}

export interface Floor {
  type: FloorType
  name: number
}

export type FloorType = 'level' | 'floor' | 'basement'
