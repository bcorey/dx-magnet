use super::{DraggableTransitionData, DraggableTransitionMode};
use crate::components::{DragError, DragErrorType, DraggableVariants};
use dioxus::prelude::*;
use dioxus_elements::geometry::{
    euclid::{Point2D, Rect, Size2D},
    ClientSpace, ElementSpace,
};

#[derive(Clone, PartialEq, Debug)]
pub enum DragAreaStates {
    Initial,
    Dragging(DragAreaActiveDragData),
    Released(DragEndings),
}

#[derive(Clone, PartialEq, Debug)]
pub struct DragAreaActiveDragData {
    pub current_pos: Point2D<f64, f64>,
    pub starting_data: DragOrigin,
}

impl DragAreaActiveDragData {
    pub fn update_current_pos(&mut self, new_pos: Point2D<f64, f64>) {
        self.current_pos = new_pos;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DragEndings {
    Snapping(SnapInfo),
    Releasing(Point2D<f64, f64>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum DragOrigin {
    Snapped(SnapInfo),
    Free(Rect<f64, f64>),
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
                current_pos: event.data.client_coordinates().cast_unit(),
                starting_data: grab_data.drag_origin,
            });
        }
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
        Self::default()
    }

    pub fn get_drag_state(&self) -> DragAreaStates {
        self.drag_state.clone()
    }

    pub fn set_snap_info(&mut self, info: Option<SnapInfo>) {
        self.snap_info = info;
    }

    pub fn get_snap_info(&self) -> Option<SnapInfo> {
        self.snap_info.clone()
    }

    fn stop_drag(&mut self) {
        if let DragAreaStates::Dragging(drag_data) = self.drag_state.clone() {
            self.drag_state = match self.snap_info.clone() {
                Some(info) => DragAreaStates::Released(DragEndings::Snapping(info)),
                None => DragAreaStates::Released(DragEndings::Releasing(drag_data.current_pos)),
            };
        }
    }

    pub fn is_dragging(&self) -> bool {
        matches!(self.drag_state, DragAreaStates::Dragging(_))
    }

    pub fn start_drag(&mut self, drag_data: DragAreaActiveDragData) -> &mut Self {
        if let DragAreaStates::Initial | DragAreaStates::Released(_) = self.drag_state {
            self.drag_state = DragAreaStates::Dragging(drag_data);
        }
        self
    }

    pub fn update_drag(&mut self, pos: Point2D<f64, f64>) {
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

impl Default for GlobalDragState {
    fn default() -> Self {
        Self {
            drag_state: DragAreaStates::Initial,
            snap_info: None,
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
    Released(Rect<f64, f64>),
    Snapped(DraggableSnapStates),
}

#[derive(Clone, PartialEq, Debug)]
enum DraggableSnapStates {
    Preview(DraggableTransitionData),
    Final(SnapInfo),
    Transitioning(DraggableTransitionData),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapInfo {
    pub rect: Rect<f64, f64>,
    pub target_id: Option<String>,
}

impl SnapInfo {
    pub fn new(target_id: Option<String>, rect: Rect<f64, f64>) -> Self {
        Self { rect, target_id }
    }
}

#[derive(Clone, Debug)]
pub struct DraggableRenderData {
    pub style: String,
    pub rect: Option<Rect<f64, f64>>,
}

impl Default for DraggableRenderData {
    fn default() -> Self {
        Self {
            style: DRAGGABLE_BASE_STYLES.to_string(),
            rect: None,
        }
    }
}

impl DraggableRenderData {
    fn new(style: String, rect: Rect<f64, f64>) -> Self {
        Self {
            style,
            rect: Some(rect),
        }
    }
}

#[derive(Debug)]
pub struct LocalDragState {
    drag_state: DraggableStates,
    draggable_variant: DraggableVariants,
    rect: Option<Rect<f64, f64>>,
    id: String,
}

impl LocalDragState {
    pub fn new(variant: DraggableVariants, id: String) -> Self {
        Self {
            drag_state: DraggableStates::Resting(DraggableRestStates::Initial),
            draggable_variant: variant,
            rect: None,
            id,
        }
    }

    pub fn get_rect(&self) -> Option<Rect<f64, f64>> {
        self.rect
    }

    pub fn set_rect(&mut self, rect: Option<Rect<f64, f64>>) {
        self.rect = rect;
    }

    pub fn get_element_id(&self) -> String {
        self.id.clone()
    }

    pub fn resize_snapped(&mut self) {
        //tracing::info!("resizing {}", self.id);
        if let DraggableStates::Resting(DraggableRestStates::Snapped(snap_state)) = &self.drag_state
        {
            let new_snap_info = match snap_state {
                DraggableSnapStates::Final(snap_info) => self.resize_snapped_helper(snap_info),
                DraggableSnapStates::Preview(preview_data) => {
                    self.resize_snapped_helper(&preview_data.to)
                }
                DraggableSnapStates::Transitioning(transition) => {
                    self.resize_snapped_helper(&transition.to)
                }
            };
            self.drag_state = DraggableStates::Resting(DraggableRestStates::Snapped(
                DraggableSnapStates::Final(new_snap_info),
            ));
            tracing::info!("resized: {:?}", self.drag_state);
        }
    }

    fn resize_snapped_helper(&self, snap_info: &SnapInfo) -> SnapInfo {
        if let Some(target_id) = snap_info.target_id.clone() {
            return SnapInfo::new(Some(target_id), self.rect.unwrap());
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
                    let rect = self.rect.unwrap();
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

    fn poll_transition(&mut self, mut transition: DraggableTransitionData) -> DraggableSnapStates {
        let is_finished = false;
        match is_finished {
            true => match transition.mode {
                DraggableTransitionMode::Avoidance => DraggableSnapStates::Preview(transition),
                DraggableTransitionMode::Resting => DraggableSnapStates::Final(transition.to),
            },
            false => DraggableSnapStates::Transitioning(transition),
        }
    }

    pub fn update_state(&mut self, global_drag_state: DragAreaStates) {
        let old = self.drag_state.clone();
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
            (
                DraggableStates::Resting(DraggableRestStates::Snapped(
                    DraggableSnapStates::Transitioning(transition),
                )),
                _,
            ) => {
                self.poll_transition(transition);
            }
            (
                DraggableStates::Grabbed(draggable_grab_data),
                DragAreaStates::Dragging(drag_area_dragging_state),
            ) => {}
            (_, _) => (),
        };

        let new = self.drag_state.clone();
        tracing::info!("old: {:?} /n vs new: {:?}", old, new);
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
        pointer_position: Point2D<f64, f64>,
        draggable_grab_data: DraggableGrabData,
    ) -> DraggableStates {
        let x = pointer_position.x - draggable_grab_data.grab_point.x;
        let y = pointer_position.y - draggable_grab_data.grab_point.y;
        let resting_position: Point2D<f64, f64> = Point2D::new(x, y);
        let resting_size = self.rect.unwrap().size;
        DraggableStates::Resting(DraggableRestStates::Released(Rect::new(
            resting_position,
            resting_size,
        )))
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
            DraggableSnapStates::Transitioning(DraggableTransitionData::new(
                SnapInfo::new(None, from),
                snap_data,
                DraggableTransitionMode::Resting,
                self.id.clone(),
            )),
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
            ) => match other_snap_info.rect.intersects(&preview_data.from.rect) {
                true => DraggableSnapStates::Final(preview_data.to),
                false => DraggableSnapStates::Transitioning(preview_data.reverse()),
            },
            (DraggableSnapStates::Preview(preview_data), DragEndings::Releasing(_)) => {
                DraggableSnapStates::Transitioning(preview_data.reverse())
            }
            (DraggableSnapStates::Transitioning(transition), _) => {
                let res = self.poll_transition(transition);
                match res {
                    DraggableSnapStates::Preview(preview) => DraggableSnapStates::Final(preview.to),
                    DraggableSnapStates::Transitioning(transition) => {
                        DraggableSnapStates::Final(transition.to)
                    }
                    _ => res,
                }
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
            DraggableRestStates::Initial => self.rect.unwrap(),
            DraggableRestStates::Released(rect) => rect,
            DraggableRestStates::Snapped(snap_state) => match snap_state {
                DraggableSnapStates::Final(rect) => rect.rect,
                DraggableSnapStates::Preview(transition) => transition.to.rect,
                DraggableSnapStates::Transitioning(_) => self.rect.unwrap(),
            },
        };
        let intersects_this_rect =
            this_rect.contains(drag_area_dragging_state.current_pos.cast_unit());
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
                DraggableSnapStates::Transitioning(DraggableTransitionData::new(
                    info,
                    start_snap,
                    DraggableTransitionMode::Avoidance,
                    self.id.clone(),
                ))
            }
            (DraggableSnapStates::Preview(transition), true) => {
                DraggableSnapStates::Transitioning(transition.reverse())
            }
            (DraggableSnapStates::Preview(transition), false)
                if !transition
                    .from
                    .rect
                    .contains(drag_area_dragging_state.current_pos.cast_unit()) =>
            {
                DraggableSnapStates::Transitioning(transition.reverse())
            }
            (DraggableSnapStates::Transitioning(transition), _) => self.poll_transition(transition),
            (_, false) => snap_state,
        };

        self.drag_state = DraggableStates::Resting(DraggableRestStates::Snapped(snap_state));
    }

    pub fn get_render_data(&self, global_drag_state: DragAreaStates) -> DraggableRenderData {
        tracing::info!("getting render data");
        match (self.drag_state.clone(), global_drag_state.clone()) {
            (DraggableStates::Grabbed(grab_data), DragAreaStates::Dragging(drag_data)) => {
                let origin =
                    Self::origin_with_grab_offset(grab_data.grab_point, drag_data.current_pos);
                let size = Self::get_grabbed_size();
                DraggableRenderData::new(self.location_with_grab_offset(), Rect::new(origin, size))
            }
            (DraggableStates::Grabbed(grab_data), DragAreaStates::Released(ending)) => {
                let location = match ending {
                    DragEndings::Releasing(release) => release,
                    DragEndings::Snapping(snap) => snap.rect.origin,
                };
                let origin = Self::origin_with_grab_offset(grab_data.grab_point, location);
                let size = Self::get_grabbed_size();
                DraggableRenderData::new(self.location_with_grab_offset(), Rect::new(origin, size))
            }
            (DraggableStates::Resting(rest_state), DragAreaStates::Released(_)) => {
                self.get_render_data_for_resting_states(rest_state)
            }
            (
                DraggableStates::Resting(DraggableRestStates::Snapped(snap_state)),
                DragAreaStates::Dragging(_),
            ) => self.get_render_data_for_avoidance_states(snap_state),
            (DraggableStates::Resting(DraggableRestStates::Initial), DragAreaStates::Initial) => {
                self.initial_style()
            }
            (_, _) => {
                tracing::error!(
                    "Illegal state when getting render data for {:?}. local state: {:?}, global state: {:?}",
                    self.id,
                    self.drag_state,
                    global_drag_state
                );
                DraggableRenderData::new(String::new(), Rect::zero())
            }
        }
    }

    fn get_render_data_for_resting_states(
        &self,
        draggable_rest_state: DraggableRestStates,
    ) -> DraggableRenderData {
        match draggable_rest_state {
            DraggableRestStates::Initial => self.initial_style(),
            DraggableRestStates::Released(release_rect) => {
                let size = Self::get_grabbed_size();
                DraggableRenderData::new(self.location(), Rect::new(release_rect.origin, size))
            }
            DraggableRestStates::Snapped(snap_state) => {
                self.get_render_data_for_avoidance_states(snap_state)
            }
        }
    }

    fn get_render_data_for_avoidance_states(
        &self,
        draggable_snap_state: DraggableSnapStates,
    ) -> DraggableRenderData {
        match draggable_snap_state {
            DraggableSnapStates::Final(snap_info) => {
                DraggableRenderData::new(self.snapped_style(), snap_info.rect)
            }
            DraggableSnapStates::Preview(transition) => {
                DraggableRenderData::new(self.snapped_style(), transition.to.rect)
            }
            DraggableSnapStates::Transitioning(_transition) => {
                DraggableRenderData::new(self.snapped_transitioning_style(), self.rect.unwrap())
            }
        }
    }

    fn origin_with_grab_offset(
        drag_point: Point2D<f64, ElementSpace>,
        pointer_pos: Point2D<f64, f64>,
    ) -> Point2D<f64, f64> {
        let x = pointer_pos.x - drag_point.x;
        let y = pointer_pos.y - drag_point.y;
        Point2D::new(x, y)
    }

    fn get_grabbed_size() -> Size2D<f64, f64> {
        Size2D::new(200., 200.)
    }

    fn location_with_grab_offset(&self) -> String {
        format!("{}{}", DRAGGABLE_BASE_STYLES, DRAGGABLE_STYLES)
    }

    fn location(&self) -> String {
        format!("{}{}", DRAGGABLE_BASE_STYLES, DRAGGABLE_STYLES)
    }

    fn snapped_style(&self) -> String {
        format!("{}{}", DRAGGABLE_BASE_STYLES, SNAPPED_DRAGGABLE_STYLES,)
    }

    fn snapped_transitioning_style(&self) -> String {
        format!(
            "{} {}",
            self.snapped_style(),
            TRANSITIONING_DRAGGABLE_STYLES
        )
    }

    fn initial_style(&self) -> DraggableRenderData {
        DraggableRenderData::default()
    }
}
