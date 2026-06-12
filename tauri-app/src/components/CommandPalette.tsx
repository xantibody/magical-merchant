import { createSignal, createEffect, createResource, on, onCleanup, For, Show } from "solid-js";
import { typedInvoke, type Note } from "../lib/commands";
import { fuzzyScore } from "../lib/fuzzy";
import Icon from "./Icon";

const GROUP_LIMIT = 5;

export type PaletteItem =
  | { kind: "note"; filename: string; label: string; detail: string }
  | { kind: "timeline"; date: string; label: string; detail: string };

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  onOpenNote: (filename: string) => void;
  onOpenTimelineDate: (date: string) => void;
}

/** Fuzzy title matches, best first. */
function titleMatches(notes: Note[], query: string): { note: Note; score: number }[] {
  return notes
    .flatMap((note) => {
      const score = note.title ? fuzzyScore(query, note.title) : null;
      return score === null ? [] : [{ note, score }];
    })
    .sort((a, b) => b.score - a.score)
    .slice(0, GROUP_LIMIT);
}

export default function CommandPalette(props: CommandPaletteProps) {
  const [query, setQuery] = createSignal("");
  const [debouncedQuery, setDebouncedQuery] = createSignal("");
  const [selected, setSelected] = createSignal(0);
  let inputRef: HTMLInputElement | undefined;
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  const [notes] = createResource(
    () => props.open,
    (open) => (open ? typedInvoke("list_notes") : Promise.resolve([])),
  );
  const [noteHits] = createResource(debouncedQuery, (q) =>
    q.trim() ? typedInvoke("search_notes", { query: q }) : Promise.resolve([]),
  );
  const [timelineHits] = createResource(debouncedQuery, (q) =>
    q.trim() ? typedInvoke("search_timeline", { query: q }) : Promise.resolve([]),
  );

  const handleInput = (value: string) => {
    setQuery(value);
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => setDebouncedQuery(value), 150);
  };

  onCleanup(() => {
    if (debounceTimer) clearTimeout(debounceTimer);
  });

  createEffect(
    on(
      () => props.open,
      (open) => {
        if (open) {
          setQuery("");
          setDebouncedQuery("");
          setSelected(0);
          queueMicrotask(() => inputRef?.focus());
        }
      },
    ),
  );

  const items = (): PaletteItem[] => {
    const q = query().trim();
    const all = notes() ?? [];

    if (!q) {
      // 空クエリ: 最近のノート（list_notes は新しい順）
      return all.slice(0, 7).map((n) => ({
        kind: "note" as const,
        filename: n.filename,
        label: n.title || n.preview || "(untitled)",
        detail: "",
      }));
    }

    const titles: PaletteItem[] = titleMatches(all, q).map(({ note }) => ({
      kind: "note" as const,
      filename: note.filename,
      label: note.title || "(untitled)",
      detail: "",
    }));
    const seen = new Set(titles.map((t) => (t.kind === "note" ? t.filename : "")));

    const contents: PaletteItem[] = (noteHits() ?? [])
      .filter((hit) => !seen.has(hit.filename))
      .slice(0, GROUP_LIMIT)
      .map((hit) => ({
        kind: "note" as const,
        filename: hit.filename,
        label: hit.title || "(untitled)",
        detail: hit.snippet,
      }));

    const timeline: PaletteItem[] = (timelineHits() ?? []).slice(0, GROUP_LIMIT).map((hit) => ({
      kind: "timeline" as const,
      date: hit.date,
      label: `${hit.date} ${hit.time}`.trim(),
      detail: hit.snippet,
    }));

    return [...titles, ...contents, ...timeline];
  };

  // 選択位置は結果が変わったらリセット
  createEffect(on(items, () => setSelected(0), { defer: true }));

  const choose = (item: PaletteItem) => {
    if (item.kind === "note") props.onOpenNote(item.filename);
    else props.onOpenTimelineDate(item.date);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    const list = items();
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelected((i) => Math.min(i + 1, list.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelected((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const item = list[selected()];
      if (item) choose(item);
    } else if (e.key === "Escape") {
      e.preventDefault();
      props.onClose();
    }
  };

  return (
    <Show when={props.open}>
      <div class="palette-backdrop" onClick={props.onClose}>
        <div
          class="palette"
          role="dialog"
          aria-label="Command palette"
          onClick={(e) => e.stopPropagation()}
        >
          <div class="palette-input-row">
            <Icon name="magnifying-glass" size={16} />
            <input
              ref={inputRef}
              type="text"
              class="palette-input"
              placeholder="Jump to note or search everything..."
              value={query()}
              onInput={(e) => handleInput(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
            />
          </div>
          <div class="palette-results">
            <Show when={items().length} fallback={<p class="palette-empty">該当なし</p>}>
              <For each={items()}>
                {(item, index) => (
                  <button
                    class="palette-item"
                    classList={{ "palette-item--selected": index() === selected() }}
                    onMouseEnter={() => setSelected(index())}
                    onClick={() => choose(item)}
                  >
                    <Icon name={item.kind === "note" ? "note-pencil" : "lightning"} size={14} />
                    <span class="palette-item-label">{item.label}</span>
                    <Show when={item.detail}>
                      <span class="palette-item-detail">{item.detail}</span>
                    </Show>
                  </button>
                )}
              </For>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
}
