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

        app.add_systems(Update, move_camera);

        app.add_systems(
            FixedUpdate,
            ((
                player_check_floor,
                player_reset_y_vel,
                (player_movement, player_gravity),
            )
                .chain()
                .after(PhysicsSystems::Last),),
        );
    }
}

#[derive(Component, Reflect, Clone, Copy, Default)]
#[require(CharacterBody {grounded: true, up: Dir3::Y, max_dot_variance: 0.49, last_normal: Dir3::Y}, CharacterGroundSnap {distance: 0.5}, Collider::capsule(0.2,0.8), PlayerMarker, PlayerLookDirection, StateMachine,)]
#[reflect(Component)]
pub struct PlayerCharacterMarker;

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
pub struct PlayerLookDirection(pub Vec3);

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
pub struct PlayerMarker;

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
pub struct CameraPivot;

fn move_camera(
    query: Query<(&mut Transform, &ChildOf), With<CameraPivot>>,
    mut players: Query<(&mut PlayerLookDirection, &ActionState<PlayerInput>)>,
    time: Res<Time>,
) {
    for (mut transform, child_of) in query {
        let (mut direction, input) = players.get_mut(child_of.0).unwrap();

        let camera_movement = input.axis_pair(&PlayerInput::Camera) * time.delta_secs();

        let mut euler_angles = transform.rotation.to_euler(EulerRot::YXZ);
        let old_euler_angles = euler_angles;

        euler_angles.1 += camera_movement.y;
        euler_angles.1 = euler_angles
            .1
            .clamp(-80.0_f32.to_radians(), 80.0_f32.to_radians());

        let diff = euler_angles.1 - old_euler_angles.1;

        transform.rotate_local_x(diff);
        transform.rotate_y(camera_movement.x);

        direction.0 = transform.rotation * Vec3::Z;
    }
}

fn player_reset_y_vel(players: Query<(&mut LinearVelocity, &StateMachine)>, time: Res<Time>) {
    for (mut velocity, state) in players {
        if state.set_y_0() {
            velocity.y = velocity.y.lerp(0.0, time.delta_secs() * 10.0);
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
        bevy::app::hotpatch::call(|| {
            let movement_stats = state.movement_stats();

            let mut input_direction = input.axis_pair(&PlayerInput::Move);
            input_direction.y = -input_direction.y;

            let look_dir = Dir2::new(look_direction.0.xz()).unwrap_or(Dir2::Y);

            input_direction = input_direction
                .rotate(*look_dir)
                .rotate(Vec2::from_angle(-std::f32::consts::PI / 2.0));

            let mut target_velocity = input_direction * movement_stats.max_speed;

            let flat_velocity = velocity.xz();

            if velocity.length() > movement_stats.max_speed {
                if input_direction.length_squared() > 0.01 {
                    let new_target = flat_velocity.length() * input_direction;
                    
                    target_velocity = new_target;
                }
            }

            let moved_flat_vel = flat_velocity.move_towards(
                target_velocity,
                time.delta_secs() * movement_stats.acceleration,
            );

            velocity.x = moved_flat_vel.x;
            velocity.z = moved_flat_vel.y;
        });
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
