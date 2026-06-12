import { render, cleanup } from "@solidjs/testing-library";
import { page } from "vitest/browser";
import { describe, it, expect, afterEach, vi } from "vitest";
import MarkdownPreview from "./MarkdownPreview";

describe("MarkdownPreview", () => {
  afterEach(() => cleanup());

  it("calls onWikilinkClick with the title when a wikilink is clicked", async () => {
    const onClick = vi.fn();
    const { baseElement } = render(() => (
      <MarkdownPreview source="see [[My Note]]" onWikilinkClick={onClick} />
    ));
    const screen = page.elementLocator(baseElement);
    const link = screen.locator("a.wikilink");
    await expect.element(link).toBeInTheDocument();
    await link.click();
    expect(onClick).toHaveBeenCalledWith("My Note");
  });

  it("does not call onWikilinkClick for normal links", async () => {
    const onClick = vi.fn();
    const { baseElement } = render(() => (
      <MarkdownPreview source="[plain](#anchor)" onWikilinkClick={onClick} />
    ));
    const screen = page.elementLocator(baseElement);
    const link = screen.locator(".markdown-preview a");
    await expect.element(link).toBeInTheDocument();
    await link.click();
    expect(onClick).not.toHaveBeenCalled();
  });

  it("marks unresolved wikilinks using the resolver", async () => {
    const { baseElement } = render(() => (
      <MarkdownPreview source="[[Missing]]" resolveWikilink={() => null} />
    ));
    const screen = page.elementLocator(baseElement);
    await expect.element(screen.locator("a.wikilink--unresolved")).toBeInTheDocument();
  });
});
