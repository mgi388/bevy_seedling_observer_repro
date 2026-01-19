use bevy_app::prelude::*;
use bevy_app_ext::prelude::*;
use bevy_camera::prelude::*;
use bevy_ecs::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
#[cfg(feature = "reflect")]
use bevy_reflect::prelude::*;
use bevy_render::view::Hdr;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.try_add_plugins(EnhancedInputPlugin);

        app.try_add_plugins(EguiPlugin::default());
        app.init_resource::<InspectorEnabled>();
        app.add_observer(on_toggle_inspector);
        // Intentionally use `add_plugins` so we panic if the plugin is already
        // added.
        app.add_plugins(
            WorldInspectorPlugin::new().run_if(|enabled: Res<InspectorEnabled>| enabled.0),
        );
        app.insert_resource(bevy_egui::EguiGlobalSettings {
            auto_create_primary_context: false,
            ..Default::default()
        });

        app.add_systems(Startup, spawn_egui_inspector_camera);
    }
}

pub fn spawn_egui_inspector_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Egui inspector camera"),
        Camera3d::default(),
        Camera {
            order: isize::MAX, // something very high
            ..Default::default()
        },
        Hdr,
        // This is required for bevy-inspector-egui to work.
        //
        // See https://github.com/jakobhellermann/bevy-inspector-egui/issues/286.
        bevy_egui::PrimaryEguiContext,
    ));
}

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleInspector;

pub fn on_toggle_inspector(
    _toggle: On<Start<ToggleInspector>>,
    mut enabled: ResMut<InspectorEnabled>,
) {
    enabled.0 = !enabled.0;
}

#[derive(Default, Resource)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Resource))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
pub struct InspectorEnabled(pub bool);
