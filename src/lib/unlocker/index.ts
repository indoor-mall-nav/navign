import { type BleDevice, connect, read, send } from "@mnlphlp/plugin-blec";
import { invoke } from "@tauri-apps/api/core";
import * as protocol from "./protocol";
import { DeviceCapability } from "./protocol";
import { authenticate, type AuthOptions } from "@tauri-apps/plugin-biometric";

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
  challengeHash: Uint8Array;
  deviceSignature: Uint8Array;
  timestamp: number;
  counter: number;
}

export async function unlockDevice(device: BleDevice, entity: string) {
  try {
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
    const deviceInquiry = new Uint8Array([0x01]);
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

    // Stage 3: Handle the nonce and request a challenge from the backend.
    // This is handled by the Tauri core, so invoke the command.
    const challengeResponse: Challenge = JSON.parse(
      await invoke<string>("request_challenge", {
        beacon: objectId,
        nonce: uint8ArrayToBase64(nonce),
        entity: entity,
      }),
    );
    console.log("Challenge response received:", challengeResponse);

    // Stage 4: Biometric authentication.
    const biometricConfig: AuthOptions = {
      allowDeviceCredential: false,
      cancelTitle: "Cancel",
      fallbackTitle: "Use Password",
      title: "Biometric Authentication",
      subtitle:
        "Authenticate to unlock the gateway/entrance, so that we can confirm it's you.",
      confirmationRequired: true,
      maxAttemps: 3,
    };
    await authenticate(
      "Authenticate to unlock the gateway/entrance",
      biometricConfig,
    );
    console.log("Biometric authentication successful");

    // Stage 5: Convert the challenge response to a proof submission packet and send it to the device.
    const proof: DeviceProof = JSON.parse(
      await invoke<string>("generate_device_proof", {
        challengeHash: uint8ArrayToBase64(challengeResponse.challengeHash),
        deviceSignature: uint8ArrayToBase64(challengeResponse.deviceSignature),
        timestamp: challengeResponse.timestamp.toString(),
        counter: challengeResponse.counter.toString(),
      }),
    );
    console.log("Device proof generated:", proof);

    const proofPacket = protocol.createProofSubmissionPacket(
      new Uint8Array(proof.challengeHash),
      new Uint8Array(proof.deviceSignature),
      BigInt(proof.timestamp),
      BigInt(proof.counter),
    );
    await send(
      protocol.UNLOCKER_CHARACTERISTIC_UUID,
      proofPacket,
      "withResponse",
      protocol.UNLOCKER_SERVICE_UUID,
    );
    console.log("Proof submission sent");
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
  } catch (e) {
    console.error("Error during unlocking process:", e);
  }
}
