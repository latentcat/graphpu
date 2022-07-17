#[derive(Debug, PartialEq)]
pub enum Enum {
    First,
    Second,
    Third,
}

#[derive(Debug, PartialEq)]
pub enum ComputeMethod {
    ForceAtlas2,
    Randomize,
}

pub struct InspectorModel {
    pub radio_arr: [Enum; 2],
    pub compute_method: ComputeMethod,
    pub is_computing: bool,
}

impl Default for InspectorModel {
    fn default() -> Self {
        Self {
            radio_arr: [Enum::First, Enum::Second],
            compute_method: ComputeMethod::ForceAtlas2,
            is_computing: false,
        }
    }
}

impl InspectorModel {
    pub fn reset(&mut self) {
        self.radio_arr = [Enum::First, Enum::Second];
    }
    pub fn switch_computing(&mut self) {
        self.is_computing = !self.is_computing;
    }
}
