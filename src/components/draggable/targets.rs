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

    let mut target_div = use_signal(|| None as Option<Rc<MountedData>>);
    let mut target_rect = use_signal(|| None as Option<Rect<f64, f64>>);

    let read_target_rect = move || async move {
        tracing::info!("reading target rect");
        let read = target_div.read();
        let client_rect = read.as_ref().map(|el| el.get_client_rect());

        if let Some(client_rect) = client_rect {
            if let Ok(rect) = client_rect.await {
                tracing::info!("setting target rect");
                target_rect.set(Some(rect));
            }
        }
    };

    use_effect(move || {
        let _trig = target_div.read();
        spawn(async move {
            read_target_rect().await;
        });
    });

    let window_size = use_window_size();
    use_effect(move || {
        let _trigger = window_size.read();
        spawn(async move {
            read_target_rect().await;
        });
    });

    let target_is_active = use_memo(move || {
        let drag_state = global_drag_state.read().get_drag_state();
        let target_rect = match target_rect.peek().clone() {
            Some(rect) => rect,
            None => return false,
        };
        match drag_state {
            DragAreaStates::Dragging(drag_info) => {
                target_rect.contains(drag_info.current_pos.cast_unit())
            }
            _ => false,
        }
    });

    use_effect(move || {
        let active = target_is_active();
        let rect = match target_rect.peek().clone() {
            Some(rect) => rect,
            None => return,
        };
        if active {
            let snap_info = SnapInfo::new(Some(id.peek().clone()), rect);
            global_drag_state.write().set_snap_info(Some(snap_info))
        } else {
            let drag_state = global_drag_state.peek().get_drag_state();
            match drag_state {
                DragAreaStates::Dragging(_drag_info) => {
                    let info_opt = global_drag_state.peek().get_snap_info();
                    if let Some(info) = info_opt {
                        if info.rect == rect {
                            global_drag_state.write().set_snap_info(None);
                        }
                    }
                }
                _ => (),
            }
        }
    });

    let style = use_memo(move || match target_is_active() {
        true => format!("{}{}", DRAG_TARGET_STYLE, DRAG_TARGET_ACTIVE_STYLE),
        false => DRAG_TARGET_STYLE.to_string(),
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
