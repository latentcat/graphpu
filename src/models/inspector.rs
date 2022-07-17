#[derive(Debug, PartialEq)]
pub enum Enum {
    First,
    Second,
    Third,
}

pub enum ComputeMethod {
    ForceAtlas2,
    Randomize,
}

pub struct InspectorModel {
    pub radio_arr: [Enum; 2],
    pub compute_method: ComputeMethod,
}

impl Default for InspectorModel {
    fn default() -> Self {
        Self {
            radio_arr: [Enum::First, Enum::Second],
            compute_method: ComputeMethod::ForceAtlas2,
        }
    }
}

impl InspectorModel {
    pub fn reset(&mut self) {
        self.radio_arr = [Enum::First, Enum::Second];
    }
}
