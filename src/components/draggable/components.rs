use std::rc::Rc;

use crate::components::layout::Container;
use crate::components::{draggable::*, Window};
use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::Rect;
use dx_flipbook::components::Animatable;
use dx_flipbook::hooks::use_flipbook_signal;

const DRAG_AREA_COLUMNS: u8 = 8u8;
const DRAG_AREA_ROWS: u8 = 2u8;

#[component]
pub fn DragArea(active: bool, children: Element) -> Element {
    let mut global_drag_info = use_context_provider(|| Signal::new(GlobalDragState::new()));

    let mut grid =
        use_context_provider(|| Signal::new(GridData::new(DRAG_AREA_COLUMNS, DRAG_AREA_ROWS)));
    let mut mounted = use_signal(|| None as Option<Rc<MountedData>>);
    let read_area_rect = move || async move {
        tracing::info!("reading area rect");
        let read = mounted.peek();
        let client_rect = read.as_ref().map(|el| el.get_client_rect());

        if let Some(client_rect) = client_rect {
            if let Ok(rect) = client_rect.await {
                let old_rect = grid.peek().get_grid_rect();
                tracing::info!("old area: {:?} vs new: {:?}", old_rect, rect);
                if old_rect != Some(rect.cast_unit()) {
                    tracing::info!("setting area rect");
                    grid.write().update_mounted(rect.cast_unit());
                }
            }
        }
    };
    // implement window size adjustments after dx 0.6
    // let window_size = use_window_size();
    // use_effect(move || {
    //     let _window_size = window_size.read();
    //     if let Some(rect) = grid.peek().get_grid_rect() {
    //         spawn(async move {
    //             read_area_rect().await;
    //         });
    //     }
    // });
    use_effect(move || {
        let _trig = mounted();
        spawn(async move {
            read_area_rect().await;
        });
    });

    let mut on_pointer_move = move |event: PointerEvent| {
        if !active || !global_drag_info.peek().is_dragging() {
            return;
        }
        let point = event.data.client_coordinates();
        global_drag_info.write().update_drag(point.cast_unit());
    };
    let style = use_memo(move || global_drag_info.read().get_drag_area_style());

    rsx! {
        div {
            style: style,
            onpointermove: move |event| on_pointer_move(event),
            onpointerup: move |_| DraggableStateController::stop_drag(global_drag_info),
            onmounted: move |cx| mounted.set(Some(cx.data())),
            Container {
                columns: 8,
                {children}
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
    handle: Option<Element>,
) -> Element {
    let id = use_signal(|| uuid::Uuid::new_v4().to_string());
    let mut local_drag_info =
        use_context_provider(|| Signal::new(LocalDragState::new(variant, id())));
    let global_drag_info: Signal<GlobalDragState> = use_context::<Signal<GlobalDragState>>();
    let mut animation_controller = use_flipbook_signal();
    let current_rect = use_memo(move || animation_controller.read().read_rect());
    let animation_is_active = use_memo(move || !animation_controller.read().read_is_finished());

    let initial_snap_info = use_context::<Signal<Option<SnapInfo>>>();

    // should only write to local state once the targets are mounted
    use_effect(move || {
        if let Some(snap) = initial_snap_info() {
            if local_drag_info.peek().get_is_uninitialized() {
                local_drag_info.write().initialize(snap);
            }
        }
    });

    // implement resizing on window resize after dx 0.6 release
    // use_window_size() currently does not cooperate with dx core unless on master
    // let grid = use_context::<Signal<GridData>>();
    // use_effect(move || {
    //     let read = grid.read();
    //     local_drag_info.write().resize_snapped(read.clone());
    // });

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
        if !animation_controller.peek().peek_is_finished() {
            return;
        }

        match position_data {
            DraggablePositionData::Anim(anim) => {
                tracing::info!("ordering animation {:?}", anim.clone());
                animation_controller.write().play_now(anim);
            }
            DraggablePositionData::Rect(rect)
                if animation_controller.peek().peek_is_finished()
                    && (animation_controller.peek().peek_rect().is_none()
                        || animation_controller
                            .peek()
                            .peek_rect()
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
            display_state.style = format!("{}\n {}", display_state.style, user_style);
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
                {handle}
            }
            Window {
                //StateLogger{}
                {children}
            }
        }
    }
}

const DRAG_HANDLE_STYLES: &str = "
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
";

#[component]
fn DragHandle(title: String, children: Element) -> Element {
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

#[component]
fn StateLogger() -> Element {
    let local_drag_info = use_context::<Signal<LocalDragState>>();
    let debug = use_memo(move || {
        let state = local_drag_info.read().get_drag_state();
        format!("{:?}", state)
    });
    rsx! {
        div {
            style: "text-align: left; font-size: .5rem; max-width: 100%; overflow: hidden; max-height: 100%;",
            "{debug}"
        }
    }
}
