import MarkdownIt from "markdown-it";
import { getHighlighter } from "./highlighter";

interface ShikiBlock {
  id: string;
  code: string;
  lang: string;
}

export interface WikilinkEnv {
  /** Returns the target filename, or null when no note has the title. */
  resolveWikilink?: (title: string) => string | null;
}

interface RenderEnv extends WikilinkEnv {
  __shikiBlocks?: ShikiBlock[];
}

// [[Title]] links. Registered before "link" so the earlier "backticks" rule
// has already consumed inline code spans; fences never reach inline parsing.
function wikilinkPlugin(markdown: MarkdownIt): void {
  markdown.inline.ruler.before("link", "wikilink", (state, silent) => {
    if (!state.src.startsWith("[[", state.pos)) return false;
    const close = state.src.indexOf("]]", state.pos + 2);
    if (close < 0) return false;
    const title = state.src.slice(state.pos + 2, close).trim();
    if (!title || /[[\]\n]/.test(title)) return false;
    if (!silent) {
      const token = state.push("wikilink", "a", 0);
      token.content = title;
    }
    state.pos = close + 2;
    return true;
  });
  markdown.renderer.rules.wikilink = (tokens, idx, _options, env: WikilinkEnv) => {
    const title = tokens[idx].content;
    const resolved = env.resolveWikilink ? env.resolveWikilink(title) : undefined;
    const cls = resolved === null ? "wikilink wikilink--unresolved" : "wikilink";
    const escaped = markdown.utils.escapeHtml(title);
    return `<a href="#" class="${cls}" data-wikilink="${escaped}">${escaped}</a>`;
  };
}

const fenceMd = MarkdownIt({
  html: false,
  linkify: true,
  typographer: true,
}).use(wikilinkPlugin);

fenceMd.renderer.rules.fence = (tokens, idx, _options, renderEnv: RenderEnv) => {
  const token = tokens[idx];
  const lang = token.info.trim();
  const code = token.content;

  const id = `shiki-${idx}`;
  renderEnv.__shikiBlocks = renderEnv.__shikiBlocks || [];
  renderEnv.__shikiBlocks.push({ id, code, lang });

  return `<div id="${id}" class="shiki-placeholder"><pre><code>${fenceMd.utils.escapeHtml(code)}</code></pre></div>`;
};

export async function renderMarkdown(source: string, wikilinkEnv?: WikilinkEnv): Promise<string> {
  const env: RenderEnv = { ...wikilinkEnv };
  let html = fenceMd.render(source, env);

  const blocks = env.__shikiBlocks || [];
  if (blocks.length > 0) {
    const highlighter = await getHighlighter();
    for (const block of blocks) {
      // 未ロード言語はプレーンテキストとして描画（フルバンドルを避けるため）
      const lang = highlighter.getLoadedLanguages().includes(block.lang) ? block.lang : "text";
      try {
        // デュアルテーマで描画し、テーマ切替にはCSS変数で即追従させる
        const highlighted = highlighter.codeToHtml(block.code, {
          lang,
          themes: {
            light: "github-light-default",
            dark: "github-dark-default",
          },
          defaultColor: false,
        });
        html = html.replace(
          `<div id="${block.id}" class="shiki-placeholder"><pre><code>${fenceMd.utils.escapeHtml(block.code)}</code></pre></div>`,
          highlighted,
        );
      } catch {
        // keep fallback
      }
    }
  }

  return html;
}
