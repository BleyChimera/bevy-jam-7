use crate::character_body::CharacterBody;
use crate::input::PlayerInput;

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerCharacterMarker>()
            .register_type::<PlayerMarker>()
            .register_type::<CameraPivot>();

        app.add_systems(Update, move_camera);

        app.add_systems(
            FixedUpdate,
            (player_movement, player_gravity).after(PhysicsSystems::Last),
        );
    }
}

#[derive(Component, Reflect, Clone, Copy, Default)]
#[require(CharacterBody {grounded: true, up: Dir3::Y, max_dot_variance: 0.49}, Collider::capsule(0.2,0.8), PlayerMarker, PlayerLookDirection)]
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
        
        bevy::app::hotpatch::call(|| {
            direction.0 = transform.rotation * Vec3::Z;
        })
    }
}

fn player_movement(
    players: Query<(
        &mut LinearVelocity,
        &ActionState<PlayerInput>,
        &CharacterBody,
    )>,
    time: Res<Time>,
) {
    for (mut velocity, input, body) in players {
        bevy::app::hotpatch::call(|| {
            if body.grounded {
                let target_velocity = input.axis_pair(&PlayerInput::Move);
            }
            else {
                println!("You really need a state machine or a component that determines movement stats on several states");
            }
        })
    }
}

fn player_gravity(players: Query<(&mut LinearVelocity, &CharacterBody)>, time: Res<Time>) {
    for (mut velocity, body) in players {
        bevy::app::hotpatch::call(|| {
            if body.grounded {
                velocity.y = 0.0;
            } else {
                if velocity.y > 0.0 {
                    velocity.y -= time.delta_secs() * 10.0;
                }
                else {
                    velocity.y -= time.delta_secs() * 15.0;
                }
            }
        })
    }
}
