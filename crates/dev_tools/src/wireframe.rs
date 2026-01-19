use bevy_app::prelude::*;
use bevy_app_ext::prelude::*;
use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_pbr::wireframe::*;

pub struct WireframeDevToolsPlugin;

impl Plugin for WireframeDevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.try_add_plugins(EnhancedInputPlugin);

        app.try_add_plugins(WireframePlugin::default());
        app.insert_resource(WireframeConfig {
            global: false,
            default_color: Color::WHITE,
        });
        app.add_observer(on_toggle_wireframes);
    }
}

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleWireframes;

pub fn on_toggle_wireframes(
    _toggle: On<Start<ToggleWireframes>>,
    mut config: ResMut<WireframeConfig>,
) {
    config.global = !config.global;
}
