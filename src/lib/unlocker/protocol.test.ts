import { expect, test } from "vitest";
import {
  createNonceRequestPacket,
  parseNonceResponsePacket,
} from "@/lib/unlocker/protocol.ts";

test('Test nonce-related serialization and deserialization.', () => {
  expect(createNonceRequestPacket()).toEqual(new Uint8Array([0x03]));
  expect(
    parseNonceResponsePacket(new Uint8Array([0x04, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])),
  ).toEqual(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]));
})