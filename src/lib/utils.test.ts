// Comprehensive unit tests for utility functions
import { describe, it, expect, vi } from "vitest";
import {
  rssiToDistance,
  formatTimestamp,
  isValidObjectId,
  safeJsonParse,
  debounce,
  calculateDistance,
  isPointInPolygon,
  retryWithBackoff,
} from "./utils";

describe("rssiToDistance", () => {
  it("should return -1 for RSSI of 0", () => {
    expect(rssiToDistance(0)).toBe(-1.0);
  });

  it("should calculate distance correctly with default parameters", () => {
    const distance = rssiToDistance(-59);
    expect(distance).toBeCloseTo(1.0, 1);
  });

  it("should calculate distance with custom txPower", () => {
    const distance = rssiToDistance(-70, -50, 2.0);
    expect(distance).toBeGreaterThan(0);
  });

  it("should handle very weak signals", () => {
    const distance = rssiToDistance(-100);
    expect(distance).toBeGreaterThan(10);
  });

  it("should handle very strong signals", () => {
    const distance = rssiToDistance(-30);
    expect(distance).toBeLessThan(1);
  });
});

describe("formatTimestamp", () => {
  it("should format timestamp to readable string", () => {
    const timestamp = Date.UTC(2025, 0, 1, 0, 0, 0);
    const formatted = formatTimestamp(timestamp);
    expect(formatted).toContain("2025");
  });

  it("should handle current timestamp", () => {
    const now = Date.now();
    const formatted = formatTimestamp(now);
    expect(formatted).toBeTruthy();
    expect(typeof formatted).toBe("string");
  });

  it("should format different timestamps differently", () => {
    const ts1 = Date.UTC(2025, 0, 1);
    const ts2 = Date.UTC(2025, 5, 15);
    expect(formatTimestamp(ts1)).not.toBe(formatTimestamp(ts2));
  });
});

describe("isValidObjectId", () => {
  it("should validate correct MongoDB ObjectId", () => {
    expect(isValidObjectId("507f1f77bcf86cd799439011")).toBe(true);
    expect(isValidObjectId("507f191e810c19729de860ea")).toBe(true);
  });

  it("should reject invalid ObjectIds", () => {
    expect(isValidObjectId("invalid")).toBe(false);
    expect(isValidObjectId("507f1f77bcf86cd79943901")).toBe(false); // Too short
    expect(isValidObjectId("507f1f77bcf86cd7994390111")).toBe(false); // Too long
    expect(isValidObjectId("507f1f77bcf86cd79943901g")).toBe(false); // Invalid char
  });

  it("should handle empty string", () => {
    expect(isValidObjectId("")).toBe(false);
  });

  it("should handle uppercase hex", () => {
    expect(isValidObjectId("507F1F77BCF86CD799439011")).toBe(true);
  });
});

describe("safeJsonParse", () => {
  it("should parse valid JSON", () => {
    const json = '{"name": "test", "value": 123}';
    const result = safeJsonParse(json, {});
    expect(result).toEqual({ name: "test", value: 123 });
  });

  it("should return fallback for invalid JSON", () => {
    const result = safeJsonParse("invalid json", { default: true });
    expect(result).toEqual({ default: true });
  });

  it("should handle empty string", () => {
    const result = safeJsonParse("", null);
    expect(result).toBeNull();
  });

  it("should parse arrays", () => {
    const json = "[1, 2, 3]";
    const result = safeJsonParse(json, []);
    expect(result).toEqual([1, 2, 3]);
  });

  it("should handle null values", () => {
    const json = '{"value": null}';
    const result = safeJsonParse(json, {});
    expect(result).toEqual({ value: null });
  });
});

describe("debounce", () => {
  it("should delay function execution", async () => {
    let counter = 0;
    const increment = () => counter++;
    const debounced = debounce(increment, 100);

    debounced();
    expect(counter).toBe(0);

    await new Promise((resolve) => setTimeout(resolve, 150));
    expect(counter).toBe(1);
  });

  it("should cancel previous calls", async () => {
    let counter = 0;
    const increment = () => counter++;
    const debounced = debounce(increment, 100);

    debounced();
    debounced();
    debounced();

    await new Promise((resolve) => setTimeout(resolve, 150));
    expect(counter).toBe(1);
  });

  it("should pass arguments correctly", async () => {
    let result = "";
    const fn = (val: string) => {
      result = val;
    };
    const debounced = debounce(fn, 50);

    debounced("test");
    await new Promise((resolve) => setTimeout(resolve, 100));
    expect(result).toBe("test");
  });
});

describe("calculateDistance", () => {
  it("should calculate distance between same points as zero", () => {
    const distance = calculateDistance(0, 0, 0, 0);
    expect(distance).toBeCloseTo(0, 1);
  });

  it("should calculate distance between known points", () => {
    // Distance between (0, 0) and (0, 1) should be ~111km
    const distance = calculateDistance(0, 0, 0, 1);
    expect(distance).toBeGreaterThan(100000);
    expect(distance).toBeLessThan(120000);
  });

  it("should be symmetric", () => {
    const d1 = calculateDistance(40.7128, -74.006, 34.0522, -118.2437);
    const d2 = calculateDistance(34.0522, -118.2437, 40.7128, -74.006);
    expect(d1).toBeCloseTo(d2, 0);
  });

  it("should handle negative coordinates", () => {
    const distance = calculateDistance(-33.8688, 151.2093, 51.5074, -0.1278);
    expect(distance).toBeGreaterThan(0);
  });
});

describe("isPointInPolygon", () => {
  it("should detect point inside square", () => {
    const polygon: [number, number][] = [
      [0, 0],
      [10, 0],
      [10, 10],
      [0, 10],
    ];
    expect(isPointInPolygon([5, 5], polygon)).toBe(true);
  });

  it("should detect point outside square", () => {
    const polygon: [number, number][] = [
      [0, 0],
      [10, 0],
      [10, 10],
      [0, 10],
    ];
    expect(isPointInPolygon([15, 15], polygon)).toBe(false);
  });

  it("should detect point on edge", () => {
    const polygon: [number, number][] = [
      [0, 0],
      [10, 0],
      [10, 10],
      [0, 10],
    ];
    const result = isPointInPolygon([0, 5], polygon);
    expect(typeof result).toBe("boolean");
  });

  it("should work with complex polygon", () => {
    const polygon: [number, number][] = [
      [0, 0],
      [5, 0],
      [10, 5],
      [5, 10],
      [0, 5],
    ];
    expect(isPointInPolygon([5, 5], polygon)).toBe(true);
    expect(isPointInPolygon([9, 9], polygon)).toBe(false);
  });

  it("should handle triangle", () => {
    const polygon: [number, number][] = [
      [0, 0],
      [10, 0],
      [5, 10],
    ];
    expect(isPointInPolygon([5, 3], polygon)).toBe(true);
    expect(isPointInPolygon([0, 10], polygon)).toBe(false);
  });
});

describe("retryWithBackoff", () => {
  it("should succeed on first try", async () => {
    const fn = vi.fn().mockResolvedValue("success");
    const result = await retryWithBackoff(fn);
    expect(result).toBe("success");
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it("should retry on failure", async () => {
    const fn = vi
      .fn()
      .mockRejectedValueOnce(new Error("fail1"))
      .mockRejectedValueOnce(new Error("fail2"))
      .mockResolvedValue("success");

    const result = await retryWithBackoff(fn, 3, 10);
    expect(result).toBe("success");
    expect(fn).toHaveBeenCalledTimes(3);
  });

  it("should throw after max retries", async () => {
    const fn = vi.fn().mockRejectedValue(new Error("persistent error"));

    await expect(retryWithBackoff(fn, 2, 10)).rejects.toThrow(
      "persistent error",
    );
    expect(fn).toHaveBeenCalledTimes(2);
  });

  it("should use exponential backoff", async () => {
    const fn = vi
      .fn()
      .mockRejectedValueOnce(new Error("fail1"))
      .mockRejectedValueOnce(new Error("fail2"))
      .mockResolvedValue("success");

    const start = Date.now();
    await retryWithBackoff(fn, 3, 50);
    const duration = Date.now() - start;

    // Should take at least 50ms (first retry) + 100ms (second retry)
    expect(duration).toBeGreaterThan(140);
  });

  it("should handle custom retry count", async () => {
    const fn = vi.fn().mockRejectedValue(new Error("fail"));

    await expect(retryWithBackoff(fn, 5, 1)).rejects.toThrow();
    expect(fn).toHaveBeenCalledTimes(5);
  });
});

describe("Edge Cases", () => {
  it("should handle extreme RSSI values", () => {
    expect(rssiToDistance(-200)).toBeGreaterThan(0);
    expect(rssiToDistance(0)).toBe(-1);
  });

  it("should handle invalid polygon data", () => {
    const emptyPolygon: [number, number][] = [];
    expect(() => isPointInPolygon([0, 0], emptyPolygon)).not.toThrow();
  });

  it("should handle very long JSON strings", () => {
    const largeObject = { data: "x".repeat(10000) };
    const json = JSON.stringify(largeObject);
    const result = safeJsonParse(json, { data: "" });
    expect(result.data).toHaveLength(10000);
  });

  it("should handle concurrent debounce calls", async () => {
    let counter = 0;
    const increment = () => counter++;
    const debounced = debounce(increment, 50);

    for (let i = 0; i < 100; i++) {
      debounced();
    }

    await new Promise((resolve) => setTimeout(resolve, 100));
    expect(counter).toBe(1);
  });
});
