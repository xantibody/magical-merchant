use dioxus::prelude::*;

#[component]
pub fn Tasks() -> Element {
    rsx! {
        div { class: "view",
            "Tasks"
        }
    }
}
