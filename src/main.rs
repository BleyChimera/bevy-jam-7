use avian3d::prelude::*;
use bevy::prelude::*;

use std::collections::HashMap;

mod character_body;
mod input;
mod player;

const MISERERE_PATH: &str = "miserere.glb";
const TEST_MAP: &str = "test_level.glb";
const MAIN_MAP: &str = "main_level.glb";

fn main() {
    let mut app = App::new();

    app.insert_resource(Time::from_hz(60.0));

    app.insert_resource(RunTimer {
        time: 0.0,
        finished: false,
    });

    app.register_type::<MiserereAnimationTarget>()
        .register_type::<MiserereAnimationsConnector>()
        .register_type::<ColliderContructorWithFlagsBecauseSkeinDoesntSupportThem>()
        .register_type::<WinCondition>()
        .register_type::<TimerMarker>();

    app.add_plugins((
        DefaultPlugins,
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        bevy_skein::SkeinPlugin::default(),
        PhysicsPlugins::new(FixedUpdate),
        PhysicsDebugPlugin::default(),
        input::InputPlugin,
        player::PlayerPlugin,
        character_body::CharacterBodyPlugin,
    ));

    app.add_systems(Startup, (main_setup, change_debug_phys_config));

    app.add_systems(FixedUpdate, (tick_game, reset_if_lost));

    app.add_systems(
        Update,
        (
            swap_mouse_state,
            load_animations_from_gltf,
            get_animation_target,
            test_animation,
            update_ui,
        ),
    );

    app.add_observer(switcheroo);

    app.add_observer(enable_shadows_spot);
    app.add_observer(enable_shadows_point);
    app.add_observer(enable_shadows_dir);

    app.run();
}

fn tick_game(mut run_timer: ResMut<RunTimer>, time: Res<Time>) {
    if !run_timer.finished {
        run_timer.time += time.delta_secs();
    }
}

fn change_debug_phys_config(mut gizmo_config: ResMut<GizmoConfigStore>) {
    let (gizmo_config, physics_gizmos) = gizmo_config.config_mut::<PhysicsGizmos>();

    gizmo_config.enabled = false;

    physics_gizmos.collider_color = Some(bevy::color::palettes::basic::GREEN.into());
    gizmo_config.line.style = GizmoLineStyle::Dotted;
    gizmo_config.line.width = 2.5;
}

fn swap_mouse_state(
    mut window: Single<&mut bevy::window::CursorOptions, With<bevy::window::PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if window.grab_mode != bevy::window::CursorGrabMode::Locked {
            window.visible = false;
            window.grab_mode = bevy::window::CursorGrabMode::Locked;
        } else {
            window.visible = true;
            window.grab_mode = bevy::window::CursorGrabMode::None;
        }
    }
}

fn update_ui(query: Query<&mut Text, With<TimerMarker>>, run_timer: Res<RunTimer>) {
    for mut text in query {
        let new_text = format!("Run timer: {} seconds", run_timer.time);
        text.0 = new_text;
    }
}

#[derive(Debug, Reflect, Component)]
#[reflect(Component)]
struct TimerMarker;

fn reset_if_lost(
    query: Query<
        &mut Transform,
        (
            With<player::PlayerMarker>,
            Without<player::camera::CameraPivot>,
        ),
    >,
    mut query2: Query<
        &mut Transform,
        (
            With<player::camera::CameraPivot>,
            Without<player::PlayerMarker>,
        ),
    >,
    mut run_timer: ResMut<RunTimer>,
) {
    for mut player in query {
        if player.translation.y < -1579.36 {
            player.translation = Vec3::new(0.0, 5.5, 0.0);
            run_timer.time = 0.0;
            run_timer.finished = false;
            for mut cam in &mut query2 {
                cam.translation = Vec3::new(0.0, 5.5, 0.0);
            }
        }
    }
}

fn main_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(MAIN_MAP)),
    ));

    commands.spawn((Text::new("Technically UI"), TimerMarker));

    let player_cam_transform = Transform::from_xyz(0.0, 5.5, 0.0);

    let player = commands
        .spawn((
            Name::new("Player"),
            player::PlayerCharacterMarker,
            input::PlayerInput::default_input_map(),
            player_cam_transform.clone(),
            children![
                (
                    Name::new("Miserere model"),
                    MiserereSceneTarget,
                    Transform::from_xyz(0.0, -0.5, 0.0) //SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(MISERERE_PATH))),
                ),
                /*(
                    Name::new("Fog"),
                    bevy::light::FogVolume {
                        ..Default::default()
                    },
                    Transform::from_scale(Vec3::splat(10.0))
                )*/
            ],
        ))
        .id();

    commands.spawn((
        player::camera::CameraPivot(player),
        player_cam_transform,
        children![(
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.0, 10.0),
            bevy::core_pipeline::tonemapping::Tonemapping::AgX,
            bevy::post_process::bloom::Bloom::default(),
            /*bevy::light::VolumetricFog {
                ambient_intensity: 0.0,
                step_count: 64*2,
                ..default()
            },*/
            DistanceFog {
                falloff: FogFalloff::Linear {
                    start: 25.0,
                    end: 1500.0
                },
                color: bevy::color::palettes::basic::BLACK.into(),
                ..default()
            },
            SpotLight::default(),
            //bevy::post_process::auto_exposure::AutoExposure::default(),
            /*children![(
                Name::new("Fog"),
                bevy::light::FogVolume {
                    ..Default::default()
                },
                Transform::from_scale(Vec3::splat(100.0)).with_translation(Vec3::NEG_Z * 40.0),
            ),],*/
        )],
    ));

    commands.insert_resource(MiserereModel {
        gltf_handle: asset_server.load(MISERERE_PATH),
        animation_handle: graphs.add(AnimationGraph::new()),
        animation_nodes: HashMap::new(),
    });
}

#[derive(Resource)]
pub struct RunTimer {
    time: f32,
    finished: bool,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct WinCondition;

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

    mut player_tf: Single<
        &mut Transform,
        (
            With<player::PlayerMarker>,
            Without<player::camera::CameraPivot>,
        ),
    >,
    mut player_camera_tf: Single<
        &mut Transform,
        (
            With<player::camera::CameraPivot>,
            Without<player::PlayerMarker>,
        ),
    >,
    mut run_timer: ResMut<RunTimer>,
) {
    for event in gltf.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::LoadedWithDependencies { id } => {
                player_tf.translation = Vec3::new(0.0, 5.5, 0.0);
                player_camera_tf.translation = Vec3::new(0.0, 5.5, 0.0);
                run_timer.time = 0.0;

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
    model: Res<MiserereModel>,
) {
    for player in players {
        for target in targets {
            commands
                .entity(target)
                .insert((
                    MiserereAnimationsConnector(player.0),
                    AnimationGraphHandle(model.animation_handle.clone()),
                ))
                .remove::<MiserereAnimationTarget>();

            commands.entity(player.0).remove::<MiserereSceneTarget>();
        }
    }
}

fn enable_shadows_spot(trigger: On<Add, SpotLight>, mut lights: Query<&mut SpotLight>) {
    let mut light = lights.get_mut(trigger.entity).unwrap();
    light.shadows_enabled = true;
}

fn enable_shadows_dir(
    trigger: On<Add, DirectionalLight>,
    mut lights: Query<&mut DirectionalLight>,
) {
    let mut light = lights.get_mut(trigger.entity).unwrap();
    light.shadows_enabled = true;
}

fn enable_shadows_point(trigger: On<Add, PointLight>, mut lights: Query<&mut PointLight>) {
    let mut light = lights.get_mut(trigger.entity).unwrap();
    light.shadows_enabled = true;
}

// TODO: MOVE ANIMATION RELATED THINGS TO PLAYER MODULE
fn test_animation(
    player_model: Res<MiserereModel>,
    players: Query<(&player::state_machine::StateMachine, &LinearVelocity)>,
    animations: Query<(&MiserereAnimationsConnector, &mut AnimationPlayer)>,
    time: Res<Time>,
) {
    let idle_name = "Idle".to_string();
    let walk_name = "Walk".to_string();
    let slide_start = "SlideStart".to_string();
    let slide_name = "Slide".to_string();
    let crouch_start = "CrouchStart".to_string();
    let crouch_name = "Crouch".to_string();

    let glide_name = "Glide".to_string();
    let dive_name = "Dive".to_string();
    let air_up_name = "AirUp".to_string();
    let air_down_name = "AirDown".to_string();

    let jump_normal_name = "JumpNormal".to_string();
    let jump_crouch_name = "JumpCrouch".to_string();
    let jump_dive_name = "JumpDive".to_string();

    for (connector, mut animation) in animations {
        let mut stop_all_animations_but = |exceptions: &[&String]| {
            for (name, animation_clip) in player_model.animation_nodes.iter() {
                if !exceptions.contains(&name) {
                    animation.stop(animation_clip.clone());
                }
            }
        };

        let Ok((state, velocity)) = players.get(connector.0) else {
            continue;
        };

        match &state.movement_state {
            player::state_machine::MajorMoveState::Grounded(substate) => match substate {
                player::state_machine::MinorGroundState::Moving => {
                    stop_all_animations_but(&[&idle_name, &walk_name]);

                    let ratio = velocity.length() / 10.0;

                    animation
                        .play(
                            player_model
                                .animation_nodes
                                .get(&idle_name)
                                .unwrap()
                                .clone(),
                        )
                        .set_weight((1.0 - ratio).clamp(0.0, 1.0))
                        .repeat();
                    animation
                        .play(
                            player_model
                                .animation_nodes
                                .get(&walk_name)
                                .unwrap()
                                .clone(),
                        )
                        .set_weight(ratio.clamp(0.0, 1.0))
                        .repeat();
                }
                player::state_machine::MinorGroundState::Sliding => {
                    stop_all_animations_but(&[&slide_start, &slide_name]);

                    let slide_start = player_model.animation_nodes.get(&slide_start).unwrap();

                    animation.play(slide_start.clone()).set_weight(1.0);

                    let mut play_slide = false;
                    'check: for (node, animation_clip) in animation.playing_animations() {
                        if node == slide_start {
                            if animation_clip.is_finished() {
                                play_slide = true;
                                break 'check;
                            }
                        }
                    }
                    if play_slide {
                        animation
                            .play(
                                player_model
                                    .animation_nodes
                                    .get(&slide_name)
                                    .unwrap()
                                    .clone(),
                            )
                            .set_weight(1.0)
                            .repeat();
                    }
                }
                player::state_machine::MinorGroundState::Crouched => {
                    stop_all_animations_but(&[&crouch_start, &crouch_name]);

                    let crouch_start = player_model.animation_nodes.get(&crouch_start).unwrap();

                    animation.play(crouch_start.clone()).set_weight(1.0);

                    let mut play_slide = false;
                    'check: for (node, animation_clip) in animation.playing_animations() {
                        if node == crouch_start {
                            if animation_clip.is_finished() {
                                play_slide = true;
                                break 'check;
                            }
                        }
                    }
                    if play_slide {
                        animation
                            .play(
                                player_model
                                    .animation_nodes
                                    .get(&crouch_name)
                                    .unwrap()
                                    .clone(),
                            )
                            .set_weight(1.0)
                            .repeat();
                    }
                }
            },
            player::state_machine::MajorMoveState::Airborne(substate) => match substate {
                player::state_machine::MinorAirborneState::Jumping(jump_type) => match jump_type {
                    player::state_machine::JumpType::Normal(_) => {
                        stop_all_animations_but(&[&jump_normal_name]);

                        animation
                            .play(
                                player_model
                                    .animation_nodes
                                    .get(&jump_normal_name)
                                    .unwrap()
                                    .clone(),
                            )
                            .set_weight(1.0);
                    }
                    player::state_machine::JumpType::Crouch(_) => {
                        stop_all_animations_but(&[&jump_crouch_name]);

                        animation
                            .play(
                                player_model
                                    .animation_nodes
                                    .get(&jump_crouch_name)
                                    .unwrap()
                                    .clone(),
                            )
                            .set_weight(1.0);
                    }
                    player::state_machine::JumpType::Dive(_) => {
                        stop_all_animations_but(&[&jump_dive_name]);

                        animation
                            .play(
                                player_model
                                    .animation_nodes
                                    .get(&jump_dive_name)
                                    .unwrap()
                                    .clone(),
                            )
                            .set_weight(1.0);
                    }
                },
                player::state_machine::MinorAirborneState::Glide => {
                    stop_all_animations_but(&[&glide_name]);

                    animation
                        .play(
                            player_model
                                .animation_nodes
                                .get(&glide_name)
                                .unwrap()
                                .clone(),
                        )
                        .set_weight(1.0)
                        .repeat();
                }
                player::state_machine::MinorAirborneState::Dive => {
                    stop_all_animations_but(&[&dive_name]);

                    animation
                        .play(
                            player_model
                                .animation_nodes
                                .get(&dive_name)
                                .unwrap()
                                .clone(),
                        )
                        .set_weight(1.0);
                }
                player::state_machine::MinorAirborneState::Falling => {
                    stop_all_animations_but(&[
                        &jump_normal_name,
                        &jump_dive_name,
                        &jump_crouch_name,
                        &air_up_name,
                        &air_down_name,
                    ]);

                    let y_factor = velocity.y.clamp(-1.0, 1.0);

                    let mut play_animation = true;
                    let nodes = [
                        player_model.animation_nodes.get(&jump_normal_name).unwrap(),
                        player_model.animation_nodes.get(&jump_dive_name).unwrap(),
                        player_model.animation_nodes.get(&jump_crouch_name).unwrap(),
                    ];

                    'search: for (node, animation_clip) in animation.playing_animations_mut() {
                        for compare in nodes {
                            if node == compare {
                                if animation_clip.is_finished() {
                                    play_animation = true;
                                    animation_clip.set_weight(
                                        animation_clip
                                            .weight()
                                            .lerp(0.0, time.delta_secs() * 10.0)
                                            .max(0.0),
                                    );
                                    break 'search;
                                } else {
                                    play_animation = false;
                                }
                            }
                        }
                    }

                    if play_animation {
                        let air_up = animation
                            .play(
                                player_model
                                    .animation_nodes
                                    .get(&air_up_name)
                                    .unwrap()
                                    .clone(),
                            )
                            .repeat();
                        air_up.set_weight(
                            air_up
                                .weight()
                                .lerp(y_factor.max(0.0), time.delta_secs().max(y_factor.max(0.0))),
                        );

                        let air_down = animation
                            .play(
                                player_model
                                    .animation_nodes
                                    .get(&air_down_name)
                                    .unwrap()
                                    .clone(),
                            )
                            .repeat();

                        air_down.set_weight(air_down.weight().lerp(
                            (-y_factor).max(0.0),
                            time.delta_secs().max((-y_factor).max(0.0)),
                        ));
                    }
                }
            },
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct ColliderContructorWithFlagsBecauseSkeinDoesntSupportThem;

fn switcheroo(
    trigger: On<Add, ColliderContructorWithFlagsBecauseSkeinDoesntSupportThem>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .remove::<ColliderContructorWithFlagsBecauseSkeinDoesntSupportThem>()
        .insert(ColliderConstructor::TrimeshFromMeshWithConfig(
            TrimeshFlags::FIX_INTERNAL_EDGES,
        ));
}
