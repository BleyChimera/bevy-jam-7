use crate::character_body::{CharacterBody, CharacterGroundSnap};
use crate::input::PlayerInput;

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use state_machine::*;

pub mod state_machine;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerCharacterMarker>()
            .register_type::<PlayerMarker>()
            .register_type::<CameraPivot>();

        app.add_plugins(state_machine::StateMachinePlugin);

        app.add_systems(
            Update,
            (
                (rotate_camera_manual, rotate_camera_auto, move_camera),
                (update_camera_direction, unstuck_camera),
            )
                .chain(),
        );

        app.add_systems(
            FixedUpdate,
            ((
                player_check_floor,
                player_reset_y_vel,
                player_slide,
                (player_gravity, player_movement),
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
        Collider::capsule(0.2, 0.8),
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

#[derive(Component, Reflect, Clone, Copy)]
#[reflect(Component)]
pub struct CameraPivot(pub Entity);

fn rotate_camera_manual(
    query: Query<(&mut Transform, &CameraPivot), Without<PlayerCharacterMarker>>,
    mut players: Query<&ActionState<PlayerInput>, With<PlayerCharacterMarker>>,
    time: Res<Time>,
) {
    for (mut transform, pivot) in query {
        let Ok(input) = players.get_mut(pivot.0) else {
            continue;
        };

        let camera_movement = input.axis_pair(&PlayerInput::Camera) * time.delta_secs();

        let mut euler_angles = transform.rotation.to_euler(EulerRot::YXZ);
        let old_euler_angles = euler_angles;

        euler_angles.1 -= camera_movement.y;
        euler_angles.1 = euler_angles
            .1
            .clamp(-80.0_f32.to_radians(), 80.0_f32.to_radians());

        let diff = euler_angles.1 - old_euler_angles.1;

        transform.rotate_local_x(diff);
        transform.rotate_y(-camera_movement.x);
    }
}

fn rotate_camera_auto(
    query: Query<(&mut Transform, &CameraPivot), Without<PlayerCharacterMarker>>,
    mut players: Query<(&ActionState<PlayerInput>, &LinearVelocity), With<PlayerCharacterMarker>>,
    time: Res<Time>,
) {
    for (mut transform, pivot) in query {
        let Ok((input, velocity)) = players.get_mut(pivot.0) else {
            continue;
        };

        let camera_input = input.axis_pair(&PlayerInput::Camera);

        // If camera is being pushed by player don't try to sway it
        if camera_input.length_squared() > 0.25 {
            continue;
        }

        let flat_velocity = velocity.xz();

        // Get direction to point into
        let Some(flat_dir) = flat_velocity.try_normalize() else {
            continue;
        };

        let point_direction = (transform.rotation * Vec3::NEG_Z).xz().normalize_or_zero();

        let mut angle_diff = point_direction.angle_to(flat_dir);

        if angle_diff.abs() > std::f32::consts::PI - 0.0001 {
            angle_diff = angle_diff * 0.0;
        } else if angle_diff.abs() > std::f32::consts::PI / 2.0 {
            angle_diff = angle_diff * 0.1;
        }

        transform.rotate_y(-angle_diff * time.delta_secs());
    }
}

fn update_camera_direction(
    query: Query<(&Transform, &CameraPivot)>,
    mut players: Query<&mut PlayerLookDirection, With<PlayerCharacterMarker>>,
) {
    for (transform, pivot) in query {
        let Ok(mut direction) = players.get_mut(pivot.0) else {
            continue;
        };

        direction.0 = transform.rotation * Vec3::NEG_Z;
    }
}

fn move_camera(
    query: Query<(&mut Transform, &CameraPivot), Without<PlayerCharacterMarker>>,
    mut players: Query<(&Transform, &LinearVelocity), With<PlayerCharacterMarker>>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (mut transform, pivot) in query {
        let Ok((player_transform, velocity)) = players.get_mut(pivot.0) else {
            continue;
        };
        let top_of_player = player_transform.translation;
        let top_of_player = top_of_player + Vec3::Y * 0.5;

        let cast = spatial_query.cast_ray(
            top_of_player,
            Dir3::new(velocity.0).unwrap_or(Dir3::Y),
            (velocity.0 / 4.0).length(),
            false,
            &SpatialQueryFilter::from_excluded_entities([pivot.0]),
        );

        let target_point = if let Some(cast) = cast {
            top_of_player + velocity.0.normalize_or_zero() * cast.distance
        } else {
            top_of_player + velocity.0 / 4.0
        };

        transform.translation = transform.translation.move_towards(
            target_point,
            time.delta_secs() * (transform.translation - target_point).length() * 15.0,
        );
    }
}

fn unstuck_camera(
    pivots: Query<(&Transform, &CameraPivot)>,
    cameras: Query<(&mut Transform, &ChildOf), Without<CameraPivot>>,
    //time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (mut transform, parent) in cameras {
        let Ok((pivot_transform, pivot)) = pivots.get(parent.0) else {
            continue;
        };

        let cast = spatial_query.cast_shape(
            &Collider::sphere(0.1),
            pivot_transform.translation,
            Quat::IDENTITY,
            Dir3::new(pivot_transform.rotation * Vec3::Z).unwrap(),
            &ShapeCastConfig {
                max_distance: 10.0,
                target_distance: 0.0,
                compute_contact_on_penetration: true,
                ignore_origin_penetration: true,
            },
            &SpatialQueryFilter::from_excluded_entities([pivot.0]),
        );

        if let Some(cast) = cast {
            transform.translation.z = cast.distance;
        } else {
            transform.translation.z = 10.0;
        }
    }
}

fn player_reset_y_vel(players: Query<(&mut LinearVelocity, &StateMachine)>, time: Res<Time>) {
    for (mut velocity, state) in players {
        if state.set_y_0() {
            velocity.y = velocity.y.lerp(0.0, time.delta_secs() * 10.0);
        }
    }
}

fn player_slide(
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
            state.movement_state = MajorMoveState::Grounded(MinorGroundState::Sliding);
            continue;
        }

        match state.movement_state {
            MajorMoveState::Grounded(substate) => match substate {
                // Slide if you can
                MinorGroundState::Moving | MinorGroundState::Crouched => {
                    if velocity.length() > 7.5 && input.pressed(&PlayerInput::Crouch) {
                        state.movement_state = MajorMoveState::Grounded(MinorGroundState::Sliding);
                    }
                    if body.force_slide {
                        state.movement_state = MajorMoveState::Grounded(MinorGroundState::Sliding);
                    }
                }
                // Check if it can still slide
                MinorGroundState::Sliding => {
                    if velocity.length() < 5.0 || !input.pressed(&PlayerInput::Crouch) {
                        state.movement_state = MajorMoveState::Grounded(MinorGroundState::Moving);
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
            machine.movement_state = MajorMoveState::Airborne(MinorAirborneState::Falling);
        }

        if !machine.is_grounded() && body.grounded {
            machine.movement_state = MajorMoveState::Grounded(MinorGroundState::Moving);
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
