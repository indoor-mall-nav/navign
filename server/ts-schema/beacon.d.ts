// Auto-generated from Rust schema: beacon.rs

export interface ObjectId {
  $oid: string;
}

export interface Beacon {
  _id: ObjectId;
  /** Reference to the Entity */
  entity: ObjectId;
  /** Reference to the Area where the beacon is located */
  area: ObjectId;
  /** Optional reference to the Merchant associated with the beacon. */
  merchant: ObjectId | null;
  /** Optional reference to the Connection associated with the beacon. */
  connection: ObjectId | null;
  /** The ssid of the beacon, typically used for display purposes in BLE scanning.
   * Format:
   * ```
   * BEACON-<area_id>-<beacon_id>
   * ```
   * where `<area_id>` is the ID of the area and `<beacon_id>` is the unique identifier of the beacon.
   * They are incremental value from 0 and, the area id uses 2-byte hex encoding,
   * whereas the beacon id uses 4-byte hex encoding.
   */
  name: string;
  /** The displaying name of the beacon, which can be used for user-friendly identification.
   * This can be the name of the area, merchant, or a custom name.
   */
  description: string | null;
  /** The type of the beacon, which can indicate its purpose or functionality. */
  type: BeaconType;
  /** The location of the beacon, represented as a pair of coordinates (longitude, latitude). */
  location: [number, number];
  device: BeaconDevice;
}

export type BeaconDevice = "esp32" | "esp32c3" | "esp32s3" | "esp32c6";

/** Represents the type of beacon, which can indicate its purpose or functionality. */
export type BeaconType = "navigation" | "marketing";
