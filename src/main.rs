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
            Container {
                columns: 8,
                Cell {
                    span: 1..2,
                    rows: 1,
                    columns: 1,
                    DragTarget{
                        Draggable {
                            variant: DraggableVariants::DOCKED,
                            style_opt: "--bg: #fec651; --fg: #55198c; --hint: #cb9262;".to_string(),
                            Window {
                                "draggable window"
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
                            Window {
                                "draggable window 2"
                            }
                        }
                    }
                    DragTarget{
                        Draggable {
                            variant: DraggableVariants::DOCKED,
                            Window {
                                "draggable window 2"
                            }
                        }
                    }
                    DragTarget{
                        Draggable {
                            variant: DraggableVariants::DOCKED,
                            Window {
                                "draggable window"
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
                            Window {
                                "draggable window 2"
                            }
                        }
                    }
                    DragTarget{}
                    DragTarget{
                        Draggable {
                            variant: DraggableVariants::DOCKED,
                            Window {
                                "draggable window"
                            }
                        }
                    }
                }
                Cell {
                    span: 6..3,
                    DragTarget{
                        Draggable {
                            variant: DraggableVariants::DOCKED,
                            Window {
                                "the really big one",
                            }
                        }
                    }
                }
            }
        }
    }
}
