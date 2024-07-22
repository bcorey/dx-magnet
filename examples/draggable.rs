#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;

use dx_magnet::components::*;

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
            // Cell {
            //     span: 1..2,
            //     rows: 1,
            //     columns: 1,
            // }
            Cell {
                span: 3..2,
                rows: 1,
                columns: 1,
                DragTarget {
                    Draggable {
                        variant: DraggableVariants::DOCKED,
                        title: "panel 1".to_string(),
                    }
                }
            }
            Cell {
                span: 3..2
            }
        }
    }
}
