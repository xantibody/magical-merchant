import { createSignal, createEffect, on } from "solid-js";
import { renderMarkdown } from "../lib/markdown";

interface MarkdownPreviewProps {
  source: string;
  resolveWikilink?: (title: string) => string | null;
  onWikilinkClick?: (title: string) => void;
}

export default function MarkdownPreview(props: MarkdownPreviewProps) {
  const [html, setHtml] = createSignal("");

  let renderVersion = 0;

  createEffect(
    on(
      () => props.source,
      async (source) => {
        const currentVersion = ++renderVersion;
        if (!source) {
          setHtml("");
          return;
        }
        const rendered = await renderMarkdown(source, {
          resolveWikilink: props.resolveWikilink,
        });
        if (currentVersion === renderVersion) {
          setHtml(rendered);
        }
      },
    ),
  );

  // innerHTML なのでリンクに直接ハンドラを付けられない（イベント委譲）
  const handleClick = (e: MouseEvent) => {
    const link = (e.target as HTMLElement).closest("a.wikilink");
    if (link instanceof HTMLElement && link.dataset.wikilink) {
      e.preventDefault();
      props.onWikilinkClick?.(link.dataset.wikilink);
    }
  };

  return <div class="markdown-preview" innerHTML={html()} onClick={handleClick} />;
}
