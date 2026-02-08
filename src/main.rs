use avian3d::prelude::*;
use bevy::prelude::*;

mod character_body;
mod input;
mod player;

fn main() {
    let mut app = App::new();

    app.insert_resource(Time::from_hz(60.0));

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

    app.run();
}

fn test_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("test_level.glb")),
    ));

    commands.spawn((
        player::PlayerCharacterMarker,
        input::PlayerInput::default_input_map(),
        Transform::from_xyz(0.0, 0.5, 0.0),
        children![(
            player::CameraPivot,
            Transform::from_xyz(0.0, 1.0, 0.0),
            children![(Camera3d::default(), Transform::from_xyz(0.0, 0.0, 10.0))]
        )],
    ));
}
