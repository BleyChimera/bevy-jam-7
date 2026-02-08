use avian3d::prelude::*;
use bevy::prelude::*;

pub struct CharacterBodyPlugin;

impl Plugin for CharacterBodyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterBody>();

        app.add_systems(
            FixedUpdate,
            (character_body_movement,).in_set(PhysicsSystems::Last),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Component)]
#[require(
    RigidBody::Kinematic,
    LinearVelocity,
    CustomPositionIntegration,
    TransformInterpolation
)]
#[reflect(Component)]
pub struct CharacterBody {
    pub grounded: bool,
    pub up: Dir3,
    pub max_dot_variance: f32,
}

fn character_body_movement(
    sliding: MoveAndSlide,
    bodies: Query<(
        Entity,
        &mut CharacterBody,
        &Collider,
        &mut Transform,
        &mut LinearVelocity,
    )>,
    time: Res<Time>,
) {
    for (entity, mut body, collider, mut transform, mut velocity) in bodies.into_iter() {
        let move_result = sliding.move_and_slide(
            collider,
            transform.translation,
            transform.rotation,
            velocity.0,
            time.delta(),
            &MoveAndSlideConfig {
                move_and_slide_iterations: 255,
                skin_width: 0.01,
                ..Default::default()
            },
            &SpatialQueryFilter::from_excluded_entities([entity]),
            |result| {
                if result.normal.dot(*body.up) > body.max_dot_variance {
                    body.grounded = true;
                }
                MoveAndSlideHitResponse::Accept
            },
        );

        transform.translation = move_result.position;
        velocity.0 = move_result.projected_velocity;

        if !body.grounded {
            continue;
        }

        let mut touched_floor = false;
        let snap_movement = sliding.move_and_slide(
            collider,
            transform.translation,
            transform.rotation,
            -*body.up,
            std::time::Duration::from_secs(1),
            &MoveAndSlideConfig {
                move_and_slide_iterations: 1,
                skin_width: 0.01,
                ..Default::default()
            },
            &SpatialQueryFilter::from_excluded_entities([entity]),
            |hit| {
                if hit.normal.dot(*body.up) > body.max_dot_variance {
                    touched_floor = true;
                }
                MoveAndSlideHitResponse::Accept
            },
        );
        if touched_floor {
            transform.translation = snap_movement.position;
        } else {
            body.grounded = false
        }
    }
}
