#[derive(Debug, PartialEq)]
pub enum ComputeMethod {
    ForceAtlas2,
    Randomize,
}

pub struct ComputeModel {
    pub compute_method: ComputeMethod,
    pub is_computing: bool,
    pub is_dispatching: bool,
}

impl Default for ComputeModel {
    fn default() -> Self {
        Self {
            compute_method: ComputeMethod::ForceAtlas2,
            is_computing: false,
            is_dispatching: false,
        }
    }
}

impl ComputeModel {
    pub fn switch_computing(&mut self) {
        self.is_computing = !self.is_computing;
    }

    pub fn set_computing(&mut self, state: bool) {
        self.is_computing = state;
    }

    pub fn set_dispatching(&mut self, state: bool) {
        self.is_dispatching = state;
    }
}
