use crate::components::draggable::*;
use crate::components::layout::Container;
use animatable::components::Animatable;
use animatable::controllers::AnimationController;
use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::Rect;
use dioxus_sdk::utils::window::use_window_size;

#[component]
pub fn DragArea(active: bool, children: Element) -> Element {
    let global_drag_info = use_context_provider(|| Signal::new(GlobalDragState::new()));

    let style = use_memo(move || global_drag_info.read().get_drag_area_style());

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
    style: Option<String>,
    children: Element,
) -> Element {
    let id = use_signal(|| uuid::Uuid::new_v4().to_string());
    let mut local_drag_info =
        use_context_provider(|| Signal::new(LocalDragState::new(variant, id())));
    let global_drag_info = use_context::<Signal<GlobalDragState>>();
    let mut animation_controller = use_signal(|| AnimationController::default());

    use_effect(move || {
        let rect = animation_controller.read().get_rect();
        if local_drag_info.peek().get_rect() != rect {
            local_drag_info.write().set_rect(rect);
        }
    });

    let window_size_info = use_window_size();

    use_effect(move || {
        let _trigger = window_size_info.read();
        local_drag_info.write().resize_snapped();
    });

    use_effect(move || {
        let global = global_drag_info.read().get_drag_state();
        local_drag_info.write().update_state(global);
    });

    let display_state: String = use_memo(move || {
        let mut display_state = local_drag_info
            .read()
            .get_render_data(global_drag_info.read().get_drag_state());

        display_state
            .rect
            .map(|rect| animation_controller.write().set_rect(rect));
        if let Some(user_style) = &style {
            display_state.style = format!("{}\n{}", display_state.style, user_style);
        }
        display_state.style
    })
    .to_string();
    tracing::info!("{}", &display_state);
    rsx! {
        Animatable {
            controller: animation_controller,
            style: display_state,
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
