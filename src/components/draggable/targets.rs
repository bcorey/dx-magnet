use std::rc::Rc;

use crate::components::draggable::*;
use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::Rect;
use dioxus_sdk::utils::window::use_window_size;

const DRAG_TARGET_STYLE: &str = r#"
    width: 100%;
    height: 100%;
"#;

const DRAG_TARGET_ACTIVE_STYLE: &str = r#"
    background-color: var(--bg);
    background-image: repeating-linear-gradient(50deg, var(--fg), var(--fg) .05rem, transparent .01rem, transparent .4rem);
"#;

#[component]
pub fn DragTarget(children: Element) -> Element {
    let id = use_signal(|| uuid::Uuid::new_v4().to_string());
    let mut global_drag_state = use_context::<Signal<GlobalDragState>>();
    let mut target_active = use_signal(|| false);

    let mut target_div = use_signal(|| None as Option<Rc<MountedData>>);
    let mut target_rect = use_signal(|| None as Option<Rect<f64, f64>>);

    let style = match target_active() {
        true => format!("{}{}", DRAG_TARGET_STYLE, DRAG_TARGET_ACTIVE_STYLE),
        false => DRAG_TARGET_STYLE.to_string(),
    };

    let read_target_rect = move || async move {
        let read = target_div.read();
        let client_rect = read.as_ref().map(|el| el.get_client_rect());

        if let Some(client_rect) = client_rect {
            if let Ok(rect) = client_rect.await {
                target_rect.set(Some(rect));
            }
        }
    };

    let window_size = use_window_size();

    use_effect(move || {
        let _trigger = window_size.read();
        spawn(async move {
            read_target_rect().await;
        });
    });

    use_effect(move || {
        let drag_state = global_drag_state.read().get_drag_state();
        if let DragAreaStates::Dragging(drag_point) = drag_state {
            if let Some(rect) = target_rect.peek().as_ref() {
                let is_inside_rect = rect.contains(drag_point.current_pos.cast_unit());
                let state_has_changed = target_active() != is_inside_rect;
                let active = target_active();
                if state_has_changed {
                    target_active.set(!active);
                }
                if state_has_changed && !active {
                    let snap_info = SnapInfo::new(Some(id()), *rect);
                    global_drag_state.write().set_snap_info(Some(snap_info));
                }
            } else {
                tracing::warn!("could not find drag target by ID");
            }
        }
    });

    rsx! {
        div {
            id: id,
            key: "{id}",
            style: style,
            onmounted: move |cx| target_div.set(Some(cx.data())),
            {children}
        }
    }
}
