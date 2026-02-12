use super::{PlayerCharacterMarker, PlayerLookDirection};
use crate::input::PlayerInput;

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraPivot>();

        app.add_systems(
            Update,
            (
                (rotate_camera_manual, rotate_camera_auto, move_camera),
                (update_camera_direction, unstuck_camera),
            )
                .chain(),
        );
    }
}

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

const CAMERA_ROTATION_SPEED: f32 = 0.75;
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

        transform.rotate_y(-angle_diff * time.delta_secs() * CAMERA_ROTATION_SPEED);
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

const PREDICTED_TIME: f32 = 0.2;
const SPEED_CAMERA: f32 = 10.0;
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
        let top_of_player = top_of_player + Vec3::Y * super::PLAYER_HEIGHT / 2.0;

        let cast = spatial_query.cast_ray(
            top_of_player,
            Dir3::new(velocity.0).unwrap_or(Dir3::Y),
            (velocity.0 * PREDICTED_TIME).length(),
            false,
            &SpatialQueryFilter::from_excluded_entities([pivot.0]),
        );

        let target_point = if let Some(cast) = cast {
            top_of_player + velocity.0.normalize_or_zero() * cast.distance
        } else {
            top_of_player + velocity.0 * PREDICTED_TIME
        };

        transform.translation = transform.translation.move_towards(
            target_point,
            time.delta_secs() * (transform.translation - target_point).length() * SPEED_CAMERA,
        );
    }
}

const CAMERA_DISTANCE: f32 = 5.0;
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
                max_distance: CAMERA_DISTANCE,
                target_distance: 0.0,
                compute_contact_on_penetration: true,
                ignore_origin_penetration: true,
            },
            &SpatialQueryFilter::from_excluded_entities([pivot.0]),
        );

        if let Some(cast) = cast {
            transform.translation.z = cast.distance;
        } else {
            transform.translation.z = CAMERA_DISTANCE;
        }
    }
}
