pub mod inspector;
pub mod ui;
pub mod wireframe;

use bevy_app::prelude::*;
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_gizmos::prelude::*;
#[cfg(feature = "reflect")]
use bevy_reflect::prelude::*;
use bevy_transform::prelude::*;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{ShowAxes, inspector::*, ui::*, wireframe::*};
}

pub struct DevToolsPlugin;

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "reflect")]
        app.register_type::<ShowAxes>();

        app.add_systems(PostUpdate, draw_axes.after(TransformSystems::Propagate));
    }
}

#[derive(Clone, Component, Copy, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component, Default))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
pub struct ShowAxes;

fn draw_axes(mut gizmos: Gizmos, query: Query<(&GlobalTransform, Option<&Aabb>), With<ShowAxes>>) {
    for (&transform, maybe_aabb) in &query {
        let length = if let Some(aabb) = maybe_aabb {
            aabb.half_extents.length()
        } else {
            1.0 // fallback to this if no AABB
        };
        gizmos.axes(transform, length);
    }
}
