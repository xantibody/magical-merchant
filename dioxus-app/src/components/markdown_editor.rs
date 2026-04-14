use std::collections::HashMap;

use dioxus::prelude::*;

use crate::markdown::render_markdown;

/// Markdown editor with per-line inline rendering.
///
/// The active line shows raw Markdown. Other lines show rendered HTML.
/// Rendering happens asynchronously so editing is never blocked.
#[component]
pub fn MarkdownEditor(
    value: String,
    placeholder: Option<String>,
    oninput: EventHandler<String>,
) -> Element {
    let mut active_line = use_signal(|| 0usize);
    let mut render_cache = use_signal(HashMap::<usize, (String, String)>::new);
    // Tracks lines that need async rendering
    let mut pending_renders = use_signal(Vec::<(usize, String)>::new);
    let placeholder = placeholder.unwrap_or_default();

    let lines: Vec<String> = if value.is_empty() {
        vec![String::new()]
    } else {
        value.split('\n').map(String::from).collect()
    };
    let total_lines = lines.len();

    // Queue non-active lines that need rendering
    {
        let cache = render_cache();
        let mut to_render = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            if idx == active_line() {
                continue;
            }
            if line.trim().is_empty() {
                continue;
            }
            let needs_render = match cache.get(&idx) {
                Some((cached_src, _html)) => cached_src != line,
                None => true,
            };
            if needs_render {
                to_render.push((idx, line.clone()));
            }
        }
        if !to_render.is_empty() {
            pending_renders.set(to_render);
        }
    }

    // Process pending renders asynchronously
    use_effect(move || {
        let pending = pending_renders();
        if pending.is_empty() {
            return;
        }
        spawn(async move {
            for (idx, src) in pending.iter() {
                let src_for_render = src.clone();
                let src_for_cache = src.clone();
                let html = tokio::task::spawn_blocking(move || render_markdown(&src_for_render))
                    .await
                    .unwrap_or_default();
                // Update cache one at a time so UI updates incrementally
                render_cache.write().insert(*idx, (src_for_cache, html));
            }
            pending_renders.set(Vec::new());
        });
    });

    // Clean stale cache entries
    {
        let cache = render_cache();
        let stale: Vec<usize> = cache
            .keys()
            .filter(|k| **k >= total_lines)
            .copied()
            .collect();
        if !stale.is_empty() {
            let mut c = cache.clone();
            for k in stale {
                c.remove(&k);
            }
            render_cache.set(c);
        }
    }

    let update_line = {
        let value = value.clone();
        move |idx: usize, new_content: String| {
            let mut parts: Vec<String> = value.split('\n').map(String::from).collect();
            while parts.len() <= idx {
                parts.push(String::new());
            }
            parts[idx] = new_content;
            oninput(parts.join("\n"));
        }
    };

    let insert_line_after = {
        let value = value.clone();
        move |idx: usize| {
            let mut parts: Vec<String> = value.split('\n').map(String::from).collect();
            parts.insert(idx + 1, String::new());
            active_line.set(idx + 1);
            oninput(parts.join("\n"));
        }
    };

    let cache = render_cache();

    rsx! {
        div {
            class: "markdown-editor",
            for (idx , line) in lines.iter().enumerate() {
                if idx == active_line() {
                    div {
                        key: "line-{idx}",
                        class: "md-line md-line-edit",
                        input {
                            class: "md-line-input",
                            r#type: "text",
                            placeholder: if idx == 0 && value.is_empty() { placeholder.clone() } else { String::new() },
                            value: "{line}",
                            autofocus: true,
                            oninput: {
                                let update = update_line.clone();
                                move |e: Event<FormData>| update(idx, e.value())
                            },
                            onkeydown: {
                                let mut insert = insert_line_after.clone();
                                move |e: Event<KeyboardData>| {
                                    match e.key() {
                                        Key::Enter => {
                                            e.prevent_default();
                                            insert(idx);
                                        }
                                        Key::ArrowUp if idx > 0 => {
                                            active_line.set(idx - 1);
                                        }
                                        Key::ArrowDown if idx < total_lines - 1 => {
                                            active_line.set(idx + 1);
                                        }
                                        _ => {}
                                    }
                                }
                            },
                        }
                    }
                } else if line.trim().is_empty() {
                    div {
                        key: "line-{idx}",
                        class: "md-line md-line-empty",
                        onclick: move |_| active_line.set(idx),
                        "\u{00A0}"
                    }
                } else {
                    match cache.get(&idx) {
                        Some((_src, html)) => rsx! {
                            div {
                                key: "line-{idx}",
                                class: "md-line md-line-view",
                                onclick: move |_| active_line.set(idx),
                                dangerous_inner_html: "{html}",
                            }
                        },
                        None => rsx! {
                            div {
                                key: "line-{idx}",
                                class: "md-line md-line-raw",
                                onclick: move |_| active_line.set(idx),
                                "{line}"
                            }
                        },
                    }
                }
            }
        }
    }
}
