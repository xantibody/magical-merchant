import { Show, For, createSignal, onMount, onCleanup } from "solid-js";
import { Portal } from "solid-js/web";
import type { Editor } from "@milkdown/kit/core";
import { commandsCtx, editorViewCtx, rootCtx, type CmdKey } from "@milkdown/kit/core";
import {
  sinkListItemCommand,
  liftListItemCommand,
  createCodeBlockCommand,
  insertHrCommand,
  toggleStrongCommand,
  toggleEmphasisCommand,
  toggleInlineCodeCommand,
  wrapInBulletListCommand,
  wrapInOrderedListCommand,
  wrapInBlockquoteCommand,
  wrapInHeadingCommand,
  turnIntoTextCommand,
} from "@milkdown/kit/preset/commonmark";
import { undoCommand, redoCommand } from "@milkdown/kit/plugin/history";
import Icon, { type IconName } from "./Icon";

interface MarkdownToolbarProps {
  editor: Editor | undefined;
}

type ToolbarAction = {
  icon: IconName;
  label: string;
  run: (editor: Editor) => void;
};

const callCommand =
  <T,>(key: CmdKey<T>, payload?: T) =>
  (editor: Editor) =>
    editor.action((ctx) => ctx.get(commandsCtx).call(key, payload));

/** H1 → H2 → H3 → 本文 を1ボタンで巡回（モバイルで階層メニューを開かせない） */
const cycleHeading = (editor: Editor) =>
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const block = view.state.selection.$from.node(1);
    const level = block?.type.name === "heading" ? (block.attrs.level as number) : 0;
    const commands = ctx.get(commandsCtx);
    if (level >= 3) {
      commands.call(turnIntoTextCommand.key);
    } else {
      commands.call(wrapInHeadingCommand.key, level + 1);
    }
  });

// グループは区切り線で分ける。モバイルでの使用頻度順に並べる。
const GROUPS: ToolbarAction[][] = [
  [
    { icon: "arrow-counter-clockwise", label: "元に戻す", run: callCommand(undoCommand.key) },
    { icon: "arrow-clockwise", label: "やり直す", run: callCommand(redoCommand.key) },
  ],
  [
    { icon: "text-h", label: "見出し", run: cycleHeading },
    { icon: "text-b", label: "太字", run: callCommand(toggleStrongCommand.key) },
    { icon: "text-italic", label: "斜体", run: callCommand(toggleEmphasisCommand.key) },
    { icon: "code", label: "インラインコード", run: callCommand(toggleInlineCodeCommand.key) },
  ],
  [
    { icon: "list-bullets", label: "箇条書き", run: callCommand(wrapInBulletListCommand.key) },
    { icon: "list-numbers", label: "番号付き", run: callCommand(wrapInOrderedListCommand.key) },
    { icon: "text-indent", label: "インデント", run: callCommand(sinkListItemCommand.key) },
    { icon: "text-outdent", label: "アウトデント", run: callCommand(liftListItemCommand.key) },
  ],
  [
    { icon: "quotes", label: "引用", run: callCommand(wrapInBlockquoteCommand.key) },
    { icon: "code-block", label: "コードブロック", run: callCommand(createCodeBlockCommand.key) },
    { icon: "minus", label: "区切り線", run: callCommand(insertHrCommand.key) },
  ],
];

export default function MarkdownToolbar(props: MarkdownToolbarProps) {
  const [toolbarTop, setToolbarTop] = createSignal<number | undefined>();

  // 仮想キーボードの直上に張り付かせる（visualViewport の変化に追従）
  onMount(() => {
    const vv = window.visualViewport;
    if (!vv) return;

    let raf = 0;
    const update = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(() => setToolbarTop(vv.offsetTop + vv.height));
    };

    vv.addEventListener("resize", update);
    vv.addEventListener("scroll", update);
    onCleanup(() => {
      cancelAnimationFrame(raf);
      vv.removeEventListener("resize", update);
      vv.removeEventListener("scroll", update);
    });
  });

  const exec = (run: (editor: Editor) => void) => {
    const editor = props.editor;
    if (!editor) return;
    run(editor);
    editor.action((ctx) => {
      const root = ctx.get(rootCtx) as HTMLElement;
      const pm = root.querySelector(".ProseMirror") as HTMLElement | null;
      pm?.focus();
    });
  };

  const top = () => toolbarTop();

  return (
    <Show when={props.editor}>
      <Portal>
        <div
          class="markdown-toolbar"
          role="toolbar"
          aria-label="Markdown formatting"
          style={
            top() != null
              ? { top: `${top()}px`, bottom: "auto", transform: "translateY(-100%)" }
              : undefined
          }
        >
          <For each={GROUPS}>
            {(group, groupIndex) => (
              <>
                <Show when={groupIndex() > 0}>
                  <span class="markdown-toolbar-divider" aria-hidden="true" />
                </Show>
                <For each={group}>
                  {(action) => (
                    <button
                      type="button"
                      onPointerDown={(e) => e.preventDefault()}
                      onClick={() => exec(action.run)}
                      aria-label={action.label}
                      title={action.label}
                    >
                      <Icon name={action.icon} size={20} />
                    </button>
                  )}
                </For>
              </>
            )}
          </For>
        </div>
      </Portal>
    </Show>
  );
}
