use avian3d::prelude::*;
use bevy::prelude::*;

use std::collections::HashMap;

mod character_body;
mod input;
mod player;

const MISERERE_PATH: &str = "miserere.glb";

fn main() {
    let mut app = App::new();

    app.insert_resource(Time::from_hz(60.0));

    app.register_type::<MiserereAnimationTarget>()
        .register_type::<MiserereAnimationsConnector>();

    app.add_plugins((
        DefaultPlugins,
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        bevy_skein::SkeinPlugin::default(),
        PhysicsPlugins::new(FixedUpdate),
        PhysicsDebugPlugin::default(),
        bevy_seedling::SeedlingPlugin::default(),
        input::InputPlugin,
        player::PlayerPlugin,
        character_body::CharacterBodyPlugin,
    ));

    app.add_systems(Startup, (test_setup,));

    app.add_systems(
        Update,
        (
            load_animations_from_gltf,
            get_animation_target,
            test_animation,
        ),
    );

    app.run();
}

fn test_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("test_level.glb")),
    ));

    let player_cam_transform = Transform::from_xyz(0.0, 10.5, 0.0);

    let player = commands
        .spawn((
            Name::new("Player"),
            player::PlayerCharacterMarker,
            input::PlayerInput::default_input_map(),
            player_cam_transform.clone(),
            children![(
                MiserereSceneTarget,
                Transform::from_xyz(0.0, -0.5, 0.0) //SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(MISERERE_PATH))),
            )],
        ))
        .id();

    commands.spawn((
        player::camera::CameraPivot(player),
        player_cam_transform,
        children![(Camera3d::default(), Transform::from_xyz(0.0, 0.0, 10.0))],
    ));

    commands.insert_resource(MiserereModel {
        gltf_handle: asset_server.load(MISERERE_PATH),
        animation_handle: graphs.add(AnimationGraph::new()),
        animation_nodes: HashMap::new(),
    });
}

#[derive(Resource)]
pub struct MiserereModel {
    gltf_handle: Handle<Gltf>,
    animation_handle: Handle<AnimationGraph>,
    animation_nodes: HashMap<String, AnimationNodeIndex>,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct MiserereSceneTarget;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct MiserereAnimationTarget;

#[derive(Component, Reflect)]
struct MiserereAnimationsConnector(Entity);

fn load_animations_from_gltf(
    mut commands: Commands,
    mut gltf: MessageReader<AssetEvent<Gltf>>,
    scene_instantiate: Query<Entity, With<MiserereSceneTarget>>,
    mut player_model: ResMut<MiserereModel>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for event in gltf.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::LoadedWithDependencies { id } => {
                if player_model.gltf_handle.id() == *id {
                    let miserere = gltfs.get(*id).unwrap();
                    for entity in scene_instantiate {
                        commands
                            .entity(entity)
                            .insert((SceneRoot(miserere.scenes.get(0).unwrap().clone()),));
                    }

                    let graph = graphs.get_mut(&player_model.animation_handle).unwrap();
                    for (name, animation) in &miserere.named_animations {
                        let animation_node = graph.add_clip(animation.clone(), 1.0, graph.root);

                        player_model
                            .animation_nodes
                            .insert(name.to_string(), animation_node);
                    }
                }
            }
            AssetEvent::Unused { id: _ }
            | AssetEvent::Removed { id: _ }
            | AssetEvent::Modified { id: _ } => {}
        }
    }
}

fn get_animation_target(
    mut commands: Commands,
    players: Query<&ChildOf, With<MiserereSceneTarget>>,
    targets: Query<Entity, With<MiserereAnimationTarget>>,
) {
    for player in players {
        for target in targets {
            commands
                .entity(target)
                .insert(MiserereAnimationsConnector(player.0))
                .remove::<MiserereAnimationTarget>();

            commands.entity(player.0).remove::<MiserereSceneTarget>();
        }
    }
}

fn test_animation(
    player_model: Res<MiserereModel>,
    players: Query<(&player::state_machine::StateMachine, &LinearVelocity)>,
    animations: Query<(&MiserereAnimationsConnector, &mut AnimationPlayer)>,
) {
    for (connector, mut animation) in animations {
        let Ok((state, velocity)) = players.get(connector.0) else {
            continue;
        };

        match &state.movement_state {
            player::state_machine::MajorMoveState::Grounded(substate) => match substate {
                player::state_machine::MinorGroundState::Moving => {
                    let ratio = velocity.length() / 10.0;

                    animation
                        .play(player_model.animation_nodes.get("Idle").unwrap().clone())
                        .set_weight(1.0 - ratio);
                    animation
                        .play(player_model.animation_nodes.get("Walk").unwrap().clone())
                        .set_weight(ratio);
                }
                _ => {}
            },
            player::state_machine::MajorMoveState::Airborne(_) => {}
        }
    }
}
