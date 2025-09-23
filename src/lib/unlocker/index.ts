import { type BleDevice, connect, read, send, disconnect } from "@mnlphlp/plugin-blec";
import { invoke } from "@tauri-apps/api/core";
import * as protocol from "./protocol";
import { DeviceCapability, mergeInquiryResults } from "./protocol";

export interface Challenge {
  challengeHash: Uint8Array;
  deviceSignature: Uint8Array;
  timestamp: bigint;
  counter: bigint;
}

function uint8ArrayToBase64(uint8Array: Uint8Array): string {
  if ("toBase64" in uint8Array) {
    // @ts-ignore
    return uint8Array.toBase64();
  }
  // Convert Uint8Array to a binary string
  const binaryString = String.fromCharCode.apply(null, Array.from(uint8Array));

  // Encode the binary string to base64
  const base64String = btoa(binaryString);

  return base64String;
}

/**
 * #[derive(Debug, Serialize, Deserialize)]
 * pub struct DeviceProof {
 *     pub challenge_hash: [u8; 32],
 *     #[serde(with = "BigArray")]
 *     pub device_signature: [u8; 64],
 *     pub timestamp: u64,
 *     pub counter: u64,
 * }
 */
export interface DeviceProof {
  challenge_hash: Uint8Array;
  device_signature: Uint8Array;
  timestamp: number;
  counter: number;
}

export async function unlockDevice(device: BleDevice, entity: string) {
  try {
    try {await disconnect();} catch(_) {}
    if (device.name !== "NAVIGN-BEACON") {
      throw new Error("Unsupported device");
    }

    if (device.rssi < -70) {
      throw new Error("Device is out of range");
    }

    // Stage 0: Connect to the device
    console.log("Connecting to device:", device);

    await connect(device.address, () => {
      console.log("Device disconnected");
    });
    console.log("Connected to device");

    // Stage 1: Inquire device about its ObjectId.
    console.log(
      "Sending device inquiry request...",
      protocol.UNLOCKER_CHARACTERISTIC_UUID,
    );

    const polls = [0x00, 0x01];

    const results = [];

    for await (let poll of polls) {
      const deviceInquiry = new Uint8Array([0x01, poll]);
      await send(
        protocol.UNLOCKER_CHARACTERISTIC_UUID,
        deviceInquiry,
        "withoutResponse",
        protocol.UNLOCKER_SERVICE_UUID,
      );
      console.log("Inquiry request sent");
      const inquiryResult = await read(
        protocol.UNLOCKER_CHARACTERISTIC_UUID,
        protocol.UNLOCKER_SERVICE_UUID,
      );
      console.log("Inquiry response received:", inquiryResult);
      const result = protocol.parseInquiryResponsePacket(inquiryResult);
      if (!result) {
        throw new Error("Invalid inquiry response");
      }
      results.push(result);
    }

    const result = mergeInquiryResults(results);

    if (!result) {
      throw new Error("Failed to merge inquiry results");
    }

    console.log("Inquiry result:", result);
    const objectId = result.id.$oid;
    if (!result.capabilities.includes(DeviceCapability.UNLOCK_GATE)) {
      throw new Error("Device does not support unlocking");
    }

    // Stage 2: Request a nonce from the device.
    const nonceRequest = protocol.createNonceRequestPacket();
    await send(
      protocol.UNLOCKER_CHARACTERISTIC_UUID,
      nonceRequest,
      "withResponse",
      protocol.UNLOCKER_SERVICE_UUID,
    );
    console.log("Nonce request sent");
    const nonceResponse = await read(
      protocol.UNLOCKER_CHARACTERISTIC_UUID,
      protocol.UNLOCKER_SERVICE_UUID,
    );
    console.log("Nonce response received:", nonceResponse);
    const nonce = protocol.parseNonceResponsePacket(nonceResponse);
    if (!nonce) {
      throw new Error("Invalid nonce response");
    }
    console.log("Nonce:", nonce);

    console.log("Requesting challenge from backend...", {
      beacon: objectId,
      nonce: uint8ArrayToBase64(nonce),
      entity: entity,
    });

    // Stage 3: Handle the nonce and request a challenge from the backend.
    // This is handled by the Tauri core, so invoke the command.
    const proof: DeviceProof = JSON.parse(
      await invoke<string>("unlock_door", {
        beacon: objectId,
        nonce: uint8ArrayToBase64(nonce),
        entity: entity,
      }),
    );

    console.log("Device proof generated:", proof);

    const proofPacket = protocol.createProofSubmissionPacket(
      new Uint8Array(proof.challenge_hash),
      new Uint8Array(proof.device_signature),
      BigInt(proof.timestamp),
      BigInt(proof.counter),
    );
    await send(
      protocol.UNLOCKER_CHARACTERISTIC_UUID,
      proofPacket,
      "withResponse",
      protocol.UNLOCKER_SERVICE_UUID,
    );
    console.log("Proof submission sent", proofPacket);
    const proofResponse = await read(
      protocol.UNLOCKER_CHARACTERISTIC_UUID,
      protocol.UNLOCKER_SERVICE_UUID,
    );
    console.log("Proof response received:", proofResponse);
    const unlockStatus =
      protocol.parseUnlockCommandResponsePacket(proofResponse);
    if (unlockStatus === null) {
      throw new Error("Invalid proof response");
    }
    if (unlockStatus === protocol.UnlockerError.SUCCESS) {
      console.log("Device unlocked successfully");
    } else {
      console.error(
        "Failed to unlock device:",
        protocol.renderUnlockerError(unlockStatus),
      );
    }
    await disconnect();
  } catch (e) {
    try {await disconnect();} catch(_) {}
    console.error("Error during unlocking process:", e);
  }
}
