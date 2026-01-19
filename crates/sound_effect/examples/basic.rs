use bevy::{color::palettes::tailwind::*, prelude::*, render::view::Hdr};
use bevy_app_ext::prelude::*;
use bevy_args::{BevyArgsPlugin, Deserialize, Parser, Serialize};
use bevy_asset_loader::prelude::*;
use bevy_editor_cam::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_seedling::spatial::SpatialListener3D;
use dev_tools::prelude::*;
use sound_config::prelude::*;
use sound_effect::prelude::*;

fn spatial_bundle(
    mesh_assets: &mut ResMut<Assets<Mesh>>,
    standard_materials: &mut ResMut<Assets<StandardMaterial>>,
    packet: &Handle<PacketAsset>,
    id: u8,
) -> impl Bundle {
    (
        SpatialSoundEffect::new(packet.clone(), id),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(mesh_assets.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: TEAL_400.into(),
            ..Default::default()
        })),
    )
}

#[derive(InputAction)]
#[action_output(bool)]
struct Play;

fn on_play(
    _: On<Start<Play>>,
    mut commands: Commands,
    args: Res<Args>,
    asset_server: Res<AssetServer>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    if args.spatial {
        commands.spawn(spatial_bundle(
            &mut mesh_assets,
            &mut standard_materials,
            &asset_server.load(&args.packet_path),
            args.id,
        ));
    } else {
        commands.spawn(SoundEffectPlayer::new(SoundEffectKey::custom(
            args.packet_path.clone(),
            args.id,
        )));
    }
}

fn maybe_auto_play(
    mut commands: Commands,
    args: Res<Args>,
    asset_server: Res<AssetServer>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    if !args.auto_play {
        return;
    }

    if args.spatial {
        commands.spawn(spatial_bundle(
            &mut mesh_assets,
            &mut standard_materials,
            &asset_server.load(&args.packet_path),
            args.id,
        ));
    } else {
        commands.spawn(SoundEffectPlayer::new(SoundEffectKey::custom(
            args.packet_path.clone(),
            args.id,
        )));
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct EditorCamera;

fn setup_scene(mut commands: Commands) {
    let camera_transform =
        Transform::from_translation(Vec3::new(-5.0, 5.0, 0.0)).looking_at(Vec3::ZERO, Vec3::Y); // looking at origin

    commands.spawn((
        Name::new("Editor camera"),
        DespawnOnExit(State::Loaded),
        EditorCamera,
        Camera3d::default(),
        Camera {
            ..Default::default()
        },
        Hdr,
        EditorCam::default(),
        camera_transform,
        SpatialListener3D,
    ));

    commands.spawn((InfiniteGridBundle::default(), DespawnOnExit(State::Loaded)));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Player;

fn setup_player(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        Player,
        actions!(Player[
            (
                Action::<ToggleInspector>::new(),
                bindings![KeyCode::F12],
            ),
            (
                Action::<Play>::new(),
                bindings![KeyCode::Space],
            ),
        ]),
    ));
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, States, PartialEq)]
enum State {
    #[default]
    Loading,
    Loaded,
    Error,
}

#[derive(Default, Deserialize, Parser, Resource, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Deserialize, Resource, Serialize)
)]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
struct Args {
    /// The sound effect packet path, e.g., "DARKOMEN/SOUND/H/MEET.H".
    #[arg(long)]
    packet_path: String,

    /// The sound effect ID within the packet, e.g., 5.
    #[arg(long)]
    id: u8,

    /// The global volume in decibels, e.g., -10.0.
    #[arg(long, default_value_t = -20.0)]
    global_volume_decibels: f32,

    /// Whether to auto play the sound effect on startup.
    #[arg(long, default_value_t = false)]
    auto_play: bool,

    /// Whether the sound effect is spatial (3D) or not.
    #[arg(long, default_value_t = false)]
    spatial: bool,

    /// Whether to not use the deferred approach. If true, uses the observer
    /// approach which is the one with the bug spawning many nodes.
    ///
    /// TODO: Not implemented yet.
    #[arg(long, default_value_t = true)]
    no_deferred: bool,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(BevyArgsPlugin::<Args>::default());

    app.add_plugins(DefaultPlugins);

    app.add_plugins(InspectorPlugin);
    app.add_plugins(InfiniteGridPlugin);
    app.add_plugins(DefaultEditorCamPlugins);

    app.try_add_plugins(EnhancedInputPlugin);

    app.init_state::<State>();

    app.add_input_context::<Player>();
    app.register_type::<Player>();

    app.add_systems(OnEnter(State::Loading), setup_player);

    app.add_loading_state(
        LoadingState::new(State::Loading)
            .load_collection::<SoundEffectPacketAssetCollection>()
            .continue_to_state(State::Loaded)
            .on_failure_continue_to_state(State::Error),
    );
    // Remove the loaded resources in reverse order when the state ends.
    app.add_systems(OnExit(State::Loaded), |mut commands: Commands| {
        commands.remove_resource::<SoundEffectPacketAssetCollection>();
    });

    app.add_systems(OnEnter(State::Error), || error!("Error loading assets"));

    app.add_plugins(SoundEffectPlugin::<SoundEffectKey>::new());

    app.add_systems(
        Startup,
        move |args: Res<Args>, mut config: ResMut<SoundConfig>, mut mode: ResMut<Mode>| {
            config.global_volume_decibels = args.global_volume_decibels;

            mode.deferred = !args.no_deferred;
        },
    );

    app.add_observer(on_play);

    app.add_systems(
        OnEnter(State::Loaded),
        (setup_scene, maybe_auto_play).chain(),
    );

    app.run();
}
