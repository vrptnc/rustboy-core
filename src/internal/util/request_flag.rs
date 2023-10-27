use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RequestFlag(pub bool);

impl RequestFlag {
  pub fn new() -> Self {
    RequestFlag(false)
  }

  pub fn set(&mut self) {
    self.0 = true;
  }

  pub fn get_and_clear(&mut self) -> bool {
    let result = self.0;
    self.0 = false;
    result
  }
}