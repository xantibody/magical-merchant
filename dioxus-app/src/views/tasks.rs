use dioxus::prelude::*;
use magical_merchant_core::{
    complete_task, create_task, list_active_tasks, list_projects, ProjectSummary, TaskSummary,
};

use crate::components::icon::{Icon, IconKind};
use crate::data_dir;

#[component]
pub fn Tasks() -> Element {
    let mut projects = use_signal(Vec::<ProjectSummary>::new);
    let mut selected_slug = use_signal(|| Option::<String>::None);
    let mut active_tasks = use_signal(Vec::<TaskSummary>::new);
    let mut title = use_signal(String::new);
    let mut creating = use_signal(|| false);

    // Load projects on mount
    use_effect(move || {
        spawn(async move {
            let base = data_dir::base_dir();
            if let Ok(Ok(result)) = tokio::task::spawn_blocking(move || list_projects(&base)).await
            {
                if selected_slug().is_none() {
                    if let Some(first) = result.first() {
                        selected_slug.set(Some(first.slug.clone()));
                    }
                }
                projects.set(result);
            }
        });
    });

    // Load tasks when project selection changes
    use_effect(move || {
        let slug = selected_slug();
        if let Some(slug) = slug {
            spawn(async move {
                let base = data_dir::base_dir();
                if let Ok(Ok(tasks)) =
                    tokio::task::spawn_blocking(move || list_active_tasks(&base, &slug)).await
                {
                    active_tasks.set(tasks);
                }
            });
        }
    });

    let reload_tasks = move || {
        if let Some(slug) = selected_slug() {
            spawn(async move {
                let base = data_dir::base_dir();
                if let Ok(Ok(tasks)) =
                    tokio::task::spawn_blocking(move || list_active_tasks(&base, &slug)).await
                {
                    active_tasks.set(tasks);
                }
            });
        }
    };

    let mut handle_create = move |_| {
        let task_title = title().trim().to_string();
        if task_title.is_empty() || creating() {
            return;
        }
        if let Some(slug) = selected_slug() {
            creating.set(true);
            spawn(async move {
                let base = data_dir::base_dir();
                let _ = tokio::task::spawn_blocking(move || {
                    create_task(&base, &slug, &task_title, &[], "")
                })
                .await;
                title.set(String::new());
                creating.set(false);
                reload_tasks();
            });
        }
    };

    let handle_complete = move |filename: String| {
        if let Some(slug) = selected_slug() {
            spawn(async move {
                let base = data_dir::base_dir();
                let _ = tokio::task::spawn_blocking(move || complete_task(&base, &slug, &filename))
                    .await;
                reload_tasks();
            });
        }
    };

    rsx! {
        div { class: "view tasks-view",
            if projects().is_empty() {
                p { class: "empty-state", "No projects yet" }
            } else {
                select {
                    class: "project-select",
                    value: selected_slug().unwrap_or_default(),
                    onchange: move |e| {
                        let val = e.value();
                        selected_slug.set(if val.is_empty() { None } else { Some(val) });
                    },
                    for project in projects().iter() {
                        option {
                            value: "{project.slug}",
                            "{project.name} ({project.active_task_count})"
                        }
                    }
                }

                div { class: "task-create",
                    input {
                        class: "task-title-input",
                        r#type: "text",
                        placeholder: "New task...",
                        value: "{title}",
                        oninput: move |e| title.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                handle_create(e);
                            }
                        },
                    }
                }

                if active_tasks().is_empty() {
                    p { class: "empty-state", "No active tasks" }
                } else {
                    div { class: "task-list",
                        for task in active_tasks().iter() {
                            div { key: "{task.filename}", class: "task-item",
                                button {
                                    class: "task-complete-btn",
                                    onclick: {
                                        let filename = task.filename.clone();
                                        move |_| handle_complete(filename.clone())
                                    },
                                    Icon { kind: IconKind::Circle, size: 18 }
                                }
                                span { class: "task-title", "{task.title}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
