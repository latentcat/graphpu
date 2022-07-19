use std::rc::Rc;

use crate::widgets::GraphicDelegation;

pub struct GraphicsModel {
  pub graphic_delegation: Rc<dyn GraphicDelegation>,
}

impl GraphicsModel {
  pub fn new(graphic_delegation: Rc<dyn GraphicDelegation>) -> Self {
    Self { graphic_delegation }
  }
}