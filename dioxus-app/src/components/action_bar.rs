use dioxus::prelude::*;

/// A container for action buttons that appear on hover.
/// Buttons are hidden by default and fade in when the parent area is hovered.
#[component]
pub fn ActionBar(children: Element) -> Element {
    rsx! {
        div { class: "action-bar",
            {children}
        }
    }
}
