use bevy::prelude::*;

pub(super) struct StateMachinePlugin;
impl Plugin for StateMachinePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StateMachine>();
    }
}

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
pub struct StateMachine {
    pub movement_state: MajorMoveState,
}

#[derive(Reflect, Clone, Copy)]
pub enum MajorMoveState {
    Grounded(MinorGroundState),
    Airborne(MinorAirborneState),
}

impl Default for MajorMoveState {
    fn default() -> Self {
        Self::Grounded(MinorGroundState::default())
    }
}

#[derive(Clone, Copy, Default, Reflect)]
pub enum MinorGroundState {
    #[default]
    Moving,
    Sliding,
    Crouched,
}

#[derive(Clone, Copy, Default, Reflect)]
pub enum MinorAirborneState {
    #[default]
    Falling,
    Jumping,
    CrouchJump,
    Glide,
}

#[derive(Clone, Copy, Default, Reflect)]
pub enum JumpPossibility {
    #[default]
    Jump,
    CrouchJump,
    No,
}

pub trait PlayerStateMachine {
    /// Can the player enter a jumpin state?
    fn can_jump(&self) -> JumpPossibility;

    /// Obtain the movement stats of a movement state
    fn movement_stats(&self) -> MovementStats;

    /// Gravity in format (gravity_up, gravity_down)
    fn gravity(&self) -> (f32, f32);
}

pub struct MovementStats {
    /// Maximum obtainable speed
    pub max_speed: f32,
    /// Constant acceleration on that state
    pub acceleration: f32,
}

impl PlayerStateMachine for StateMachine {
    fn can_jump(&self) -> JumpPossibility {
        match self.movement_state {
            MajorMoveState::Grounded(substate) => match substate {
                MinorGroundState::Moving | MinorGroundState::Sliding => {
                    return JumpPossibility::Jump;
                }
                MinorGroundState::Crouched => return JumpPossibility::CrouchJump,
            },
            MajorMoveState::Airborne(_substate) => {
                return JumpPossibility::No;
            }
        }
    }

    fn movement_stats(&self) -> MovementStats {
        match self.movement_state {
            MajorMoveState::Grounded(substate) => match substate {
                MinorGroundState::Moving => {
                    return MovementStats {
                        max_speed: 10.0,
                        acceleration: 30.0,
                    };
                }
                MinorGroundState::Sliding => {
                    return MovementStats {
                        max_speed: 0.0,
                        acceleration: 0.0,
                    };
                }
                MinorGroundState::Crouched => todo!(),
            },
            MajorMoveState::Airborne(substate) => match substate {
                MinorAirborneState::Falling => todo!(),
                MinorAirborneState::Jumping => todo!(),
                MinorAirborneState::CrouchJump => todo!(),
                MinorAirborneState::Glide => todo!(),
            },
        }
    }

    fn gravity(&self) -> (f32, f32) {
        return bevy::app::hotpatch::call(|| {
            match self.movement_state {
                MajorMoveState::Grounded(substate) => match substate {
                    MinorGroundState::Moving => return (0.0, 0.0),
                    MinorGroundState::Sliding => return (20.0, 20.0),
                    MinorGroundState::Crouched => return (20.0, 20.0),
                },
                MajorMoveState::Airborne(substate) => match substate {
                    MinorAirborneState::Falling => todo!(),
                    MinorAirborneState::Jumping => todo!(),
                    MinorAirborneState::CrouchJump => todo!(),
                    MinorAirborneState::Glide => todo!(),
                },
            }
            return (10.0, 15.0);
        });
    }
}
