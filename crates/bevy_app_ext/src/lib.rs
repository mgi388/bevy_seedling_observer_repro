use bevy_app::prelude::*;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::AppExt as _;
}

pub trait AppExt {
    fn try_add_plugins<T: Plugin>(&mut self, plugin: T) -> &mut Self;
}

impl AppExt for App {
    fn try_add_plugins<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        if !self.is_plugin_added::<T>() {
            self.add_plugins(plugin);
        }
        self
    }
}
