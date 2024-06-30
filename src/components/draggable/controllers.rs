use dioxus::prelude::*;
use dioxus_elements::geometry::{euclid::Point2D, ClientSpace, ElementSpace};
use web_sys::DomRect;

use crate::components::{dom_utilities::get_element_by_id, DraggableVariants};

#[derive(Clone, PartialEq, Debug)]
pub enum DragAreaStates {
    Initial,
    Dragging(DragAreaActiveDragData),
    Released(DragEndings),
}

#[derive(Clone, PartialEq, Debug)]
pub struct DragAreaActiveDragData {
    pub current_pos: Point2D<f64, ClientSpace>,
    pub starting_data: DragOrigin,
}

impl DragAreaActiveDragData {
    pub fn update_current_pos(&mut self, new_pos: Point2D<f64, ClientSpace>) {
        self.current_pos = new_pos;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DragEndings {
    Snapping(SnapInfo),
    Releasing(Point2D<f64, ClientSpace>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum DragOrigin {
    Snapped(SnapInfo),
    Free(RectData),
}

impl DragOrigin {
    fn get_snap_info(&self) -> SnapInfo {
        match self {
            Self::Snapped(snap_info) => snap_info.clone(),
            Self::Free(rect) => SnapInfo::new(None, *rect),
        }
    }
}

pub struct DraggableStateController;

impl DraggableStateController {
    pub fn start_drag(
        event: PointerEvent,
        mut global_drag_info: Signal<GlobalDragState>,
        mut local_drag_info: Signal<LocalDragState>,
    ) {
        let valid_drag = local_drag_info
            .write()
            .start_drag(event.data.element_coordinates());

        if let Ok(grab_data) = valid_drag {
            global_drag_info.write().start_drag(DragAreaActiveDragData {
                current_pos: event.data.client_coordinates(),
                starting_data: grab_data.drag_origin,
            });
        }
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

    pub fn update_draggables_on_window_resize(mut local_drag_info: Signal<LocalDragState>) {
        local_drag_info.write().resize_snapped();
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
    snap_info: Option<SnapInfo>,
}

impl GlobalDragState {
    pub fn new() -> Self {
        Self {
            drag_state: DragAreaStates::Initial,
            snap_info: None,
        }
    }

    pub fn get_drag_state(&self) -> DragAreaStates {
        self.drag_state.clone()
    }

    pub fn set_snap_info(&mut self, info: Option<SnapInfo>) {
        self.snap_info = info;
    }

    pub fn get_snap_info(&self) -> Option<SnapInfo> {
        return self.snap_info.clone();
    }

    fn stop_drag(&mut self) {
        if let DragAreaStates::Dragging(drag_data) = self.drag_state.clone() {
            self.drag_state = match self.snap_info.clone() {
                Some(info) => DragAreaStates::Released(DragEndings::Snapping(info)),
                None => DragAreaStates::Released(DragEndings::Releasing(drag_data.current_pos)),
            };
        }
    }

    fn is_dragging(&self) -> bool {
        match self.drag_state {
            DragAreaStates::Dragging(_) => true,
            _ => false,
        }
    }

    fn start_drag(&mut self, drag_data: DragAreaActiveDragData) -> &mut Self {
        if let DragAreaStates::Initial | DragAreaStates::Released(_) = self.drag_state {
            self.drag_state = DragAreaStates::Dragging(drag_data);
        }
        self
    }

    fn update_drag(&mut self, pos: Point2D<f64, ClientSpace>) {
        if let DragAreaStates::Dragging(mut drag_data) = self.drag_state.clone() {
            drag_data.update_current_pos(pos);
            self.drag_state = DragAreaStates::Dragging(drag_data);
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
    box-shadow: 0 0 solid var(--hint);
    z-index: 100;
"#;

const TRANSITIONING_DRAGGABLE_STYLES: &str = r#"
    transition: left .1s ease-in-out, top .1s ease-in-out, width .1s ease-in-out .1s, height .1s ease-in-out .1s, box-shadow .1s ease-in-out;
"#;

#[derive(Clone, Debug)]
enum DraggableStates {
    Grabbed(DraggableGrabData),
    Resting(DraggableRestStates),
}

#[derive(Clone, Debug)]
struct DraggableGrabData {
    grab_point: Point2D<f64, ElementSpace>,
    drag_origin: DragOrigin,
}

#[derive(Clone, Debug)]
enum DraggableRestStates {
    Initial,
    Released(RectData),
    Snapped(DraggableSnapStates),
}

#[derive(Clone, PartialEq, Debug)]
enum DraggableSnapStates {
    Preview(DraggableTransitionData),
    Final(SnapInfo),
    Transitioning(DraggableTransitionData),
}

#[derive(Clone, PartialEq, Debug)]
struct DraggableTransitionData {
    from: SnapInfo,
    to: SnapInfo,
    mode: DraggableTransitionMode,
}

impl DraggableTransitionData {
    fn reverse(&self) -> DraggableTransitionData {
        DraggableTransitionData {
            from: self.to.clone(),
            to: self.from.clone(),
            mode: self.mode.reverse(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
enum DraggableTransitionMode {
    Avoidance,
    Resting,
}

impl DraggableTransitionMode {
    fn reverse(&self) -> Self {
        match self {
            Self::Avoidance => Self::Resting,
            Self::Resting => Self::Avoidance,
        }
    }
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

    pub fn is_overlapping(&self, other: RectData) -> bool {
        self.get_is_within_bounds(other.position)
    }

    pub fn from_bounding_box(web_sys_data: DomRect) -> Self {
        let position: Point2D<f64, ClientSpace> = Point2D::new(web_sys_data.x(), web_sys_data.y());
        let size: Point2D<f64, ClientSpace> =
            Point2D::new(web_sys_data.width(), web_sys_data.height());
        Self { position, size }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapInfo {
    rect: RectData,
    target_id: Option<String>,
}

impl SnapInfo {
    pub fn new(target_id: Option<String>, rect: RectData) -> Self {
        Self { rect, target_id }
    }
}

#[derive(Debug, Clone)]
pub struct DragError(DragErrorType);

#[derive(Debug, Clone)]
pub enum DragErrorType {
    IllegalDragStart,
    DomRetrievalError,
}

#[derive(Debug)]
pub struct LocalDragState {
    drag_state: DraggableStates,
    draggable_variant: DraggableVariants,
    id: String,
}

impl LocalDragState {
    pub fn new(variant: DraggableVariants, id: String) -> Self {
        Self {
            drag_state: DraggableStates::Resting(DraggableRestStates::Initial),
            draggable_variant: variant,
            id,
        }
    }

    pub fn get_element_id(&self) -> String {
        self.id.clone()
    }

    pub fn resize_snapped(&mut self) {
        //tracing::info!("resizing {}", self.id);
        if let DraggableStates::Resting(DraggableRestStates::Snapped(snap_state)) = &self.drag_state
        {
            let new_snap_info = match snap_state {
                DraggableSnapStates::Final(snap_info) => Self::resize_snapped_helper(snap_info),
                DraggableSnapStates::Preview(preview_data) => {
                    Self::resize_snapped_helper(&preview_data.to)
                }
                DraggableSnapStates::Transitioning(transition) => {
                    Self::resize_snapped_helper(&transition.to)
                }
            };
            self.drag_state = DraggableStates::Resting(DraggableRestStates::Snapped(
                DraggableSnapStates::Final(new_snap_info),
            ));
            tracing::info!("new drag state: {:?}", self.drag_state);
        }
    }

    fn resize_snapped_helper(snap_info: &SnapInfo) -> SnapInfo {
        if let Some(target_id) = snap_info.target_id.clone() {
            let dom_rect = get_element_by_id(target_id.as_str())
                .unwrap()
                .get_bounding_client_rect();
            let rect = RectData::from_bounding_box(dom_rect);
            return SnapInfo::new(snap_info.target_id.clone(), rect);
        }
        snap_info.clone()
    }

    fn start_drag(
        &mut self,
        grab_point: Point2D<f64, ElementSpace>,
    ) -> Result<DraggableGrabData, DragError> {
        let drag_origin = match self.drag_state.clone() {
            DraggableStates::Resting(rest) => match rest {
                DraggableRestStates::Snapped(snap_data) => match snap_data {
                    DraggableSnapStates::Final(final_snap) => DragOrigin::Snapped(final_snap),
                    DraggableSnapStates::Transitioning(transition) => {
                        DragOrigin::Snapped(transition.to)
                    }
                    _ => {
                        tracing::error!("bad start: {:?}", snap_data);
                        return Err(DragError(DragErrorType::IllegalDragStart));
                    }
                },
                _ => {
                    let rect = self.get_rect();
                    DragOrigin::Free(rect)
                }
            },
            _ => return Err(DragError(DragErrorType::IllegalDragStart)),
        };
        let grab_data = DraggableGrabData {
            grab_point,
            drag_origin,
        };
        self.drag_state = DraggableStates::Grabbed(grab_data.clone());
        Ok(grab_data)
    }

    pub fn get_rect(&self) -> RectData {
        let el = get_element_by_id(&self.id).unwrap();
        let rect = el.get_bounding_client_rect();
        let position: Point2D<f64, ClientSpace> = Point2D::new(rect.x(), rect.y());
        let size: Point2D<f64, ClientSpace> = Point2D::new(rect.width(), rect.height());
        RectData { position, size }
    }

    fn poll_transition(&mut self, transition: DraggableTransitionData) -> DraggableSnapStates {
        let rect = self.get_rect();
        if transition.to.rect != rect {
            return DraggableSnapStates::Transitioning(transition);
        }

        match transition.mode {
            DraggableTransitionMode::Avoidance => DraggableSnapStates::Preview(transition),
            DraggableTransitionMode::Resting => DraggableSnapStates::Final(transition.to),
        }
    }

    fn update_state(&mut self, global_drag_state: DragAreaStates) {
        match (self.drag_state.clone(), global_drag_state) {
            (
                DraggableStates::Resting(draggable_rest_state),
                DragAreaStates::Dragging(drag_area_dragging_state),
            ) => self.update_state_while_other_is_dragged(
                draggable_rest_state,
                drag_area_dragging_state,
            ),
            (
                DraggableStates::Resting(DraggableRestStates::Snapped(snap_state)),
                DragAreaStates::Released(drag_end_data),
            ) => self.update_state_on_other_drag_end(snap_state, drag_end_data),
            (
                DraggableStates::Grabbed(draggable_grab_data),
                DragAreaStates::Released(drag_area_dragging_state),
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
            DragEndings::Releasing(pointer_position) => {
                self.get_drag_end_release_state(pointer_position, draggable_grab_data)
            }
            DragEndings::Snapping(snap_data) => {
                self.get_drag_end_snap_state(snap_data, draggable_grab_data)
            }
        };
        tracing::info!("data on drag end:  {:?}", self.drag_state);
    }

    fn get_drag_end_release_state(
        &self,
        pointer_position: Point2D<f64, ClientSpace>,
        draggable_grab_data: DraggableGrabData,
    ) -> DraggableStates {
        let x = pointer_position.x - draggable_grab_data.grab_point.x;
        let y = pointer_position.y - draggable_grab_data.grab_point.y;
        let resting_position: Point2D<f64, ClientSpace> = Point2D::new(x, y);
        let resting_size = self.get_rect().size;
        DraggableStates::Resting(DraggableRestStates::Released(RectData {
            position: resting_position,
            size: resting_size,
        }))
    }

    fn get_drag_end_snap_state(
        &self,
        snap_data: SnapInfo,
        draggable_grab_data: DraggableGrabData,
    ) -> DraggableStates {
        let from = match draggable_grab_data.drag_origin {
            DragOrigin::Free(rect) => rect,
            DragOrigin::Snapped(og_snap) => og_snap.rect,
        };
        DraggableStates::Resting(DraggableRestStates::Snapped(
            DraggableSnapStates::Transitioning(DraggableTransitionData {
                from: SnapInfo::new(None, from),
                to: snap_data,
                mode: DraggableTransitionMode::Resting,
            }),
        ))
    }

    fn update_state_on_other_drag_end(
        &mut self,
        snap_state: DraggableSnapStates,
        drag_end_data: DragEndings,
    ) {
        let new_snap_state = match (snap_state.clone(), drag_end_data) {
            (
                DraggableSnapStates::Preview(preview_data),
                DragEndings::Snapping(other_snap_info),
            ) if other_snap_info.rect.is_overlapping(preview_data.from.rect) => {
                match other_snap_info.rect.is_overlapping(preview_data.from.rect) {
                    true => DraggableSnapStates::Transitioning(preview_data),
                    false => DraggableSnapStates::Transitioning(preview_data.reverse()),
                }
            }
            (DraggableSnapStates::Preview(preview_data), DragEndings::Releasing(_)) => {
                DraggableSnapStates::Transitioning(preview_data.reverse())
            }
            (_, _) => snap_state,
        };

        self.drag_state = DraggableStates::Resting(DraggableRestStates::Snapped(new_snap_state));
    }

    fn update_state_while_other_is_dragged(
        &mut self,
        draggable_rest_state: DraggableRestStates,
        drag_area_dragging_state: DragAreaActiveDragData,
    ) {
        let this_rect = match draggable_rest_state.clone() {
            DraggableRestStates::Initial => self.get_rect(),
            DraggableRestStates::Released(rect) => rect,
            DraggableRestStates::Snapped(snap_state) => match snap_state {
                DraggableSnapStates::Final(rect) => rect.rect,
                DraggableSnapStates::Preview(transition) => transition.to.rect,
                DraggableSnapStates::Transitioning(transition) => transition.to.rect,
            },
        };
        let intersects_this_rect =
            this_rect.get_is_within_bounds(drag_area_dragging_state.current_pos);
        match (draggable_rest_state.clone(), intersects_this_rect) {
            (DraggableRestStates::Initial, _) => {
                let snap_state = DraggableSnapStates::Final(SnapInfo::new(None, this_rect));
                self.get_next_snap_state(snap_state, intersects_this_rect, drag_area_dragging_state)
            }
            (DraggableRestStates::Released(_), _) => (), // no action
            (DraggableRestStates::Snapped(snap_state), _) => {
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
        let snap_state = match (snap_state.clone(), intersects_pointer) {
            (DraggableSnapStates::Final(info), true) => {
                let start_snap = drag_area_dragging_state.starting_data.get_snap_info();
                DraggableSnapStates::Transitioning(DraggableTransitionData {
                    from: info,
                    to: start_snap,
                    mode: DraggableTransitionMode::Avoidance,
                })
            }
            (DraggableSnapStates::Preview(transition), true) => {
                DraggableSnapStates::Transitioning(transition.reverse())
            }
            (DraggableSnapStates::Preview(transition), false)
                if !transition
                    .from
                    .rect
                    .get_is_within_bounds(drag_area_dragging_state.current_pos) =>
            {
                DraggableSnapStates::Transitioning(transition.reverse())
            }
            (DraggableSnapStates::Transitioning(transition), _) => self.poll_transition(transition),
            (_, false) => snap_state,
        };

        self.drag_state = DraggableStates::Resting(DraggableRestStates::Snapped(snap_state));
    }

    fn get_render_data(&self, global_drag_state: DragAreaStates) -> String {
        match (self.drag_state.clone(), global_drag_state.clone()) {
            (DraggableStates::Grabbed(grab_data), DragAreaStates::Dragging(drag_data)) => {
                self.location_with_grab_offset(grab_data.grab_point, drag_data.current_pos)
            }
            (DraggableStates::Resting(rest_state), DragAreaStates::Released(_)) => {
                self.get_render_data_for_resting_states(rest_state)
            }
            (
                DraggableStates::Resting(DraggableRestStates::Snapped(snap_state)),
                DragAreaStates::Dragging(drag_area_dragging_state),
            ) => self.get_render_data_for_avoidance_states(snap_state),
            (DraggableStates::Resting(DraggableRestStates::Initial), DragAreaStates::Initial) => {
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
            DraggableRestStates::Initial => self.initial_style(),
            DraggableRestStates::Released(release_rect) => self.location(release_rect.position),
            DraggableRestStates::Snapped(snap_state) => {
                self.get_render_data_for_avoidance_states(snap_state)
            }
        }
    }

    fn get_render_data_for_avoidance_states(
        &self,
        draggable_snap_state: DraggableSnapStates,
    ) -> String {
        match draggable_snap_state {
            DraggableSnapStates::Final(rect) => self.snapped_style(rect.rect),
            DraggableSnapStates::Preview(transition) => self.snapped_style(transition.to.rect),
            DraggableSnapStates::Transitioning(transition) => {
                self.snapped_transitioning_style(transition.to.rect)
            }
        }
    }

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

    fn snapped_transitioning_style(&self, rect: RectData) -> String {
        format!(
            "{} {}",
            self.snapped_style(rect),
            TRANSITIONING_DRAGGABLE_STYLES
        )
    }

    fn initial_style(&self) -> String {
        match self.draggable_variant {
            DraggableVariants::DOCKED => DRAGGABLE_BASE_STYLES.to_string(),
            DraggableVariants::FLOATING(_pos) => String::new(),
        }
    }
}
