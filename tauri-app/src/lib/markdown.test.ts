import { describe, it, expect } from "vitest";
import { renderMarkdown } from "./markdown";

describe("renderMarkdown", () => {
  it("converts a heading", async () => {
    expect(await renderMarkdown("# Hello")).toContain("<h1>Hello</h1>");
  });

  it("converts a paragraph", async () => {
    expect(await renderMarkdown("Some text")).toContain("<p>Some text</p>");
  });

  it("converts an unordered list", async () => {
    const html = await renderMarkdown("- item1\n- item2");
    expect(html).toContain("<ul>");
    expect(html).toContain("<li>item1</li>");
    expect(html).toContain("<li>item2</li>");
  });

  it("converts inline code", async () => {
    expect(await renderMarkdown("use `foo()` here")).toContain("<code>foo()</code>");
  });

  it("converts bold and italic", async () => {
    const html = await renderMarkdown("**bold** and *italic*");
    expect(html).toContain("<strong>bold</strong>");
    expect(html).toContain("<em>italic</em>");
  });

  it("converts a link", async () => {
    const html = await renderMarkdown("[click](https://example.com)");
    expect(html).toContain('<a href="https://example.com">click</a>');
  });

  it("does not render raw HTML (html: false)", async () => {
    expect(await renderMarkdown('<script>alert("xss")</script>')).not.toContain("<script>");
  });

  it("returns empty string for empty string", async () => {
    expect((await renderMarkdown("")).trim()).toBe("");
  });
});

describe("wikilink rendering", () => {
  it("renders [[Title]] as a wikilink anchor", async () => {
    const html = await renderMarkdown("see [[My Note]]");
    expect(html).toContain('data-wikilink="My Note"');
    expect(html).toContain('class="wikilink"');
    expect(html).toContain(">My Note</a>");
  });

  it("does not linkify inside inline code", async () => {
    const html = await renderMarkdown("`[[code]]`");
    expect(html).not.toContain("data-wikilink");
    expect(html).toContain("<code>[[code]]</code>");
  });

  it("does not linkify inside fenced code block", async () => {
    const html = await renderMarkdown("```\n[[fence]]\n```");
    expect(html).not.toContain("data-wikilink");
  });

  it("escapes html in title", async () => {
    const html = await renderMarkdown("[[<img src=x>]]");
    expect(html).not.toContain("<img");
  });

  it("leaves unclosed [[ as plain text", async () => {
    const html = await renderMarkdown("[[unclosed");
    expect(html).not.toContain("data-wikilink");
    expect(html).toContain("[[unclosed");
  });

  it("ignores empty titles", async () => {
    const html = await renderMarkdown("[[]] and [[  ]]");
    expect(html).not.toContain("data-wikilink");
  });

  it("marks unresolved links when a resolver is provided", async () => {
    const html = await renderMarkdown("[[Known]] [[Unknown]]", {
      resolveWikilink: (title) => (title === "Known" ? "20260101_000001.md" : null),
    });
    expect(html).toContain('class="wikilink"');
    expect(html).toContain('class="wikilink wikilink--unresolved"');
  });

  it("trims wikilink titles", async () => {
    const html = await renderMarkdown("[[ Padded ]]");
    expect(html).toContain('data-wikilink="Padded"');
  });
});
