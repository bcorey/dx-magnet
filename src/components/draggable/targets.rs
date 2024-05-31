use crate::components::draggable::*;
use dioxus::prelude::*;
use dioxus_elements::geometry::{euclid::Point2D, ClientSpace};
use web_sys::DomRect;

const DRAG_TARGET_STYLE: &str = r#"
    width: 100%;
    height: 100%;
    background-color: var(--bg);
"#;

const DRAG_TARGET_ACTIVE_STYLE: &str = r#"
    background-color: var(--accent_1);
"#;

#[component]
pub fn DragTarget(children: Element) -> Element {
    let id = "target";
    let mut global_drag_state = use_context::<Signal<GlobalDragState>>();
    //let target_state = use_signal(|| DragTargetState::new());
    let mut target_active = use_signal(|| false);

    let style = match target_active() {
        true => format!("{}{}", DRAG_TARGET_STYLE, DRAG_TARGET_ACTIVE_STYLE),
        false => DRAG_TARGET_STYLE.to_string(),
    };

    let drag_state = global_drag_state.read().get_drag_state();
    if let DragAreaStates::DRAGGING(drag_point) = drag_state {
        if let Some(element) = get_element_by_id(id) {
            let rect = element.get_bounding_client_rect();
            let is_inside_rect = is_inside_rect(&rect, drag_point);
            let state_has_changed = target_active() != is_inside_rect;
            if state_has_changed {
                target_active.set(is_inside_rect);
            }
            if state_has_changed && target_active() {
                let snap_origin: Point2D<f64, ClientSpace> = Point2D::new(rect.x(), rect.y());
                let snap_size: Point2D<f64, ClientSpace> =
                    Point2D::new(rect.width(), rect.height());
                global_drag_state
                    .write()
                    .set_snap_info(Some((snap_origin, snap_size)));
            }
            if state_has_changed && !target_active() {
                global_drag_state.write().set_snap_info(None);
            }
        }
    }

    rsx! {
        div {
            id: id,
            style: style,
            {children}
        }
    }
}

fn get_element_by_id(id: &str) -> Option<web_sys::Element> {
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.get_element_by_id(id))
}

fn is_inside_rect(rect: &DomRect, point: Point2D<f64, ClientSpace>) -> bool {
    let is_in_x_bounds = point.x > rect.x() && point.x < rect.width() + rect.x();
    let is_in_y_bounds = point.y > rect.y() && point.y < rect.height() + rect.y();
    return is_in_x_bounds && is_in_y_bounds;
}
