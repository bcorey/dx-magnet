use animatable::controllers::AnimationBuilder;

use super::SnapInfo;

#[derive(Clone, PartialEq, Debug)]
pub struct DraggableTransitionData {
    pub from: SnapInfo,
    pub to: SnapInfo,
    pub anim: AnimationBuilder,
    pub mode: DraggableTransitionMode,
    id: String,
}

impl DraggableTransitionData {
    pub fn new(from: SnapInfo, to: SnapInfo, mode: DraggableTransitionMode, id: String) -> Self {
        let anim = AnimationBuilder::default()
            .animate_to(to.rect)
            .with_duration(web_time::Duration::from_millis(500))
            .with_easing(animatable::easing::Easing::BounceOut);
        tracing::info!("new anim: {:?}", anim);
        Self {
            from,
            to,
            anim,
            mode,
            id,
        }
    }

    pub fn reverse(&self) -> DraggableTransitionData {
        DraggableTransitionData {
            from: self.to.clone(),
            to: self.from.clone(),
            anim: self.anim.clone(), // TODO reverse anim
            mode: self.mode.reverse(),
            id: self.id.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DraggableTransitionMode {
    Avoidance,
    Resting,
}

impl DraggableTransitionMode {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Avoidance => Self::Resting,
            Self::Resting => Self::Avoidance,
        }
    }
}
