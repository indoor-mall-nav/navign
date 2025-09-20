import { ObjectId } from "@/schema";

/**
 * Protocol definitions for the Unlocker service.
 *
 * `0x01`: Device inquiry (no payload)
 * `0x02`: Device response (12-byte MongoDB ObjectId)
 * `0x03`: Nonce request (no payload)
 * `0x04`: Nonce response (16-byte nonce)
 * `0x05`: Proof submission (32-byte challenge hash + 64-byte device signature + 8-byte timestamp + 8-byte counter)
 * `0x06`: Unlock command (1-byte unlock flag: 0x01 for unlock, 0x00 for lock; 1-byte status code)
 *
 * #[derive(Debug, Clone, Copy)]
 * pub enum CryptoError {
 *     InvalidSignature,
 *     InvalidKey,
 *     InvalidNonce,
 *     VerificationFailed,
 *     BufferFull,
 *     RateLimited,
 *     ReplayDetected,
 *     ServerPublicKeyNotSet,
 * }
 *
 * service: 134b1d88-cd91-8134-3e94-5c4052743845
 * device char: 99d92823-9e38-72ff-6cf1-d2d593316af8
 * nonce char: 49e595a0-3e9a-4831-8a3d-c63818783144
 * proof char: 9f3e943e-153e-23e2-9d5e-3f0da83edc6f
 * unlock char: d2b0f2e4-6c3a-4e5f-8e1d-7f4b6c8e9a0b
 *
 * ALWAYS USE BIG ENDIAN FOR MULTI-BYTE VALUES
 */
export enum UnlockerProtocol {
  DEVICE_REQUEST = 0x01,
  DEVICE_RESPONSE = 0x02,
  NONCE_REQUEST = 0x03,
  NONCE_RESPONSE = 0x04,
  PROOF_SUBMISSION = 0x05,
  UNLOCK_COMMAND = 0x06,
}

export enum UnlockerStatus {
  SUCCESS = 0x00,
  FAILURE = 0x01,
}

export enum UnlockerError {
  SUCCESS = 0x00,
  INVALID_SIGNATURE = 0x01,
  INVALID_KEY = 0x02,
  INVALID_NONCE = 0x03,
  VERIFICATION_FAILED = 0x04,
  BUFFER_FULL = 0x05,
  RATE_LIMITED = 0x06,
  REPLAY_DETECTED = 0x07,
  SERVER_PUBLIC_KEY_NOT_SET = 0x08,
}

export enum DeviceCapability {
  UNLOCK_GATE = 1 << 0,
  ENVIRONMENTAL_DATA = 1 << 1,
  RSSI_CALIBRATION = 1 << 2,
}

export enum DeviceType {
  MERCHANT,
  PATHWAY,
  CONNECTION,
  TURNSTILE,
}

export interface InquiryResult {
  type: DeviceType;
  capabilities: DeviceCapability[];
  id: ObjectId;
}

export const UNLOCKER_SERVICE_UUID = "134b1d88-cd91-8134-3e94-5c4052743845";
export const UNLOCKER_CHARACTERISTIC_UUID =
  "99d92823-9e38-72ff-6cf1-d2d593316af8";

// Example function to create a nonce request packet
export function createNonceRequestPacket(): Uint8Array {
  return new Uint8Array([UnlockerProtocol.NONCE_REQUEST]);
}

// Example function to parse a nonce response packet
export function parseNonceResponsePacket(data: Uint8Array): Uint8Array | null {
  console.log("Nonce response received:", data, "with length", data.length);
  console.log("Checking data", data[0], data[0] === UnlockerProtocol.NONCE_RESPONSE, data.length);
  if (data[0] === UnlockerProtocol.NONCE_RESPONSE && data.length >= 17) {
    return data.slice(1, 17);
  }
  return null;
}

export function parseInquiryResponsePacket(
  data: Uint8Array,
): InquiryResult | null {
  if (data[0] === UnlockerProtocol.DEVICE_RESPONSE && data.length >= 27) {
    const type = data[1] as DeviceType;
    const capabilitiesByte = data[2];
    const capabilities: DeviceCapability[] = [];
    if (capabilitiesByte & DeviceCapability.UNLOCK_GATE) {
      capabilities.push(DeviceCapability.UNLOCK_GATE);
    }
    if (capabilitiesByte & DeviceCapability.ENVIRONMENTAL_DATA) {
      capabilities.push(DeviceCapability.ENVIRONMENTAL_DATA);
    }
    if (capabilitiesByte & DeviceCapability.RSSI_CALIBRATION) {
      capabilities.push(DeviceCapability.RSSI_CALIBRATION);
    }
    const objectIdBytes = data.slice(3, 27);
    const objectId = {
      $oid: Array.from(objectIdBytes)
        .map((b) => b.toString(16).padStart(2, "0"))
        .join(""),
    };
    return {
      type,
      capabilities,
      id: objectId,
    } as InquiryResult;
  }
  return null;
}

// Example function to parse an unlock command response packet
export function parseUnlockCommandResponsePacket(
  data: Uint8Array,
): UnlockerError | null {
  if (data[0] === UnlockerProtocol.UNLOCK_COMMAND && data.length >= 2) {
    return data[2] as UnlockerError;
  }
  return null;
}

// Example function to create a proof submission packet
export function createProofSubmissionPacket(
  challengeHash: Uint8Array,
  deviceSignature: Uint8Array,
  timestamp: bigint,
  counter: bigint,
): Uint8Array {
  if (challengeHash.length !== 32 || deviceSignature.length !== 64) {
    throw new Error("Invalid challenge hash or device signature length");
  }

  const packet = new Uint8Array(1 + 32 + 64 + 8 + 8);
  packet[0] = UnlockerProtocol.PROOF_SUBMISSION;
  packet.set(challengeHash, 1);
  packet.set(deviceSignature, 33);

  const timestampView = new DataView(packet.buffer, 97, 8);
  timestampView.setBigUint64(0, timestamp, false); // Big endian

  const counterView = new DataView(packet.buffer, 105, 8);
  counterView.setBigUint64(0, counter, false); // Big endian
  return packet;
}

export function renderUnlockerError(error: UnlockerError): string {
  switch (error) {
    case UnlockerError.SUCCESS:
      return "Success";
    case UnlockerError.INVALID_SIGNATURE:
      return "Invalid Signature";
    case UnlockerError.INVALID_KEY:
      return "Invalid Key";
    case UnlockerError.INVALID_NONCE:
      return "Invalid Nonce";
    case UnlockerError.VERIFICATION_FAILED:
      return "Verification Failed";
    case UnlockerError.BUFFER_FULL:
      return "Buffer Full";
    case UnlockerError.RATE_LIMITED:
      return "Rate Limited";
    case UnlockerError.REPLAY_DETECTED:
      return "Replay Detected";
    case UnlockerError.SERVER_PUBLIC_KEY_NOT_SET:
      return "Server Public Key Not Set";
    default:
      return "Unknown Error";
  }
}
