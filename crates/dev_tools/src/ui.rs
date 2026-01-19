use bevy_app::prelude::*;
use bevy_app_ext::prelude::*;
use bevy_ecs::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_ui_render::UiDebugOptions;

pub struct UiDevToolsPlugin;

impl Plugin for UiDevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.try_add_plugins(EnhancedInputPlugin);

        app.insert_resource(UiDebugOptions {
            enabled: false,
            ..Default::default()
        });
        app.add_observer(on_toggle_ui_overlay);
    }
}

#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleUiOverlay;

pub fn on_toggle_ui_overlay(
    _toggle: On<Start<ToggleUiOverlay>>,
    mut options: ResMut<UiDebugOptions>,
) {
    options.toggle();
}
