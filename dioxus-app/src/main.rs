mod components;
mod data_dir;
pub mod markdown;
mod views;

use components::icon::{Icon, IconKind};
use dioxus::prelude::*;
use views::{notes::Notes, tasks::Tasks, timeline::Timeline};

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        Timeline {},
        #[route("/notes")]
        Notes {},
        #[route("/tasks")]
        Tasks {},
}

fn main() {
    dioxus::logger::init(dioxus::logger::tracing::Level::TRACE).expect("failed to init logger");
    let base = data_dir::base_dir();
    std::fs::create_dir_all(&base).expect("could not create data directory");
    dioxus::logger::tracing::info!("data dir: {}", base.display());
    dioxus::launch(App);
}

const OPEN_PROPS: Asset = asset!("/assets/open-props.min.css");
const STYLE: Asset = asset!("/assets/style.css");

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: OPEN_PROPS }
        document::Link { rel: "stylesheet", href: STYLE }
        Router::<Route> {}
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Timeline,
    Notes,
    Tasks,
}

impl Mode {
    fn icon(self) -> IconKind {
        match self {
            Mode::Timeline => IconKind::Lightning,
            Mode::Notes => IconKind::NotePencil,
            Mode::Tasks => IconKind::CheckSquare,
        }
    }
}

#[component]
fn AppLayout() -> Element {
    let mut menu_open = use_signal(|| false);
    let route: Route = use_route();

    let current_mode = match &route {
        Route::Timeline {} => Mode::Timeline,
        Route::Notes {} => Mode::Notes,
        Route::Tasks {} => Mode::Tasks,
    };

    rsx! {
        div { class: "app",
            header { class: "header",
                button {
                    class: "toggle-btn",
                    onclick: move |_| menu_open.toggle(),
                    Icon { kind: IconKind::List, size: 24 }
                }
                Icon { kind: current_mode.icon(), size: 20 }
            }

            if menu_open() {
                nav { class: "toggle-menu",
                    Link {
                        to: Route::Timeline {},
                        class: if current_mode == Mode::Timeline { "menu-item active" } else { "menu-item" },
                        onclick: move |_| menu_open.set(false),
                        Icon { kind: IconKind::Lightning }
                        "Timeline"
                    }
                    Link {
                        to: Route::Notes {},
                        class: if current_mode == Mode::Notes { "menu-item active" } else { "menu-item" },
                        onclick: move |_| menu_open.set(false),
                        Icon { kind: IconKind::NotePencil }
                        "Notes"
                    }
                    Link {
                        to: Route::Tasks {},
                        class: if current_mode == Mode::Tasks { "menu-item active" } else { "menu-item" },
                        onclick: move |_| menu_open.set(false),
                        Icon { kind: IconKind::CheckSquare }
                        "Tasks"
                    }
                }
            }

            main { class: "memo-area",
                Outlet::<Route> {}
            }
        }
    }
}
