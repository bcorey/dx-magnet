use crate::components::draggable::*;
use dioxus::prelude::*;

#[component]
pub fn DragArea(active: bool, children: Element) -> Element {
    let global_drag_info = use_context_provider(|| Signal::new(GlobalDragState::new()));

    let style = global_drag_info.read().get_drag_area_style();

    rsx! {
        div {
            style: style,
            onpointermove: move |event| DraggableStateController::update_drag_area(event, global_drag_info, active),
            onpointerup: move |_| DraggableStateController::stop_drag(global_drag_info),
            {children},
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum DraggableVariants {
    DOCKED,
    FLOATING((f64, f64)),
}

#[component]
pub fn Draggable(
    variant: DraggableVariants,
    style_opt: Option<String>,
    children: Element,
) -> Element {
    let id = use_signal(|| uuid::Uuid::new_v4().to_string());
    let local_drag_info = use_context_provider(|| Signal::new(LocalDragState::new(variant, id())));
    let global_drag_info = use_context::<Signal<GlobalDragState>>();
    let mut style =
        DraggableStateController::update_draggable_position(local_drag_info, global_drag_info);

    if let Some(styles) = style_opt {
        style = format!("{}\n{}", styles, style);
    }

    rsx! {
        div {
            style: style,
            id: id,
            DragHandle {}
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
    align-self: flex-start;
    flex-basis: 2rem;
    flex-shrink: 0;
    flex-grow: 1;
"#;

#[component]
fn DragHandle() -> Element {
    let global_drag_info = use_context::<Signal<GlobalDragState>>();
    let local_drag_info = use_context::<Signal<LocalDragState>>();

    rsx! {
        div {
            style: DRAG_HANDLE_STYLES,
            onpointerdown: move |event| DraggableStateController::start_drag(event, global_drag_info, local_drag_info),
        }
    }
}
