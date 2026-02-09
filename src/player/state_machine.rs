use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
pub struct StateMachine {}

#[derive(Clone, Copy)]
pub enum MajorMoveState {
    Grounded(MinorGroundState),
    Airborne(MinorAirborneState),
}

impl Default for MajorMoveState {
    fn default() -> Self {
        Self::Grounded(MinorGroundState::default())
    }
}

#[derive(Clone, Copy, Default)]
pub enum MinorGroundState {
    #[default]
    Moving,
    Sliding,
    Crouched,
}

#[derive(Clone, Copy, Default)]
pub enum MinorAirborneState {
    #[default]
    Falling,
    Jumping,
    Glide,
}
