import { describe, it, expect } from "vitest";
import { isBackSwipe, type SwipePoint } from "./swipe-back";

const from = (x: number, y: number, t: number): SwipePoint => ({ x, y, t });

describe("isBackSwipe", () => {
  it("accepts a fast horizontal swipe from the left edge", () => {
    expect(isBackSwipe(from(10, 300, 0), from(120, 310, 200))).toBe(true);
  });

  it("rejects swipes that do not start at the left edge", () => {
    expect(isBackSwipe(from(200, 300, 0), from(320, 310, 200))).toBe(false);
  });

  it("rejects swipes that are too short", () => {
    expect(isBackSwipe(from(10, 300, 0), from(60, 305, 200))).toBe(false);
  });

  it("rejects mostly-vertical swipes", () => {
    expect(isBackSwipe(from(10, 300, 0), from(120, 400, 200))).toBe(false);
  });

  it("rejects swipes that are too slow", () => {
    expect(isBackSwipe(from(10, 300, 0), from(120, 310, 900))).toBe(false);
  });

  it("rejects zero-duration (defensive against bad timestamps)", () => {
    expect(isBackSwipe(from(10, 300, 5), from(120, 310, 5))).toBe(false);
  });
});
