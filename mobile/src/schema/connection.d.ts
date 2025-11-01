// Auto-generated from Rust schema: connection.rs

export interface ObjectId {
  $oid: string
}

export interface Connection {
  _id: ObjectId
  /** Reference to the Entity */
  entity: ObjectId
  name: string
  description: string | null
  type: ConnectionType
  /** List of Area IDs that this connection links
   * Format: Array<[ObjectId, number, number]>
   * where ObjectId is the ID of the area, and number values are coordinates (x, y)
   * representing the connection's position in the area.
   * The coordinates are relative to the area polygon.
   * For example, if the connection is a gate between two areas, the coordinates
   * would represent the position of the gate in the first area.
   * If the connection is a rail or shuttle, the coordinates would represent the
   * position of the rail or shuttle stop in the first area.
   */
  connected_areas: [ObjectId, number, number][]
  /** List of `[start_time, end_time]` in milliseconds on a 24-hour clock */
  available_period: [number, number][]
  tags: string[]
}

/** Represents the type of connection between areas or entities. */
export type ConnectionType =
  | 'gate' // A connection that allows people to pass through, such as a door or gate. Usually involve authentication or access control.
  | 'escalator' // A connection that allows people to move between different areas, such as a hallway or corridor.
  | 'elevator' // A connection that allows people to move between different levels, such as stairs or elevators.
  | 'stairs' // A connection that allows people to move between different areas, such as a pathway or tunnel.
  | 'rail' // Like in Hong Kong International Airport, Singapore Changi Airport, or Shanghai Pudong International Airport.
  | 'shuttle' // Shuttle bus.
