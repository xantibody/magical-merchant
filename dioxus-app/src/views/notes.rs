use dioxus::prelude::*;

#[component]
pub fn Notes() -> Element {
    rsx! {
        div { class: "view",
            "Notes"
        }
    }
}
