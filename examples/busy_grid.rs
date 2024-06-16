#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;

use gridline::components::*;

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
            Cell {
                span: 1..2,
                rows: 1,
                columns: 1,
                DragTarget{
                    Draggable {
                        variant: DraggableVariants::DOCKED,
                        title: "panel 1".to_string(),
                        Window {
                        }
                    }
                }
            }
            Cell {
                span: 3..1,
                rows: 3,
                columns: 1,
                DragTarget{
                    Draggable {
                        variant: DraggableVariants::DOCKED,
                        title: "panel 2".to_string(),
                        Window {
                        }
                    }
                }
                DragTarget{
                    Draggable {
                        variant: DraggableVariants::DOCKED,
                        title: "panel 3".to_string(),
                        Window {
                        }
                    }
                }
                DragTarget{
                    Draggable {
                        variant: DraggableVariants::DOCKED,
                        title: "panel 4".to_string(),
                        Window {
                        }
                    }
                }
            }
            Cell {
                span: 4..2,
                rows: 3,
                columns: 1,
                DragTarget{
                    Draggable {
                        variant: DraggableVariants::DOCKED,
                        title: "panel 5".to_string(),
                        Window {
                        }
                    }
                }
                DragTarget{
                    Draggable {
                        variant: DraggableVariants::DOCKED,
                        title: "panel 6".to_string(),
                        Window {
                        }
                    }
                }
                DragTarget{
                    Draggable {
                        variant: DraggableVariants::DOCKED,
                        title: "panel 7".to_string(),
                        Window {
                        }
                    }
                }
            }
            Cell {
                span: 6..3,
                DragTarget{
                    Draggable {
                        variant: DraggableVariants::DOCKED,
                        title: "panel 8".to_string(),
                        Window {
                        }
                    }
                }
            }
        }
    }
}
