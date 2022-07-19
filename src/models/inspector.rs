#[derive(Debug, PartialEq)]
pub enum ComputeMethod {
    ForceAtlas2,
    Randomize,
}

pub struct InspectorModel {
    pub compute_method: ComputeMethod,
    pub is_computing: bool,
}

impl Default for InspectorModel {
    fn default() -> Self {
        Self {
            compute_method: ComputeMethod::ForceAtlas2,
            is_computing: false,
        }
    }
}

impl InspectorModel {
    pub fn switch_computing(&mut self) {
        self.is_computing = !self.is_computing;
    }
}
