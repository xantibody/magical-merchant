use dioxus::prelude::*;
use magical_merchant_core::{read_timeline, save_timeline_entry, DeviceContext};

use crate::components::action_bar::ActionBar;
use crate::components::icon::{Icon, IconKind};
use crate::data_dir;

#[component]
pub fn Timeline() -> Element {
    let mut text = use_signal(String::new);
    let mut entries = use_signal(Vec::<String>::new);
    let mut saving = use_signal(|| false);

    // Load today's entries on mount
    use_effect(move || {
        spawn(async move {
            let base = data_dir::base_dir();
            let today = chrono::Local::now().date_naive();
            if let Ok(Ok(lines)) =
                tokio::task::spawn_blocking(move || read_timeline(&base, today)).await
            {
                entries.set(lines);
            }
        });
    });

    let mut handle_send = move || {
        let trimmed = text().trim().to_string();
        if trimmed.is_empty() || saving() {
            return;
        }
        saving.set(true);
        spawn(async move {
            let base = data_dir::base_dir();
            let entry_text = trimmed.clone();
            let _ = tokio::task::spawn_blocking(move || {
                save_timeline_entry(&base, &entry_text, &DeviceContext::mock())
            })
            .await;

            text.set(String::new());

            let base = data_dir::base_dir();
            let today = chrono::Local::now().date_naive();
            if let Ok(Ok(lines)) =
                tokio::task::spawn_blocking(move || read_timeline(&base, today)).await
            {
                entries.set(lines);
            }
            saving.set(false);
        });
    };

    rsx! {
        div { class: "view timeline-view",
            div { class: "timeline-input",
                textarea {
                    class: "memo-input",
                    placeholder: "What's on your mind?",
                    value: "{text}",
                    rows: "3",
                    oninput: move |e| text.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter && e.modifiers().contains(Modifiers::META) {
                            handle_send();
                        }
                    },
                }
            }

            ActionBar {
                button {
                    class: "action-btn",
                    disabled: text().trim().is_empty() || saving(),
                    onclick: move |_| handle_send(),
                    Icon { kind: IconKind::PaperPlaneTilt, size: 16 }
                }
            }

            if !entries().is_empty() {
                div { class: "timeline-entries",
                    for (i, entry) in entries().iter().rev().enumerate() {
                        div { key: "{i}", class: "timeline-entry",
                            "{entry}"
                        }
                    }
                }
            }
        }
    }
}
