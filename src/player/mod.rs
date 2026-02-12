use crate::character_body::{CharacterBody, CharacterGroundSnap};
use crate::input::PlayerInput;

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use state_machine::*;

pub mod camera;
pub mod state_machine;

pub const PLAYER_HEIGHT: f32 = 1.0;
pub const PLAYER_THICKNESS: f32 = 0.2;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerCharacterMarker>()
            .register_type::<PlayerMarker>();

        app.add_plugins((camera::CameraPlugin, state_machine::StateMachinePlugin));

        app.add_systems(
            FixedUpdate,
            ((
                player_check_floor,
                player_reset_y_vel,
                player_slide_and_crouch,
                (player_gravity, player_movement),
                player_jump,
                (player_rotation, player_tick_machine),
            )
                .chain()
                .after(PhysicsSystems::Last),),
        );
    }
}

#[derive(Component, Reflect, Clone, Copy, Default)]
#[require(
    CharacterBody {
            grounded: true,
            up: Dir3::Y,
            max_dot_variance: 0.49,
            last_normal: Dir3::Y,
            force_slide: false,
        },
        CharacterGroundSnap { distance: 0.5 },
        Collider::capsule(PLAYER_THICKNESS, PLAYER_HEIGHT-2.0*PLAYER_THICKNESS),
        PlayerMarker,
        PlayerLookDirection,
        StateMachine,
        SleepingDisabled,
)]
#[reflect(Component)]
pub struct PlayerCharacterMarker;

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
pub struct PlayerLookDirection(pub Vec3);

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
pub struct PlayerMarker;

fn player_reset_y_vel(players: Query<(&mut LinearVelocity, &StateMachine)>, time: Res<Time>) {
    for (mut velocity, state) in players {
        if state.set_y_0() {
            velocity.y = velocity.y.lerp(0.0, time.delta_secs() * 10.0);
        }
    }
}

const MIN_SLIDE_VEL: f32 = 7.5;
const MAX_SLIDE_VEL: f32 = 7.5;

fn player_slide_and_crouch(
    players: Query<(
        &mut StateMachine,
        &LinearVelocity,
        &ActionState<PlayerInput>,
        &CharacterBody,
    )>,
) {
    for (mut state, velocity, input, body) in players {
        // Return early if slide is forced
        if body.force_slide && state.is_grounded() && velocity.length_squared() > 0.001 {
            let _ = state.transition(MajorMoveState::Grounded(MinorGroundState::Sliding));
            continue;
        }

        match &state.movement_state {
            MajorMoveState::Grounded(substate) => match substate {
                // Slide if you can
                MinorGroundState::Moving => {
                    if input.pressed(&PlayerInput::Crouch) {
                        if velocity.length() > MIN_SLIDE_VEL {
                            let _ = state
                                .transition(MajorMoveState::Grounded(MinorGroundState::Sliding));
                        } else {
                            let _ = state
                                .transition(MajorMoveState::Grounded(MinorGroundState::Crouched));
                        }
                    }
                }
                MinorGroundState::Crouched => {
                    if !input.pressed(&PlayerInput::Crouch) {
                        let _ =
                            state.transition(MajorMoveState::Grounded(MinorGroundState::Moving));
                    }
                    if velocity.length() > MIN_SLIDE_VEL {
                        let _ =
                            state.transition(MajorMoveState::Grounded(MinorGroundState::Sliding));
                    }
                }
                // Check if it can still slide
                MinorGroundState::Sliding => {
                    if velocity.length() < MAX_SLIDE_VEL || !input.pressed(&PlayerInput::Crouch) {
                        let _ =
                            state.transition(MajorMoveState::Grounded(MinorGroundState::Moving));
                    }
                }
            },
            MajorMoveState::Airborne(_) => continue,
        }
    }
}

fn player_movement(
    players: Query<(
        &mut LinearVelocity,
        &ActionState<PlayerInput>,
        &PlayerLookDirection,
        &StateMachine,
    )>,
    time: Res<Time>,
) {
    for (mut velocity, input, look_direction, state) in players {
        let movement_stats = state.movement_stats();

        let mut input_direction = input.axis_pair(&PlayerInput::Move);
        input_direction.y = -input_direction.y;

        let look_dir = Dir2::new(look_direction.0.xz()).unwrap_or(Dir2::Y);

        input_direction = input_direction
            .rotate(*look_dir)
            .rotate(Vec2::from_angle(std::f32::consts::PI / 2.0));

        let flat_velocity = velocity.xz();

        if flat_velocity.length() > movement_stats.max_speed * 1.01
            && input_direction.length_squared() > 0.01
        {
            if input_direction.length_squared() > 0.01 {
                let simmilarity = input_direction.dot(flat_velocity).max(0.0);

                let target_velocity = simmilarity * input_direction;

                let moved_flat_vel = flat_velocity.move_towards(
                    target_velocity,
                    time.delta_secs() * movement_stats.rotation_rate,
                );

                velocity.x = moved_flat_vel.x;
                velocity.z = moved_flat_vel.y;
            }
        } else {
            let target_velocity = input_direction * movement_stats.max_speed;

            let moved_flat_vel = flat_velocity.move_towards(
                target_velocity,
                time.delta_secs() * movement_stats.acceleration,
            );

            velocity.x = moved_flat_vel.x;
            velocity.z = moved_flat_vel.y;
        }
    }
}

fn player_check_floor(players: Query<(&mut StateMachine, &CharacterBody)>) {
    for (mut machine, body) in players {
        if machine.is_grounded() && !body.grounded {
            let _ = machine.transition(MajorMoveState::Airborne(MinorAirborneState::Falling));
        }

        if !machine.is_grounded() && body.grounded {
            let _ = machine.transition(MajorMoveState::Grounded(MinorGroundState::Moving));
        }
    }
}

fn player_gravity(players: Query<(&mut LinearVelocity, &StateMachine)>, time: Res<Time>) {
    for (mut velocity, state) in players {
        let (up_gravity, down_gravity, terminal_velocity) = state.gravity();

        if velocity.y > 0.0 {
            velocity.y -= time.delta_secs() * up_gravity;
        } else {
            velocity.y -= time.delta_secs() * down_gravity;
        }

        if velocity.y < -terminal_velocity {
            velocity.y = -terminal_velocity;
        }
    }
}

fn player_jump(
    players: Query<(
        &mut LinearVelocity,
        &mut StateMachine,
        &mut CharacterBody,
        &ActionState<PlayerInput>,
    )>,
    time: Res<Time>,
) {
    for (mut velocity, mut state, mut body, input) in players {
        let mut stop_jump = false;

        if input.pressed(&PlayerInput::Jump) {
            let final_state = state.jump();

            if final_state.is_ok() {
                body.grounded = false;
                velocity.y = state.jump_strength();
            }
        } else {
            stop_jump = true;
        }

        if velocity.y < 0.0 {
            stop_jump = true;
        }

        match &mut state.movement_state {
            MajorMoveState::Grounded(_) => {}
            MajorMoveState::Airborne(substate) => match substate {
                MinorAirborneState::Jumping(jump_type) => match jump_type {
                    JumpType::Normal(time_left)
                    | JumpType::Dive(time_left)
                    | JumpType::Crouch(time_left) => {
                        *time_left -= time.delta_secs();

                        if *time_left <= 0.0 {
                            stop_jump = true;
                        }
                    }
                },

                _ => {}
            },
        }

        if stop_jump {
            match &state.movement_state {
                MajorMoveState::Grounded(_) => {}
                MajorMoveState::Airborne(substate) => match substate {
                    MinorAirborneState::Jumping(_) => {
                        let _ =
                            state.transition(MajorMoveState::Airborne(MinorAirborneState::Falling));
                    }
                    _ => {}
                },
            }
        }
    }
}

fn player_rotation(players: Query<(&mut Transform, &LinearVelocity)>) {
    for (mut transform, velocity) in players {
        let flat_vel = velocity.xz();

        if flat_vel.length_squared() <= 1.0 {
            continue;
        }

        let angle = Vec2::Y.angle_to(flat_vel);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, -angle, 0.0, 0.0);
    }
}

fn player_tick_machine(players: Query<&mut StateMachine>, time: Res<Time>) {
    for mut state in players {
        state.tick(*time);
    }
}
