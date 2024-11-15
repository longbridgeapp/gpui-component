use gpui::*;

pub enum ScrollWheelDirection {
	Up,
	Down,
	BigUp,
	BigDown,
}

impl ScrollWheelDirection {
	pub fn from_event(event: &ScrollWheelEvent) -> Option<Self> {
		if Self::is_scroll_up(event) {
			if event.modifiers.control {
				Some(ScrollWheelDirection::BigUp)
			} else {
				Some(ScrollWheelDirection::Up)
			}
		} else if Self::is_scroll_down(event) {
			if event.modifiers.control {
				Some(ScrollWheelDirection::BigDown)
			} else {
				Some(ScrollWheelDirection::Down)
			}
		} else {
			None
		}
	}

	fn is_scroll_up(event: &ScrollWheelEvent) -> bool {
		match event.delta {
			ScrollDelta::Pixels(point) => point.y > Pixels(0.0),
			ScrollDelta::Lines(point) => point.y > 0.0,
		}
	}

	fn is_scroll_down(event: &ScrollWheelEvent) -> bool {
		match event.delta {
			ScrollDelta::Pixels(point) => point.y < Pixels(0.0),
			ScrollDelta::Lines(point) => point.y < 0.0,
		}
	}
}

pub enum ClickType {
	Normal,
	Big,
}

impl ClickType {
	pub fn from_event(event: &ClickEvent) -> Option<Self> {
		if Self::is_normal_click(event) {
			Some(ClickType::Normal)
		} else if Self::is_big_click(event) {
			Some(ClickType::Big)
		} else {
			None
		}
	}

	fn is_normal_click(event: &ClickEvent) -> bool {
		let modifier = event.down.modifiers;
		!modifier.control
	}

	fn is_big_click(event: &ClickEvent) -> bool {
		let modifier = event.down.modifiers;
		modifier.control
	}
}
