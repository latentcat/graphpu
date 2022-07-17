#[derive(Debug, PartialEq)]
pub enum Enum {
    First,
    Second,
    Third,
}

pub struct InspectorModel {
    pub radio_arr: [Enum; 2],
}

impl Default for InspectorModel {
    fn default() -> Self {
        Self {
            radio_arr: [Enum::First, Enum::Second],
        }
    }
}

impl InspectorModel {
    pub fn reset(&mut self) {
        self.radio_arr = [Enum::First, Enum::Second];
    }
}
