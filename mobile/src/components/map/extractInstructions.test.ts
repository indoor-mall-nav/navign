import { describe, it, expect } from "vitest";
import { extractInstructions } from "./extractInstructions";

describe("Extract Instructions", () => {
  it("should extract instructions correctly", () => {
    // Test cases would go here
    let instructions = {
      instructions: [
        { move: [12.5, 38.0] },
        { move: [12.5, 42.5] },
        { move: [2.5, 42.5] },
        { move: [2.5, 55.5] },
        { move: [2.5, 68.0] },
        {
          transport: [
            "68a834a1bdfa76608b934af0",
            "68a83067bdfa76608b934aeb",
            "elevator",
          ],
        },
        { move: [2.5, 68.0] },
        { move: [25.0, 68.0] },
      ],
    };
    // @ts-ignore
    expect(extractInstructions(instructions.instructions)).toEqual([
      { type: "turn", turn: "left" },
      { type: "straight", straight: 4.5 },
      { type: "turn", turn: "right" },
      { type: "straight", straight: 38.08214804865923 },
      { type: "turn", turn: "left" },
      { type: "straight", straight: 17.5 },
      { type: "turn", turn: "right" },
      { type: "straight", straight: 50.5618433208284 },
      {
        type: "transport",
        transport: [
          "68a834a1bdfa76608b934af0",
          "68a83067bdfa76608b934aeb",
          "elevator",
        ],
      },
      { type: "turn", turn: "right" },
      { type: "straight", straight: 22.5 },
      { type: "unlock" },
    ]);
  });
});
