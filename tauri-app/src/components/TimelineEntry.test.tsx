import { render, cleanup } from "@solidjs/testing-library";
import { page } from "vitest/browser";
import { describe, it, expect, afterEach, vi } from "vitest";
import TimelineEntry from "./TimelineEntry";

describe("TimelineEntry", () => {
  afterEach(() => cleanup());

  it("renders plain text without markdown", async () => {
    const { baseElement } = render(() => <TimelineEntry raw="- [12:34:56] hello world" />);
    const screen = page.elementLocator(baseElement);
    await expect.element(screen.locator(".timeline-entry-text")).toHaveTextContent("hello world");
  });

  it("forwards wikilink clicks in markdown mode", async () => {
    const onClick = vi.fn();
    const { baseElement } = render(() => (
      <TimelineEntry
        raw="- [12:34:56] met about [[Project X]]"
        markdown
        onWikilinkClick={onClick}
      />
    ));
    const screen = page.elementLocator(baseElement);
    const link = screen.locator("a.wikilink");
    await expect.element(link).toBeInTheDocument();
    await link.click();
    expect(onClick).toHaveBeenCalledWith("Project X");
  });
});
