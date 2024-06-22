use crate::components::{dom_utilities::get_element_by_id, draggable::*};
use dioxus::prelude::*;
use dioxus_elements::geometry::{euclid::Point2D, ClientSpace};
use web_sys::DomRect;

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

    let style = match target_active() {
        true => format!("{}{}", DRAG_TARGET_STYLE, DRAG_TARGET_ACTIVE_STYLE),
        false => DRAG_TARGET_STYLE.to_string(),
    };

    let drag_state = global_drag_state.read().get_drag_state();
    if let DragAreaStates::Dragging(drag_point) = drag_state {
        if let Ok(element) = get_element_by_id(id().as_str()) {
            let rect = element.get_bounding_client_rect();
            let is_inside_rect = is_inside_rect(&rect, drag_point.current_pos);
            let state_has_changed = target_active() != is_inside_rect;
            let active = target_active();
            if state_has_changed {
                target_active.set(!active);
            }
            if state_has_changed && !active {
                let snap_origin: Point2D<f64, ClientSpace> = Point2D::new(rect.x(), rect.y());
                let snap_size: Point2D<f64, ClientSpace> =
                    Point2D::new(rect.width(), rect.height());
                let rect = RectData {
                    position: snap_origin,
                    size: snap_size,
                };
                let snap_info = SnapInfo::new(Some(id()), rect);
                global_drag_state.write().set_snap_info(Some(snap_info));
            }
        } else {
            tracing::warn!("could not find drag target by ID");
        }
    }

    rsx! {
        div {
            id: id,
            key: "{id}",
            style: style,
            {children}
        }
    }
}

fn is_inside_rect(rect: &DomRect, point: Point2D<f64, ClientSpace>) -> bool {
    let is_in_x_bounds = point.x > rect.x() && point.x < rect.width() + rect.x();
    let is_in_y_bounds = point.y > rect.y() && point.y < rect.height() + rect.y();
    return is_in_x_bounds && is_in_y_bounds;
}
