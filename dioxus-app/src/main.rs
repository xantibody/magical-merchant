mod data_dir;

use dioxus::prelude::*;

fn main() {
    let base = data_dir::base_dir();
    std::fs::create_dir_all(&base).expect("could not create data directory");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            "Magical Merchant"
        }
    }
}
