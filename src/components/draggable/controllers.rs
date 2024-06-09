use dioxus::prelude::*;
use dioxus_elements::geometry::{euclid::Point2D, ClientSpace, ElementSpace};

use crate::{dom_utilities::get_element_by_id, DraggableVariants};

#[derive(Clone, PartialEq, Debug)]
pub enum DragAreaStates {
    INITIAL,
    DRAGGING(DragAreaActiveDragData),
    RELEASED(DragEndings),
}

#[derive(Clone, PartialEq, Debug)]
pub struct DragAreaActiveDragData {
    pub current_pos: Point2D<f64, ClientSpace>,
    pub starting_data: RectData,
}

impl DragAreaActiveDragData {
    pub fn update_current_pos(&mut self, new_pos: Point2D<f64, ClientSpace>) {
        self.current_pos = new_pos;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DragEndings {
    SNAPPING(RectData),
    RELEASING(Point2D<f64, ClientSpace>),
}

pub struct DraggableStateController;

impl DraggableStateController {
    pub fn start_drag(
        event: PointerEvent,
        mut global_drag_info: Signal<GlobalDragState>,
        mut local_drag_info: Signal<LocalDragState>,
    ) {
        local_drag_info
            .write()
            .start_drag(event.data.element_coordinates());
        let start_rect = local_drag_info.read().get_rect();
        global_drag_info.write().start_drag(DragAreaActiveDragData {
            current_pos: event.data.client_coordinates(),
            starting_data: start_rect,
        });
    }

    pub fn update_drag_area(
        event: PointerEvent,
        mut global_drag_info: Signal<GlobalDragState>,
        active: bool,
    ) {
        if !active || !global_drag_info.read().is_dragging() {
            return;
        }
        let point = event.data.client_coordinates();
        global_drag_info.write().update_drag(point);
    }

    pub fn update_draggable_position(
        mut local_drag_info: Signal<LocalDragState>,
        global_drag_info: Signal<GlobalDragState>,
    ) -> String {
        local_drag_info
            .write()
            .on_component_update(global_drag_info.read().get_drag_state())
    }

    pub fn stop_drag(mut global_drag_info: Signal<GlobalDragState>) {
        global_drag_info.write().stop_drag();
    }
}

const DRAG_AREA_BASE_STYLES: &str = r#"
    background-image: radial-gradient(black .05rem, transparent 0);
    background-size: .6rem .6rem;
    width: 100%;
    height: 100%;
"#;

const DRAG_AREA_ACTIVE_STYLES: &str = r#"
    -webkit-user-select: none;
    user-select: none;
"#;

#[derive(Clone)]
pub struct GlobalDragState {
    drag_state: DragAreaStates,
    snap_info: Option<(Point2D<f64, ClientSpace>, Point2D<f64, ClientSpace>)>,
}

impl GlobalDragState {
    pub fn new() -> Self {
        Self {
            drag_state: DragAreaStates::INITIAL,
            snap_info: None,
        }
    }

    pub fn get_drag_state(&self) -> DragAreaStates {
        self.drag_state.clone()
    }

    pub fn set_snap_info(
        &mut self,
        info: Option<(Point2D<f64, ClientSpace>, Point2D<f64, ClientSpace>)>,
    ) {
        self.snap_info = info;
    }

    pub fn get_snap_info(&self) -> Option<(Point2D<f64, ClientSpace>, Point2D<f64, ClientSpace>)> {
        return self.snap_info;
    }

    fn stop_drag(&mut self) {
        if let DragAreaStates::DRAGGING(drag_data) = self.drag_state.clone() {
            self.drag_state = match self.snap_info {
                Some((snap_origin, snap_size)) => {
                    DragAreaStates::RELEASED(DragEndings::SNAPPING(RectData {
                        position: snap_origin,
                        size: snap_size,
                    }))
                }
                None => DragAreaStates::RELEASED(DragEndings::RELEASING(drag_data.current_pos)),
            };
        }
    }

    fn is_dragging(&self) -> bool {
        match self.drag_state {
            DragAreaStates::DRAGGING(_) => true,
            _ => false,
        }
    }

    fn start_drag(&mut self, drag_data: DragAreaActiveDragData) -> &mut Self {
        if let DragAreaStates::INITIAL | DragAreaStates::RELEASED(_) = self.drag_state {
            self.drag_state = DragAreaStates::DRAGGING(drag_data);
        }
        self
    }

    fn update_drag(&mut self, pos: Point2D<f64, ClientSpace>) {
        if let DragAreaStates::DRAGGING(mut drag_data) = self.drag_state.clone() {
            drag_data.update_current_pos(pos);
            self.drag_state = DragAreaStates::DRAGGING(drag_data);
        }
    }

    pub fn get_drag_area_style(&self) -> String {
        match self.is_dragging() {
            true => format!("{}{}", DRAG_AREA_BASE_STYLES, DRAG_AREA_ACTIVE_STYLES),
            false => DRAG_AREA_BASE_STYLES.to_string(),
        }
    }
}

const DRAGGABLE_BASE_STYLES: &str = r#"
    display: flex;
    flex-flow: column;
    flex-direction: column;
    height: 100%;
    align-content: flex-start;
"#;

const DRAGGABLE_STYLES: &str = r#"
    position: absolute;
    background-color: var(--accent_0);
    z-index: 10000;
    width: 180px;
    height: 200px;
    box-shadow: .4rem .3rem var(--hint);
"#;

const SNAPPED_DRAGGABLE_STYLES: &str = r#"
    transition: left .1s ease-in-out, top .1s ease-in-out, width .1s ease-in-out .1s, height .1s ease-in-out .1s, box-shadow .1s ease-in-out;
    box-shadow: 0 0 solid var(--hint);
    z-index: 100;
"#;

#[derive(Clone, Copy, Debug)]
enum DraggableStates {
    GRABBED(DraggableGrabData),
    RESTING(DraggableRestStates),
}

#[derive(Clone, Copy, Debug)]
struct DraggableGrabData {
    grab_point: Point2D<f64, ElementSpace>,
    pointer_position: Point2D<f64, ClientSpace>,
}

#[derive(Clone, Copy, Debug)]
enum DraggableRestStates {
    INITIAL,
    RELEASED(RectData),
    SNAPPED(DraggableSnapStates),
}

#[derive(Clone, PartialEq, Copy, Debug)]
enum DraggableSnapStates {
    PREVIEW { from: RectData, to: RectData },
    FINAL(RectData),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct RectData {
    pub position: Point2D<f64, ClientSpace>,
    pub size: Point2D<f64, ClientSpace>,
}

impl RectData {
    pub fn get_is_within_bounds<U>(&self, point: Point2D<f64, U>) -> bool {
        (point.x >= self.position.x && point.x <= self.position.x + self.size.x)
            && (point.y >= self.position.y && point.y <= self.position.y + self.size.y)
    }

    pub fn get_is_overlapping(&self, other: RectData) -> bool {
        self.get_is_within_bounds(other.position)
    }
}

pub struct LocalDragState {
    drag_state: DraggableStates,
    draggable_variant: DraggableVariants,
    id: String,
}

impl LocalDragState {
    pub fn new(variant: DraggableVariants, id: String) -> Self {
        Self {
            drag_state: DraggableStates::RESTING(DraggableRestStates::INITIAL),
            draggable_variant: variant,
            id: id,
        }
    }

    pub fn get_element_id(&self) -> String {
        self.id.clone()
    }

    fn start_drag(&mut self, grab_point: Point2D<f64, ElementSpace>) {
        let start_location = self.get_rect().position;
        let grab_data = DraggableGrabData {
            grab_point,
            pointer_position: start_location,
        };
        self.drag_state = DraggableStates::GRABBED(grab_data);
    }

    pub fn get_rect(&self) -> RectData {
        get_element_by_id(&self.id)
            .and_then(|el: web_sys::Element| {
                let rect = el.get_bounding_client_rect();
                let position: Point2D<f64, ClientSpace> = Point2D::new(rect.x(), rect.y());
                let size: Point2D<f64, ClientSpace> = Point2D::new(rect.width(), rect.height());
                Some(RectData { position, size })
            })
            .expect(format!("Could not get draggable {} by ID via DOM.", self.id).as_str())
    }

    fn update_state(&mut self, global_drag_state: DragAreaStates) {
        match (self.drag_state.clone(), global_drag_state) {
            (
                DraggableStates::RESTING(draggable_rest_state),
                DragAreaStates::DRAGGING(drag_area_dragging_state),
            ) => self.update_state_while_other_is_dragged(
                draggable_rest_state,
                drag_area_dragging_state,
            ),
            (
                DraggableStates::RESTING(DraggableRestStates::SNAPPED(snap_state)),
                DragAreaStates::RELEASED(drag_end_data),
            ) => self.update_state_on_other_drag_end(snap_state, drag_end_data),
            (
                DraggableStates::GRABBED(draggable_grab_data),
                DragAreaStates::RELEASED(drag_area_dragging_state),
            ) => self.update_state_on_self_drag_end(draggable_grab_data, drag_area_dragging_state),
            (_, _) => (),
        };
    }

    fn update_state_on_self_drag_end(
        &mut self,
        draggable_grab_data: DraggableGrabData,
        drag_area_dragging_state: DragEndings,
    ) {
        self.drag_state = match drag_area_dragging_state {
            DragEndings::RELEASING(pointer_position) => {
                let x = pointer_position.x - draggable_grab_data.grab_point.x;
                let y = pointer_position.y - draggable_grab_data.grab_point.y;
                let resting_position: Point2D<f64, ClientSpace> = Point2D::new(x, y);
                let resting_size = self.get_rect().size;
                DraggableStates::RESTING(DraggableRestStates::RELEASED(RectData {
                    position: resting_position,
                    size: resting_size,
                }))
            }
            DragEndings::SNAPPING(snap_data) => DraggableStates::RESTING(
                DraggableRestStates::SNAPPED(DraggableSnapStates::FINAL(snap_data)),
            ),
        }
    }

    fn update_state_on_other_drag_end(
        &mut self,
        snap_state: DraggableSnapStates,
        drag_end_data: DragEndings,
    ) {
        let new_snap_state = match (snap_state, drag_end_data) {
            (DraggableSnapStates::PREVIEW { from, to }, DragEndings::SNAPPING(other_rect))
                if other_rect.get_is_overlapping(from) =>
            {
                match other_rect.get_is_overlapping(from) {
                    true => DraggableSnapStates::FINAL(to),
                    false => DraggableSnapStates::FINAL(from),
                }
            }
            (DraggableSnapStates::PREVIEW { from, .. }, DragEndings::RELEASING(_)) => {
                DraggableSnapStates::FINAL(from)
            }
            (_, _) => snap_state,
        };

        self.drag_state = DraggableStates::RESTING(DraggableRestStates::SNAPPED(new_snap_state));
    }

    fn update_state_while_other_is_dragged(
        &mut self,
        draggable_rest_state: DraggableRestStates,
        drag_area_dragging_state: DragAreaActiveDragData,
    ) {
        let this_rect = match draggable_rest_state.clone() {
            DraggableRestStates::INITIAL => self.get_rect(),
            DraggableRestStates::RELEASED(rect) => rect,
            DraggableRestStates::SNAPPED(snap_state) => match snap_state {
                DraggableSnapStates::FINAL(rect) => rect,
                DraggableSnapStates::PREVIEW { to, .. } => to,
            },
        };
        let intersects_this_rect =
            this_rect.get_is_within_bounds(drag_area_dragging_state.current_pos);
        match (draggable_rest_state.clone(), intersects_this_rect) {
            (DraggableRestStates::INITIAL, _) => {
                let snap_state = DraggableSnapStates::FINAL(this_rect);
                self.get_next_snap_state(snap_state, intersects_this_rect, drag_area_dragging_state)
            }
            (DraggableRestStates::RELEASED(_), _) => (), // no action
            (DraggableRestStates::SNAPPED(snap_state), _) => {
                self.get_next_snap_state(
                    snap_state,
                    intersects_this_rect,
                    drag_area_dragging_state,
                );
            }
        }
    }

    fn get_next_snap_state(
        &mut self,
        snap_state: DraggableSnapStates,
        intersects_pointer: bool,
        drag_area_dragging_state: DragAreaActiveDragData,
    ) {
        let snap_state = match (snap_state, intersects_pointer) {
            (DraggableSnapStates::FINAL(rect), true) => DraggableSnapStates::PREVIEW {
                from: rect,
                to: drag_area_dragging_state.starting_data,
            },
            (DraggableSnapStates::PREVIEW { from, .. }, true) => DraggableSnapStates::FINAL(from),
            (DraggableSnapStates::PREVIEW { from, .. }, false)
                if !from.get_is_within_bounds(drag_area_dragging_state.current_pos) =>
            {
                DraggableSnapStates::FINAL(from)
            }
            (_, false) => snap_state,
        };

        self.drag_state = DraggableStates::RESTING(DraggableRestStates::SNAPPED(snap_state));
    }

    fn get_render_data(&self, global_drag_state: DragAreaStates) -> String {
        match (self.drag_state, global_drag_state.clone()) {
            (DraggableStates::GRABBED(grab_data), DragAreaStates::DRAGGING(drag_data)) => {
                self.location_with_grab_offset(grab_data.grab_point, drag_data.current_pos)
            }
            (DraggableStates::RESTING(rest_state), DragAreaStates::RELEASED(_)) => {
                self.get_render_data_for_resting_states(rest_state)
            }
            (
                DraggableStates::RESTING(DraggableRestStates::SNAPPED(snap_state)),
                DragAreaStates::DRAGGING(drag_area_dragging_state),
            ) => self.get_render_data_for_avoidance_states(snap_state, drag_area_dragging_state),
            (DraggableStates::RESTING(DraggableRestStates::INITIAL), DragAreaStates::INITIAL) => {
                self.initial_style()
            }
            (_, _) => {
                tracing::error!(
                    "Illegal state when getting render data. local state: {:?}, global state: {:?}",
                    self.drag_state,
                    global_drag_state
                );
                String::new()
            }
        }
    }

    fn get_render_data_for_resting_states(
        &self,
        draggable_rest_state: DraggableRestStates,
    ) -> String {
        match draggable_rest_state {
            DraggableRestStates::INITIAL => self.initial_style(),
            DraggableRestStates::RELEASED(release_rect) => self.location(release_rect.position),
            DraggableRestStates::SNAPPED(DraggableSnapStates::FINAL(snap_rect)) => {
                self.snapped_style(snap_rect)
            }
            DraggableRestStates::SNAPPED(DraggableSnapStates::PREVIEW { to, .. }) => {
                self.snapped_style(to)
            }
        }
    }

    fn get_render_data_for_avoidance_states(
        &self,
        draggable_snap_state: DraggableSnapStates,
        _drag_area_dragging_state: DragAreaActiveDragData,
    ) -> String {
        match draggable_snap_state {
            DraggableSnapStates::FINAL(rect) => self.snapped_style(rect),
            DraggableSnapStates::PREVIEW { to, .. } => self.snapped_style(to),
        }
    }

    // runs every component update
    pub fn on_component_update(&mut self, global_drag_state: DragAreaStates) -> String {
        self.update_state(global_drag_state.clone());
        self.get_render_data(global_drag_state)
    }

    fn location_with_grab_offset(
        &self,
        drag_point: Point2D<f64, ElementSpace>,
        pointer_pos: Point2D<f64, ClientSpace>,
    ) -> String {
        let x = pointer_pos.x - drag_point.x;
        let y = pointer_pos.y - drag_point.y;

        format!(
            "{}{}\n left: {}px; top: {}px;",
            DRAGGABLE_BASE_STYLES, DRAGGABLE_STYLES, x, y
        )
    }

    fn location(&self, pos: Point2D<f64, ClientSpace>) -> String {
        format!(
            "{}{}\n left: {}px; top: {}px;",
            DRAGGABLE_BASE_STYLES, DRAGGABLE_STYLES, pos.x, pos.y
        )
    }

    fn snapped_style(&self, rect: RectData) -> String {
        format!(
            "{}{}{}\n width: {}px; height: {}px;",
            DRAGGABLE_BASE_STYLES,
            self.location(rect.position),
            SNAPPED_DRAGGABLE_STYLES,
            rect.size.x,
            rect.size.y
        )
    }

    fn initial_style(&self) -> String {
        match self.draggable_variant {
            DraggableVariants::DOCKED => DRAGGABLE_BASE_STYLES.to_string(),
            DraggableVariants::FLOATING(_pos) => String::new(),
        }
    }
}
