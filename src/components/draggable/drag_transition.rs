use web_time::Duration;

use super::{RectData, SnapInfo};

#[derive(Clone, PartialEq, Debug)]
pub struct DraggableTransitionData {
    pub from: SnapInfo,
    pub to: SnapInfo,
    pub current: RectData,
    pub mode: DraggableTransitionMode,
    id: String,
    start_time: web_time::SystemTime,
    cycles: u64,
}

impl DraggableTransitionData {
    pub fn new(from: SnapInfo, to: SnapInfo, mode: DraggableTransitionMode, id: String) -> Self {
        let current = from.rect;
        Self {
            from,
            to,
            mode,
            current,
            id,
            start_time: web_time::SystemTime::now(),
            cycles: 0,
        }
    }

    pub fn reverse(&self) -> DraggableTransitionData {
        DraggableTransitionData {
            from: self.to.clone(),
            to: self.from.clone(),
            current: self.current,
            mode: self.mode.reverse(),
            id: self.id.clone(),
            start_time: web_time::SystemTime::now(),
            cycles: self.cycles,
        }
    }

    fn tick(&mut self) {
        self.cycles += 1;
    }

    pub fn poll(&mut self) -> bool {
        let max_dur = Duration::from_millis(1000);
        let elapsed = self.start_time.elapsed().unwrap();
        if elapsed >= max_dur {
            return true;
        }
        self.tick();
        let percent_elapsed = elapsed.as_millis() as f64 / max_dur.as_millis() as f64;
        self.current = self
            .to
            .rect
            .percent_transition(self.from.rect, percent_elapsed);
        false
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
