use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use leafwing_input_manager::prelude::*;

mod character_body;
mod input;
mod player;

use input::PlayerInput;

//extremely incohesive feverdream

fn main() {
    let mut app = App::new();

    app.insert_resource(Time::from_hz(60.0));

    app.add_plugins((
        DefaultPlugins,
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        SeedlingPlugin::default(),
        PhysicsPlugins::new(FixedUpdate),
        PhysicsDebugPlugin::default(),
        character_body::CharacterBodyPlugin,
        input::InputPlugin,
        player::PlayerPlugin,
        bevy_skein::SkeinPlugin::default(),
    ));

    app.add_systems(Startup, (test_setup, cursor_grab));

    app.add_systems(FixedUpdate, (update_speed,));

    app.run();
}

fn cursor_grab(
    mut q_windows: Query<&mut bevy::window::CursorOptions, With<bevy::window::PrimaryWindow>>,
) {
    let mut cursor = q_windows.single_mut().unwrap();

    cursor.
}

fn test_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    /*commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 20.0).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.0,
            0.0,
            0.0,
        )),
    ));*/

    commands.spawn((
        Name::new("Player"),
        Transform::from_xyz(0.0, 0.5, 0.0),
        player::PlayerCharacterMarker,
        input::PlayerInput::default_input_map(),
        children![(
            player::CameraPivot,
            Transform::from_xyz(0.0, 1.0, 0.0),
            children![(Camera3d::default(), Transform::from_xyz(0.0, 0.0, 2.0),)]
        ),],
    ));

    commands.spawn((SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("test_level.glb")),
    ),));

    /*commands.spawn((
        Collider::cuboid(50.0, 0.01, 50.0),
        RigidBody::Static,
        Transform::from_xyz(0.0, -0.05, 0.0),
    ));

    commands.spawn((
        Collider::cuboid(10.0, 0.5, 10.0),
        RigidBody::Static,
        Transform::from_xyz(0.0, 0.0, -20.0),
    ));
    commands.spawn((
        Collider::cuboid(10.0, 0.5, 10.0),
        RigidBody::Static,
        Transform::from_xyz(0.0, 0.0, -20.0).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.0,
            45.0_f32.to_radians(),
            0.0,
        )),
    ));*/
}

fn update_speed(query: Query<(&mut LinearVelocity, &ActionState<PlayerInput>)>) {
    for (mut velocity, inputs) in query {
        bevy::app::hotpatch::call(|| {
            let dual_axis = inputs.axis_pair(&PlayerInput::Move) * 0.05;

            velocity.x += dual_axis.x;
            velocity.z += -dual_axis.y;
        });
    }
}
