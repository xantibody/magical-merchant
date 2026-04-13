use std::path::PathBuf;
use std::time::Duration;

use dioxus::prelude::*;
use magical_merchant_core::{create_draft_note, update_note, DeviceContext};

use crate::data_dir;

#[derive(Clone, Copy, Debug, PartialEq)]
enum SaveStatus {
    Idle,
    Saving,
    Saved,
}

#[component]
pub fn Notes() -> Element {
    let mut body = use_signal(String::new);
    let mut tags_input = use_signal(String::new);
    let mut draft_path = use_signal(|| Option::<PathBuf>::None);
    let mut status = use_signal(|| SaveStatus::Idle);
    let mut last_saved_body = use_signal(String::new);
    let mut last_saved_tags = use_signal(String::new);

    // Autosave with debounce
    use_effect(move || {
        let current_body = body();
        let current_tags = tags_input();

        // Skip if nothing to save or nothing changed
        if current_body.trim().is_empty() {
            return;
        }
        if current_body == last_saved_body() && current_tags == last_saved_tags() {
            return;
        }

        spawn(async move {
            // Debounce: wait 1 second
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Re-check current values after debounce
            let save_body = body();
            let save_tags_input = tags_input();
            if save_body.trim().is_empty() {
                return;
            }

            let tags: Vec<String> = save_tags_input
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            status.set(SaveStatus::Saving);

            let base = data_dir::base_dir();
            let path = draft_path();
            let body_clone = save_body.clone();
            let tags_clone = tags.clone();

            let result = tokio::task::spawn_blocking(move || {
                if let Some(ref existing) = path {
                    update_note(existing, &body_clone, &tags_clone, &DeviceContext::mock())
                        .map(|()| existing.clone())
                } else {
                    create_draft_note(&base, &body_clone, &tags_clone, &DeviceContext::mock())
                }
            })
            .await;

            match result {
                Ok(Ok(path)) => {
                    draft_path.set(Some(path));
                    last_saved_body.set(save_body);
                    last_saved_tags.set(save_tags_input);
                    status.set(SaveStatus::Saved);
                }
                _ => {
                    status.set(SaveStatus::Idle);
                }
            }
        });
    });

    let handle_done = move |_| {
        body.set(String::new());
        tags_input.set(String::new());
        draft_path.set(None);
        status.set(SaveStatus::Idle);
        last_saved_body.set(String::new());
        last_saved_tags.set(String::new());
    };

    let status_text = match status() {
        SaveStatus::Idle => "",
        SaveStatus::Saving => "Saving...",
        SaveStatus::Saved => "Saved",
    };

    rsx! {
        div { class: "view notes-view",
            div { class: "notes-editor",
                textarea {
                    class: "memo-input notes-body",
                    placeholder: "Write your note in Markdown...",
                    value: "{body}",
                    oninput: move |e| body.set(e.value()),
                }
            }

            div { class: "notes-footer",
                span { class: "save-status", "{status_text}" }
                input {
                    class: "tags-input",
                    r#type: "text",
                    placeholder: "Tags (comma separated)",
                    value: "{tags_input}",
                    oninput: move |e| tags_input.set(e.value()),
                }
                button {
                    class: "done-btn",
                    disabled: draft_path().is_none(),
                    onclick: handle_done,
                    "Done"
                }
            }
        }
    }
}
