use bevy::prelude::*;

pub(super) struct StateMachinePlugin;
impl Plugin for StateMachinePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StateMachine>();
    }
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct StateMachine {
    pub movement_state: MajorMoveState,
    coyote_timer: f32,
    pub stuck_in_state_timer: f32,
    pub can_dive: bool,
}

#[derive(Reflect, Clone)]
pub enum MajorMoveState {
    Grounded(MinorGroundState),
    Airborne(MinorAirborneState),
}

impl Default for MajorMoveState {
    fn default() -> Self {
        Self::Grounded(MinorGroundState::default())
    }
}

#[derive(Clone, Default, Reflect)]
pub enum MinorGroundState {
    #[default]
    Moving,
    Sliding,
    Crouched,
}

#[derive(Clone, Default, Reflect)]
pub enum MinorAirborneState {
    #[default]
    Falling,
    Jumping(JumpType),
    Dive,
    Glide,
}

#[derive(Clone, Copy, Reflect)]
/// Internal f32 to count how much time left there is on the jump
pub enum JumpType {
    Normal(f32),
    Crouch(f32),
    Dive(f32),
}

pub trait PlayerStateMachine {
    /// Try to get into a jumping state and return the new state
    fn jump(&mut self) -> Result<MajorMoveState, MajorMoveState>;

    /// Transition to a new state unless stuck. Returns Ok(Old state) Err(New state)
    fn transition(&mut self, new_state: MajorMoveState) -> Result<MajorMoveState, MajorMoveState>;

    /// Update the state of the state machine
    fn tick(&mut self, time: Time) -> ();

    /// Check if machine is in a state where the y of the velocity should be 0.0
    fn set_y_0(&self) -> bool;

    /// Check if machine is in a grounded state
    fn is_grounded(&self) -> bool;

    /// Obtain the movement stats of a movement state
    fn movement_stats(&self) -> MovementStats;

    /// Gravity in format (gravity_up, gravity_down, terminal_velocity)
    fn gravity(&self) -> (f32, f32, f32);

    /// Get the jump strength of the jump type
    fn jump_strength(&self) -> f32;
}

pub struct MovementStats {
    /// Maximum obtainable speed
    pub max_speed: f32,
    /// Constant acceleration on that state
    pub acceleration: f32,
    /// When the player can only steer the movement use this value
    pub rotation_rate: f32,
}

const MAX_JUMP_LENGTH: f32 = 0.2;
const MAX_CROUCH_JUMP_LENGTH: f32 = 0.3;
const MAX_DIVE_JUMP_LENGTH: f32 = 0.1;

impl PlayerStateMachine for StateMachine {
    fn jump(&mut self) -> Result<MajorMoveState, MajorMoveState> {
        match &self.movement_state {
            MajorMoveState::Grounded(substate) => match substate {
                MinorGroundState::Moving => {
                    return self.transition(MajorMoveState::Airborne(MinorAirborneState::Jumping(
                        JumpType::Normal(MAX_JUMP_LENGTH),
                    )));
                }
                MinorGroundState::Crouched | MinorGroundState::Sliding => {
                    return self.transition(MajorMoveState::Airborne(MinorAirborneState::Jumping(
                        JumpType::Crouch(MAX_CROUCH_JUMP_LENGTH),
                    )));
                }
            },
            MajorMoveState::Airborne(substate) => match substate {
                MinorAirborneState::Dive => {
                    return self.transition(MajorMoveState::Airborne(MinorAirborneState::Jumping(
                        JumpType::Dive(MAX_DIVE_JUMP_LENGTH),
                    )));
                }
                MinorAirborneState::Jumping(jump_type) => {
                    return Ok(MajorMoveState::Airborne(MinorAirborneState::Jumping(
                        jump_type.clone(),
                    )));
                }
                MinorAirborneState::Falling | MinorAirborneState::Glide => {
                    if self.coyote_timer > f32::EPSILON {
                        if let Ok(_) = self.transition(MajorMoveState::Airborne(
                            MinorAirborneState::Jumping(JumpType::Normal(MAX_JUMP_LENGTH)),
                        )) {
                            self.coyote_timer = 0.0;
                            return Ok(self.movement_state.clone());
                        }
                    }
                }
            },
        }

        return Err(self.movement_state.clone());
    }

    fn tick(&mut self, time: Time) -> () {
        let delta = time.delta_secs();

        self.coyote_timer -= delta;
        self.coyote_timer = self.coyote_timer.max(0.0);

        self.stuck_in_state_timer -= delta;
        self.stuck_in_state_timer = self.stuck_in_state_timer.max(0.0);

        match self.movement_state {
            MajorMoveState::Grounded(_) => {
                self.coyote_timer = 0.25;
                self.can_dive = true;
            }
            MajorMoveState::Airborne(_) => {}
        }
    }

    fn transition(&mut self, new_state: MajorMoveState) -> Result<MajorMoveState, MajorMoveState> {
        if self.stuck_in_state_timer > 0.0 {
            return Err(new_state);
        }

        self.movement_state = new_state;

        Ok(self.movement_state.clone())
    }

    fn set_y_0(&self) -> bool {
        match &self.movement_state {
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
        match &self.movement_state {
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
                MinorAirborneState::Falling | MinorAirborneState::Jumping(_) => {
                    return MovementStats {
                        max_speed: 10.0,
                        acceleration: 10.0,
                        rotation_rate: 0.0,
                    };
                }
                MinorAirborneState::Glide => {
                    return MovementStats {
                        max_speed: 5.0,
                        acceleration: 10.0,
                        rotation_rate: 0.0,
                    };
                }
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
        match &self.movement_state {
            MajorMoveState::Grounded(substate) => match substate {
                MinorGroundState::Moving | MinorGroundState::Crouched => return (0.0, 0.0, 0.0),
                MinorGroundState::Sliding => return (60.0, 60.0, f32::INFINITY),
            },
            MajorMoveState::Airborne(substate) => match substate {
                MinorAirborneState::Jumping(_) => {
                    return (0.0, 0.0, 1.0);
                }
                MinorAirborneState::Falling => return (15.0, 25.0, 20.0),
                MinorAirborneState::Glide => return (1.0, 1.0, 5.0),
                MinorAirborneState::Dive => return (4.0, 160.0, 80.0),
            },
        }
    }

    fn jump_strength(&self) -> f32 {
        match &self.movement_state {
            MajorMoveState::Airborne(substate) => match substate {
                MinorAirborneState::Jumping(jump_type) => match jump_type {
                    JumpType::Normal(_) => return 5.0,
                    JumpType::Crouch(_) => return 7.0,
                    JumpType::Dive(_) => return 7.0,
                },
                _ => {}
            },
            _ => {}
        }
        return 0.0;
    }
}
