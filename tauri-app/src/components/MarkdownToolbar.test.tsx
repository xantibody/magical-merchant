import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import type { Editor } from "@milkdown/kit/core";
import MarkdownToolbar from "./MarkdownToolbar";

function createMockEditor(): Editor {
  return {
    action: vi.fn(),
  } as unknown as Editor;
}

describe("MarkdownToolbar", () => {
  afterEach(() => {
    cleanup();
    document.body.innerHTML = "";
  });

  it("does not render when editor is undefined", () => {
    render(() => <MarkdownToolbar editor={undefined} />);
    expect(screen.queryByRole("toolbar")).toBeNull();
  });

  it("renders the full set of formatting buttons when editor is provided", () => {
    const editor = createMockEditor();
    render(() => <MarkdownToolbar editor={editor} />);

    const toolbar = screen.getByRole("toolbar");
    expect(toolbar).toBeDefined();

    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(13);

    // 各グループの代表ボタンが揃っている
    expect(screen.getByLabelText("元に戻す")).toBeDefined();
    expect(screen.getByLabelText("見出し")).toBeDefined();
    expect(screen.getByLabelText("太字")).toBeDefined();
    expect(screen.getByLabelText("インデント")).toBeDefined();
    expect(screen.getByLabelText("コードブロック")).toBeDefined();
  });

  it("calls editor.action when a button is clicked", () => {
    const editor = createMockEditor();
    render(() => <MarkdownToolbar editor={editor} />);

    fireEvent.click(screen.getByLabelText("太字"));

    expect(editor.action).toHaveBeenCalled();
  });

  it("hides toolbar when editor becomes undefined", () => {
    const [editor, setEditor] = createSignal<Editor | undefined>(createMockEditor());
    render(() => <MarkdownToolbar editor={editor()} />);

    expect(screen.getByRole("toolbar")).toBeDefined();

    setEditor(undefined);
    expect(screen.queryByRole("toolbar")).toBeNull();
  });
});
