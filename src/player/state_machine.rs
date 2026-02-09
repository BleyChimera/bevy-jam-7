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
    pub coyote_timer: f32,
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
    Dive,
    Glide,
}

#[derive(Clone, Copy, Default, Reflect)]
pub enum JumpPossibility {
    #[default]
    Jump,
    CrouchJump,
    DiveJump,
    No,
}

pub trait PlayerStateMachine {
    /// Can the player enter a jumpin state?
    fn can_jump(&self) -> JumpPossibility;

    /// Check if machine is in a state where the y of the velocity should be 0.0
    fn set_y_0(&self) -> bool;

    /// Check if machine is in a grounded state
    fn is_grounded(&self) -> bool;

    /// Obtain the movement stats of a movement state
    fn movement_stats(&self) -> MovementStats;

    /// Gravity in format (gravity_up, gravity_down, terminal_velocity)
    fn gravity(&self) -> (f32, f32, f32);
}

pub struct MovementStats {
    /// Maximum obtainable speed
    pub max_speed: f32,
    /// Constant acceleration on that state
    pub acceleration: f32,
    /// When the player can only steer the movement use this value
    pub rotation_rate: f32,
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
            MajorMoveState::Airborne(substate) => match substate {
                MinorAirborneState::Dive => {
                    return JumpPossibility::DiveJump;
                }
                MinorAirborneState::Jumping
                | MinorAirborneState::Falling
                | MinorAirborneState::CrouchJump
                | MinorAirborneState::Glide => return JumpPossibility::No,
            },
        }
    }

    fn set_y_0(&self) -> bool {
        match self.movement_state {
            MajorMoveState::Grounded(substate) => match substate {
                MinorGroundState::Moving => true,
                MinorGroundState::Sliding => false,
                MinorGroundState::Crouched => false,
            },
            MajorMoveState::Airborne(_) => false,
        }
    }

    fn is_grounded(&self) -> bool {
        match self.movement_state {
            MajorMoveState::Grounded(_) => true,
            MajorMoveState::Airborne(_) => false,
        }
    }

    fn movement_stats(&self) -> MovementStats {
        match self.movement_state {
            MajorMoveState::Grounded(substate) => match substate {
                MinorGroundState::Moving => {
                    return MovementStats {
                        max_speed: 10.0,
                        acceleration: 30.0,
                        rotation_rate: 10.0,
                    };
                }
                MinorGroundState::Sliding => {
                    return MovementStats {
                        max_speed: 0.0,
                        acceleration: 0.0,
                        rotation_rate: 20.0,
                    };
                }
                MinorGroundState::Crouched => {
                    return MovementStats {
                        max_speed: 0.0,
                        acceleration: 10.0,
                        rotation_rate: 10.0,
                    };
                }
            },
            MajorMoveState::Airborne(substate) => match substate {
                MinorAirborneState::Falling => {
                    return MovementStats {
                        max_speed: 10.0,
                        acceleration: 10.0,
                        rotation_rate: 0.0,
                    };
                }
                MinorAirborneState::Jumping => todo!(),
                MinorAirborneState::CrouchJump => todo!(),
                MinorAirborneState::Glide => todo!(),
                MinorAirborneState::Dive => {
                    return MovementStats {
                        max_speed: 10.0,
                        acceleration: 5.0,
                        rotation_rate: 0.0,
                    };
                }
            },
        }
    }

    fn gravity(&self) -> (f32, f32, f32) {
        match self.movement_state {
            MajorMoveState::Grounded(substate) => match substate {
                MinorGroundState::Moving => return (0.0, 0.0, 0.0),
                MinorGroundState::Sliding => return (60.0, 60.0, f32::INFINITY),
                MinorGroundState::Crouched => return (20.0, 20.0, f32::INFINITY),
            },
            MajorMoveState::Airborne(substate) => match substate {
                MinorAirborneState::Falling
                | MinorAirborneState::Jumping
                | MinorAirborneState::CrouchJump => return (10.0, 15.0, 15.0),
                MinorAirborneState::Glide => return (1.0, 1.0, 5.0),
                MinorAirborneState::Dive => return (40.0, 160.0, 40.0),
            },
        }
    }
}
