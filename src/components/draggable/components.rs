use crate::components::draggable::*;
use crate::components::layout::Container;
use dioxus::prelude::*;
use dioxus_sdk::utils::window::{use_window_resize_status, use_window_size, WindowSizeWithStatus};

#[component]
pub fn DragArea(active: bool, children: Element) -> Element {
    let global_drag_info = use_context_provider(|| Signal::new(GlobalDragState::new()));

    let style = global_drag_info.read().get_drag_area_style();

    rsx! {
        div {
            style: style,
            onpointermove: move |event| DraggableStateController::update_drag_area(event, global_drag_info, active),
            onpointerup: move |_| DraggableStateController::stop_drag(global_drag_info),
            Container {
                columns: 8,
                {children},
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DraggableVariants {
    DOCKED,
    FLOATING((f64, f64)),
}

#[component]
pub fn Draggable(
    variant: DraggableVariants,
    title: String,
    style_opt: Option<String>,
    children: Element,
) -> Element {
    let id = use_signal(|| uuid::Uuid::new_v4().to_string());
    let local_drag_info = use_context_provider(|| Signal::new(LocalDragState::new(variant, id())));
    let global_drag_info = use_context::<Signal<GlobalDragState>>();

    //let window_size_info = use_window_size();

    // use_memo(move || {
    //     let _ctx = window_size_info();
    //     local_drag_info.write().resize_snapped();
    // });
    let window_size_info = use_window_resize_status();
    if let WindowSizeWithStatus::Resized(_new_size) = window_size_info {
        DraggableStateController::update_draggables_on_window_resize(local_drag_info);
    }

    let mut style =
        DraggableStateController::update_draggable_position(local_drag_info, global_drag_info);

    if let Some(styles) = style_opt {
        style = format!("{}\n{}", styles, style);
    }

    rsx! {
        div {
            style: style,
            id: id,
            DragHandle {
                title: id,
            }
            {children}
        }
    }
}

const DRAG_HANDLE_STYLES: &str = r#"
    width: 100%;
    height: 2rem;
    position: relative;
    background-color: var(--hint);
    cursor: grab;
    border: 0.05rem solid var(--fg);
    box-sizing: border-box;
    border-radius: inherit;
    flex-basis: 2rem;
    flex-shrink: 0;
    flex-grow: 1;

    display: flex;
    align-items: center;
    padding-left: .5rem;
    text-transform: uppercase;
"#;

#[component]
fn DragHandle(title: String) -> Element {
    let global_drag_info = use_context::<Signal<GlobalDragState>>();
    let local_drag_info = use_context::<Signal<LocalDragState>>();

    rsx! {
        div {
            style: DRAG_HANDLE_STYLES,
            onpointerdown: move |event| DraggableStateController::start_drag(event, global_drag_info, local_drag_info),
            "{title}",
        }
    }
}
