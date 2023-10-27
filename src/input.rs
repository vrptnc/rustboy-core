pub enum ButtonType {
    ACTION,
    DIRECTION,
}

#[derive(Copy, Clone)]
pub enum Button {
    A,
    B,
    SELECT,
    START,
    RIGHT,
    LEFT,
    UP,
    DOWN,
}

impl Button {
    pub fn button_index(&self) -> usize {
        match self {
            Button::A => 0,
            Button::B => 1,
            Button::SELECT => 2,
            Button::START => 3,
            Button::RIGHT => 0,
            Button::LEFT => 1,
            Button::UP => 2,
            Button::DOWN => 3,
        }
    }

    pub fn button_type(&self) -> ButtonType {
        match self {
            Button::A => ButtonType::ACTION,
            Button::B => ButtonType::ACTION,
            Button::SELECT => ButtonType::ACTION,
            Button::START => ButtonType::ACTION,
            Button::RIGHT => ButtonType::DIRECTION,
            Button::LEFT => ButtonType::DIRECTION,
            Button::UP => ButtonType::DIRECTION,
            Button::DOWN => ButtonType::DIRECTION
        }
    }
}