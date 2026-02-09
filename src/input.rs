use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerInput>::default());
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum PlayerInput {
    Move,
    Camera,
    Jump,
    Crouch,
}

impl Actionlike for PlayerInput {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlayerInput::Move => InputControlKind::DualAxis,
            PlayerInput::Camera => InputControlKind::DualAxis,
            PlayerInput::Jump => InputControlKind::Button,
            PlayerInput::Crouch => InputControlKind::Button,
        }
    }
}

impl PlayerInput {
    pub fn default_input_map() -> InputMap<PlayerInput> {
        InputMap::default()
            // Keyboard
            .with_dual_axis(
                PlayerInput::Move,
                VirtualDPad::wasd().with_circle_bounds(1.0),
            )
            .with_dual_axis(
                PlayerInput::Camera,
                MouseMove::default().with_processor(DualAxisProcessor::Sensitivity(
                    DualAxisSensitivity::all(0.1),
                )),
            )
            .with(PlayerInput::Jump, KeyCode::Space)
            .with(PlayerInput::Crouch, KeyCode::ControlLeft)
            // Controller
            .with_dual_axis(
                PlayerInput::Move,
                GamepadStick::LEFT
                    .with_circle_bounds(1.0)
                    .with_deadzone(-0.01, 0.01),
            )
            .with_dual_axis(
                PlayerInput::Camera,
                GamepadStick::RIGHT
                    .with_circle_bounds(1.0)
                    .with_deadzone(-0.01, 0.01),
            )
            .with(PlayerInput::Jump, GamepadButton::South)
            .with(PlayerInput::Crouch, GamepadButton::West)
    }
}
