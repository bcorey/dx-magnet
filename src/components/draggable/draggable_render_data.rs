use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::Rect;
use dx_flipbook::controllers::AnimationBuilder;

const DRAGGABLE_BASE_STYLES: &str = "
    display: flex;
    flex-flow: column;
    flex-direction: column;
    height: 100%;
    align-content: flex-start;
";

const DRAGGABLE_DRAG_STYLES: &str = "
    position: absolute;
    background-color: var(--accent_0);
    z-index: 10000;
    width: 180px;
    height: 200px;
    box-shadow: .4rem .3rem var(--hint);
";

const SNAPPED_DRAGGABLE_STYLES: &str = "
    position: absolute;
    box-shadow: 0 0 solid var(--hint);
    z-index: 100;
";

const TRANSITIONING_DRAGGABLE_STYLES: &str = "
    z-index: 5000;
";

#[derive(Clone, Debug)]
pub struct DraggableRenderData {
    pub style: String,
    pub position_data: DraggablePositionData,
}

#[derive(Clone, Debug)]
pub enum DraggablePositionData {
    Default,
    Rect(Rect<f64, f64>),
    Anim(AnimationBuilder),
}

impl Default for DraggableRenderData {
    fn default() -> Self {
        Self {
            style: DRAGGABLE_BASE_STYLES.to_string(),
            position_data: DraggablePositionData::Default,
        }
    }
}

impl DraggableRenderData {
    pub(crate) fn transitioning(anim: AnimationBuilder) -> Self {
        Self {
            style: format!(
                "{}{}{}",
                DRAGGABLE_BASE_STYLES, SNAPPED_DRAGGABLE_STYLES, TRANSITIONING_DRAGGABLE_STYLES
            ),
            position_data: DraggablePositionData::Anim(anim),
        }
    }

    pub(crate) fn snapped(rect: Rect<f64, f64>) -> Self {
        Self {
            style: format!("{}{}", DRAGGABLE_BASE_STYLES, SNAPPED_DRAGGABLE_STYLES),
            position_data: DraggablePositionData::Rect(rect),
        }
    }

    pub(crate) fn free_or_dragging(rect: Rect<f64, f64>) -> Self {
        Self {
            style: format!("{}{}", DRAGGABLE_BASE_STYLES, DRAGGABLE_DRAG_STYLES),
            position_data: DraggablePositionData::Rect(rect),
        }
    }
}
