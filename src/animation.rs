use xilem::WidgetView;
use xilem::core::Edit;
use xilem::core::one_of::Either;
use xilem::view::text_button;

#[derive(Debug)]
pub enum AnimStep {
    Idle(u64),
    Scaling,
    Slowing(u64),
    Pausing(u64),
    Shifting(u64),
}

impl Default for AnimStep {
    fn default() -> Self {
        Self::Shifting(Self::SHIFTING_FRAMES)
    }
}

impl AnimStep {
    pub const IDLE_TIME: f64 = 1.;
    pub const PAUSING_TIME: f64 = 3.;
    pub const SLOWING_TIME: f64 = 0.1;
    pub const SHIFTING_TIME: f64 = 2.;

    pub const IDLE_FRAMES: u64 = (Self::IDLE_TIME * Animation::FPS) as u64;
    pub const PAUSING_FRAMES: u64 = (Self::PAUSING_TIME * Animation::FPS) as u64;
    pub const SLOWING_FRAMES: u64 = (Self::SLOWING_TIME * Animation::FPS) as u64;
    pub const SHIFTING_FRAMES: u64 = (Self::SHIFTING_TIME * Animation::FPS) as u64;

    fn next(&self) -> AnimStep {
        match self {
            AnimStep::Idle(_) => AnimStep::Scaling,
            AnimStep::Scaling => AnimStep::Slowing(Self::SLOWING_FRAMES),
            AnimStep::Slowing(_) => AnimStep::Pausing(Self::PAUSING_FRAMES),
            AnimStep::Pausing(_) => AnimStep::Shifting(Self::SHIFTING_FRAMES),
            AnimStep::Shifting(_) => AnimStep::Idle(Self::IDLE_FRAMES),
        }
    }

    fn advance(&mut self, scaling_done: bool, slowing_done: bool) {
        match self {
            AnimStep::Idle(i) | AnimStep::Pausing(i) | AnimStep::Shifting(i) => {
                if *i > 0 {
                    *i -= 1;
                } else {
                    *self = self.next();
                }
            }
            AnimStep::Scaling => {
                if scaling_done {
                    *self = self.next();
                }
            }
            AnimStep::Slowing(i) => {
                if slowing_done || *i == 0 {
                    *self = self.next();
                } else {
                    *i -= 1;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Animation {
    pub active: bool,
    pub frame: u64,
    pub step: AnimStep,
}

impl Animation {
    pub const FRAME_DURATION: u64 = 16;
    pub const FPS: f64 = 1000. / Self::FRAME_DURATION as f64;

    pub fn tick(&mut self, scaling_done: bool, slowing_done: bool) {
        self.frame += 1;
        self.step.advance(scaling_done, slowing_done);
    }

    pub fn secs(&self) -> f64 {
        self.frame as f64 / Self::FPS
    }

    pub fn info(&self) -> String {
        if self.frame > 0 {
            format!(
                " | frame: {}, time: {:.1} s{}",
                self.frame,
                self.secs(),
                if self.active { "" } else { " [paused]" }
            )
        } else {
            "".to_string()
        }
    }

    pub fn playback_button(&mut self) -> impl WidgetView<Edit<Self>> + use<> {
        if self.active {
            Either::A(text_button("Pause", |state: &mut Self| {
                state.active = false;
            }))
        } else {
            Either::B(text_button("Play", |state: &mut Self| {
                state.active = true;
            }))
        }
    }
}
