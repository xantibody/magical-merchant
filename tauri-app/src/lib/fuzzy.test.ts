import { describe, it, expect } from "vitest";
import { fuzzyScore } from "./fuzzy";

describe("fuzzyScore", () => {
  it("returns null when query is not a subsequence", () => {
    expect(fuzzyScore("xyz", "design memo")).toBeNull();
  });

  it("returns null for empty target", () => {
    expect(fuzzyScore("a", "")).toBeNull();
  });

  it("matches empty query with lowest score", () => {
    expect(fuzzyScore("", "anything")).toBe(0);
  });

  it("scores exact substring higher than scattered subsequence", () => {
    const substring = fuzzyScore("memo", "design memo");
    const scattered = fuzzyScore("memo", "make example mode");
    expect(substring).not.toBeNull();
    expect(scattered).not.toBeNull();
    expect(substring!).toBeGreaterThan(scattered!);
  });

  it("scores prefix match higher than mid-string match", () => {
    const prefix = fuzzyScore("des", "design memo");
    const mid = fuzzyScore("des", "grand design");
    expect(prefix!).toBeGreaterThan(mid!);
  });

  it("is case-insensitive", () => {
    expect(fuzzyScore("MEMO", "design memo")).not.toBeNull();
    expect(fuzzyScore("memo", "DESIGN MEMO")).not.toBeNull();
  });

  it("matches japanese text", () => {
    expect(fuzzyScore("設計", "設計メモ")).not.toBeNull();
    expect(fuzzyScore("メモ", "設計メモ")).not.toBeNull();
  });

  it("prefers shorter targets on equal match quality", () => {
    const short = fuzzyScore("memo", "memo");
    const long = fuzzyScore("memo", "memo about everything else");
    expect(short!).toBeGreaterThan(long!);
  });
});
