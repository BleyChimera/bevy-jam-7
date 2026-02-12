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

    app.add_observer(get_animation_target);

    app.add_systems(Update, (load_animations_from_gltf, test_animation));

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
        info!("{:?}", event);
        match event {
            AssetEvent::Added { id } | AssetEvent::LoadedWithDependencies { id } => {
                if player_model.gltf_handle.id() == *id {
                    let miserere = gltfs.get(*id).unwrap();
                    for entity in scene_instantiate {
                        commands
                            .entity(entity)
                            .remove::<MiserereSceneTarget>()
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
    trigger: On<Add, MiserereAnimationTarget>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
    players: Query<Entity, With<player::PlayerMarker>>,
) {
    let entity = trigger.entity;
    let player_entity;

    let mut parent = parents.get(entity).unwrap().0;
    'search: loop {
        if let Ok(player) = players.get(parent) {
            player_entity = player;
            break 'search;
        } else {
            parent = parents.get(parent).unwrap().0;
        }
    }

    commands
        .entity(entity)
        .insert(MiserereAnimationsConnector(player_entity)).remove::<MiserereAnimationTarget>();
}

fn test_animation(mut player_model: ResMut<MiserereModel>) {}
