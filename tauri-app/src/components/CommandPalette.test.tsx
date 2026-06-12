import { render, cleanup } from "@solidjs/testing-library";
import { page, userEvent } from "vitest/browser";
import { describe, it, expect, afterEach, vi, beforeEach } from "vitest";
import CommandPalette from "./CommandPalette";

vi.mock("../lib/commands", () => ({
  typedInvoke: vi.fn((cmd: string, args?: { query?: string }) => {
    if (cmd === "list_notes") {
      return Promise.resolve([
        { path: "/n/1.md", filename: "1.md", tags: [], preview: "p1", title: "Design Memo" },
        { path: "/n/2.md", filename: "2.md", tags: [], preview: "p2", title: "Shopping List" },
      ]);
    }
    if (cmd === "search_notes") {
      return Promise.resolve(
        args?.query === "basil"
          ? [{ filename: "2.md", title: "Shopping List", snippet: "…fresh basil…" }]
          : [],
      );
    }
    if (cmd === "search_timeline") {
      return Promise.resolve(
        args?.query === "basil"
          ? [{ date: "2026-06-01", time: "09:00:00", snippet: "bought basil" }]
          : [],
      );
    }
    return Promise.resolve([]);
  }),
}));

describe("CommandPalette", () => {
  beforeEach(() => vi.clearAllMocks());
  afterEach(() => cleanup());

  it("renders nothing when closed", () => {
    const { baseElement } = render(() => (
      <CommandPalette
        open={false}
        onClose={() => {}}
        onOpenNote={() => {}}
        onOpenTimelineDate={() => {}}
      />
    ));
    expect(baseElement.querySelector(".palette")).toBeNull();
  });

  it("shows recent notes when opened with empty query", async () => {
    const { baseElement } = render(() => (
      <CommandPalette open onClose={() => {}} onOpenNote={() => {}} onOpenTimelineDate={() => {}} />
    ));
    const screen = page.elementLocator(baseElement);
    await expect
      .element(screen.locator(".palette-item-label").first())
      .toHaveTextContent("Design Memo");
  });

  it("opens a note on Enter for the selected item", async () => {
    const onOpenNote = vi.fn();
    const { baseElement } = render(() => (
      <CommandPalette
        open
        onClose={() => {}}
        onOpenNote={onOpenNote}
        onOpenTimelineDate={() => {}}
      />
    ));
    const screen = page.elementLocator(baseElement);
    await expect.element(screen.locator(".palette-item").first()).toBeInTheDocument();
    await userEvent.keyboard("{Enter}");
    expect(onOpenNote).toHaveBeenCalledWith("1.md");
  });

  it("shows timeline hits for a content query and opens the date", async () => {
    const onOpenDate = vi.fn();
    const { baseElement } = render(() => (
      <CommandPalette
        open
        onClose={() => {}}
        onOpenNote={() => {}}
        onOpenTimelineDate={onOpenDate}
      />
    ));
    const screen = page.elementLocator(baseElement);
    await userEvent.keyboard("basil");
    // 検索結果: ノート全文ヒット + タイムラインヒットの2件
    const timelineItem = screen.locator(".palette-item").nth(1);
    await expect.element(timelineItem).toHaveTextContent("2026-06-01");
    await timelineItem.click();
    expect(onOpenDate).toHaveBeenCalledWith("2026-06-01");
  });

  it("closes on Escape", async () => {
    const onClose = vi.fn();
    const { baseElement } = render(() => (
      <CommandPalette open onClose={onClose} onOpenNote={() => {}} onOpenTimelineDate={() => {}} />
    ));
    const screen = page.elementLocator(baseElement);
    await expect.element(screen.locator(".palette-input")).toBeInTheDocument();
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalled();
  });
});
