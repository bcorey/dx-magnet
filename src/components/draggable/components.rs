use std::collections::HashMap;

use crate::components::draggable::*;
use crate::components::layout::Container;
use animatable::components::Animatable;
use animatable::controllers::AnimationController;
use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::Rect;
use dioxus_logger::init;
use dioxus_sdk::utils::window::use_window_size;

#[component]
pub fn DragArea(active: bool, children: Element) -> Element {
    let mut global_drag_info = use_context_provider(|| Signal::new(GlobalDragState::new()));
    let drag_area_grid =
        use_context_provider(|| Signal::new(HashMap::new() as HashMap<String, Rect<f64, f64>>));
    let style = use_memo(move || global_drag_info.read().get_drag_area_style());

    let mut on_pointer_move = move |event: PointerEvent| {
        if !active || !global_drag_info.peek().is_dragging() {
            return;
        }
        let point = event.data.client_coordinates();
        global_drag_info.write().update_drag(point.cast_unit());
    };

    rsx! {
        div {
            style: style,
            onpointermove: move |event| on_pointer_move(event),
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
    let global_drag_info: Signal<GlobalDragState> = use_context::<Signal<GlobalDragState>>();
    let drag_area_grid = use_context::<Signal<HashMap<String, Rect<f64, f64>>>>();
    let mut animation_controller = use_signal(|| AnimationController::default());
    let current_rect = use_memo(move || animation_controller.read().get_rect());
    let animation_is_active = use_memo(move || !animation_controller.read().is_finished());

    let initial_snap_info = use_context::<Signal<Option<SnapInfo>>>();

    // should only write to local state once the targets are mounted
    use_effect(move || {
        if let Some(snap) = initial_snap_info() {
            local_drag_info.write().initialize(snap);
        }
    });

    use_effect(move || {
        let grid = drag_area_grid();
        local_drag_info.write().resize_snapped(&grid);
    });

    use_effect(move || {
        if animation_is_active() {
            tracing::info!("no update for draggable state while animation is active");
            return;
        }
        let rect = match current_rect.read().clone() {
            Some(rect) => rect,
            None => {
                tracing::error!("no current rect for draggable");
                return;
            }
        };
        let global = global_drag_info.read().get_drag_state();
        local_drag_info.write().update_state(global, rect);
    });

    let mut send_position_data = move |position_data: DraggablePositionData| {
        if !animation_controller.peek().is_finished() {
            return;
        }

        match position_data {
            DraggablePositionData::Anim(anim) => {
                tracing::info!("ordering animation {:?}", anim.clone());
                animation_controller.write().play_now(anim);
            }
            DraggablePositionData::Rect(rect)
                if animation_controller.peek().is_finished()
                    && (animation_controller.peek().get_rect().is_none()
                        || animation_controller
                            .peek()
                            .get_rect()
                            .is_some_and(|controller_rect| controller_rect != rect)) =>
            {
                animation_controller.write().set_rect(rect);
                tracing::info!("set rect to:{:?}", rect.origin);
            }
            _ => (),
        };
    };

    let display_state: String = use_memo(move || {
        let global_state = global_drag_info.peek().get_drag_state();
        let rect = current_rect
            .peek()
            .clone()
            .map_or(Rect::zero(), |rect| rect);
        let mut display_state = local_drag_info.read().get_render_data(global_state, rect);
        send_position_data(display_state.position_data.clone());
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
    overflow: hidden;
    display: flex;
    align-items: center;
    padding-left: .5rem;
    text-transform: uppercase;
"#;

#[component]
fn DragHandle(title: String) -> Element {
    let mut global_drag_info = use_context::<Signal<GlobalDragState>>();
    let mut local_drag_info = use_context::<Signal<LocalDragState>>();

    let mut start_drag = move |event: Event<PointerData>| {
        let valid_drag = local_drag_info
            .write()
            .start_drag(event.data.element_coordinates());

        if let Ok(grab_data) = valid_drag {
            global_drag_info.write().start_drag(DragAreaActiveDragData {
                current_pos: event.data.client_coordinates().cast_unit(),
                starting_data: grab_data.drag_origin,
            });
        }
    };

    rsx! {
        div {
            style: DRAG_HANDLE_STYLES,
            onpointerdown: move |event| start_drag(event),
            "{title}",
        }
    }
}
