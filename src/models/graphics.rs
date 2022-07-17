use crate::widgets::GraphicDelegation;

pub struct GraphicsModel {
  pub graphic_delegation: Box<dyn GraphicDelegation>,
}

impl GraphicsModel {
  pub fn new(graphic_delegation: Box<dyn GraphicDelegation>) -> Self {
    Self { graphic_delegation }
  }
}