use dioxus::prelude::*;

const ICON_LIGHTNING: Asset = asset!("/assets/icons/lightning.svg");
const ICON_NOTE_PENCIL: Asset = asset!("/assets/icons/note-pencil.svg");
const ICON_CHECK_SQUARE: Asset = asset!("/assets/icons/check-square.svg");
const ICON_LIST: Asset = asset!("/assets/icons/list.svg");

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IconKind {
    Lightning,
    NotePencil,
    CheckSquare,
    List,
}

impl IconKind {
    fn asset(self) -> Asset {
        match self {
            IconKind::Lightning => ICON_LIGHTNING,
            IconKind::NotePencil => ICON_NOTE_PENCIL,
            IconKind::CheckSquare => ICON_CHECK_SQUARE,
            IconKind::List => ICON_LIST,
        }
    }
}

#[component]
pub fn Icon(kind: IconKind, #[props(default = 20)] size: u32) -> Element {
    rsx! {
        img {
            src: kind.asset(),
            width: "{size}",
            height: "{size}",
            class: "icon",
        }
    }
}
