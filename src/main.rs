#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;

pub mod components;
use components::*;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        link { rel: "stylesheet", href: "styles.css" }
        DragArea {
            active: true,
            Draggable {
                Window {
                    "draggable window"
                }
            }
            Draggable {
                Window {
                    "draggable window 2"
                }
            }
            Container {
                Cell {
                    span: 1..1,
                    rows: 12,
                    columns: 1,
                    div {
                        style: "width: 100%; height: 100%; background-color: var(--hint)",
                    }
                }
                Cell {
                    span: 2..12,
                    "Hello world",
                }
            }
        }
    }
}
