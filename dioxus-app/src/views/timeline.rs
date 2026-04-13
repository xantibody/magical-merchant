use dioxus::prelude::*;

#[component]
pub fn Timeline() -> Element {
    rsx! {
        div { class: "view",
            "Timeline"
        }
    }
}
