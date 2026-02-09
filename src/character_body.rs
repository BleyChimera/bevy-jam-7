use avian3d::prelude::*;
use bevy::prelude::*;

pub struct CharacterBodyPlugin;

impl Plugin for CharacterBodyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterBody>()
            .register_type::<CharacterGroundSnap>()
            .register_type::<ForceSlide>();

        app.add_systems(
            FixedUpdate,
            ((character_body_movement, character_body_snap).chain()).in_set(PhysicsSystems::Last),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Component)]
#[reflect(Component)]
pub struct ForceSlide;

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
    pub last_normal: Dir3,
    pub force_slide: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Component)]
#[reflect(Component)]
pub struct CharacterGroundSnap {
    pub distance: f32,
}

fn character_body_movement(
    sliding: MoveAndSlide,
    bodies: Query<(
        Entity,
        &mut CharacterBody,
        &Collider,
        &mut Transform,
        &mut LinearVelocity,
        Option<&CharacterGroundSnap>,
    )>,
    force_slide: Query<&ForceSlide>,
    time: Res<Time>,
) {
    for (entity, mut body, collider, mut transform, mut velocity, snap) in bodies.into_iter() {
        body.force_slide = false;

        if snap.is_none() {
            body.grounded = false;
        }

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
                body.last_normal = *result.normal;

                if force_slide.get(result.entity).is_ok() {
                    body.force_slide = true;
                }

                MoveAndSlideHitResponse::Accept
            },
        );

        transform.translation = move_result.position;
        velocity.0 = move_result.projected_velocity;
    }
}

fn character_body_snap(
    sliding: MoveAndSlide,
    bodies: Query<(
        Entity,
        &mut CharacterBody,
        &Collider,
        &mut Transform,
        &mut LinearVelocity,
        &CharacterGroundSnap,
    )>,
    force_slide: Query<&ForceSlide>,
) {
    for (entity, mut body, collider, mut transform, mut velocity, snap) in bodies.into_iter() {
        // TODO: ADD VELOCITY DEPENDANT SNAPPING
        let _ = velocity;

        if !body.grounded {
            continue;
        }

        let mut touched_floor = false;
        let snap_movement = sliding.move_and_slide(
            collider,
            transform.translation,
            transform.rotation,
            -*body.up * snap.distance,
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
                    //velocity.0 = (velocity.0.reject_from_normalized(hit.normal.as_vec3())).normalize_or_zero() * velocity.0.length();
                }

                if force_slide.get(hit.entity).is_ok() {
                    body.force_slide = true;
                }

                body.last_normal = *hit.normal;
                MoveAndSlideHitResponse::Accept
            },
        );
        if touched_floor {
            transform.translation = snap_movement.position;
        } else {
            body.grounded = false;
        }
    }
}
