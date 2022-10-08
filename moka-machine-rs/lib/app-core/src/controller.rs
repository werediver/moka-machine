use core::ops::RangeInclusive;

use crate::deadband;

type Deadband = deadband::Deadband<RangeInclusive<f32>, f32>;

pub struct Controller {
    tolerance: f32,
    target_temp: Option<Deadband>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Action {
    EnableHeater,
    DisableHeater,
}

impl Controller {
    pub fn new(tolerance: f32) -> Self {
        Self {
            tolerance,
            target_temp: None,
        }
    }

    pub fn set_target_temp(&mut self, value: Option<f32>) {
        self.target_temp =
            value.map(|value| Deadband::new((value - self.tolerance)..=(value + self.tolerance)))
    }

    pub fn update(&self, current_temp: f32) -> Option<Action> {
        use deadband::DeadbandComparisonResult::*;

        if let Some(target_temp) = self.target_temp.as_ref() {
            return match target_temp.compare(&current_temp) {
                Low => Some(Action::EnableHeater),
                High => Some(Action::DisableHeater),
                Hold => None,
            };
        }

        None
    }
}
