pub mod asset;

mod bevy_seedling_impl;
mod sound_effects;
pub mod sound_extension;

use core::marker::PhantomData;

use bevy_app::prelude::*;
use bevy_app_ext::prelude::*;
use bevy_asset::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_ecs::prelude::*;
use bevy_platform::collections::HashMap;
#[cfg(feature = "reflect")]
use bevy_reflect::{GetTypeRegistration, Typed, prelude::*};
use darkomen::sound::sfx::SfxId;
use sound_config::prelude::*;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        SoundEffectPacketAssetCollection, SoundEffectPlugin, SpatialSoundEffect,
        asset::PacketAsset, sound_effects::SoundEffectKey,
    };

    #[doc(hidden)]
    pub use crate::bevy_seedling_impl::{Mode, SoundEffectPlayer};
}

use self::asset::*;

#[cfg(all(feature = "debug", feature = "reflect"))]
pub trait SoundEffectKeyRequirements:
    SoundEffectKey
    + core::fmt::Debug
    + FromReflect
    + Typed
    + GetTypeRegistration
    + Send
    + Sync
    + 'static
{
}
#[cfg(all(not(feature = "debug"), feature = "reflect"))]
pub trait SoundEffectKeyRequirements:
    SoundEffectKey + FromReflect + Typed + GetTypeRegistration + Send + Sync + 'static
{
}
#[cfg(all(feature = "debug", not(feature = "reflect")))]
pub trait SoundEffectKeyRequirements:
    SoundEffectKey + core::fmt::Debug + Send + Sync + 'static
{
}
#[cfg(all(not(feature = "debug"), not(feature = "reflect")))]
pub trait SoundEffectKeyRequirements: SoundEffectKey + Send + Sync + 'static {}

pub struct SoundEffectPlugin<SoundEffectKeyT: SoundEffectKeyRequirements>(
    PhantomData<SoundEffectKeyT>,
);

impl<SoundEffectKeyT: SoundEffectKeyRequirements> SoundEffectPlugin<SoundEffectKeyT> {
    pub fn new() -> Self {
        SoundEffectPlugin(PhantomData)
    }
}

impl<SoundEffectKeyT: SoundEffectKeyRequirements> Plugin for SoundEffectPlugin<SoundEffectKeyT> {
    fn build(&self, app: &mut App) {
        app.try_add_plugins(SoundConfigPlugin);
        app.try_add_plugins(SoundEffectAssetPlugin);

        app.try_add_plugins(crate::bevy_seedling_impl::SoundEffectPlugin::<
            SoundEffectKeyT,
        >::new());

        #[cfg(feature = "reflect")]
        {
            app.register_type::<SoundEffectPacketAssetCollection>();
            app.register_type::<SoundEffectId>();
            app.register_type::<RandomLoopingSoundEffect>();
            app.register_type::<SpatialSoundEffect>();
        }
    }
}

// TODO: This causes all packets to be loaded up front, which is not ideal. We
// should work out how to make screens/scenes specify the packets they only load
// what they need. Project instances spatial sound effect ID breaks down into a
// packet path and sound effect ID and based on that packet path, the original
// only loads the packets needed for that battle. We should be able to do a
// similar thing, e.g., the "Book" screen only needs to load the book-related/UI
// sound effect packets. Re the project instances comment here: The BTB file is
// the one that specifies the "spatial sound effect preset" to load and that
// preset has N packets that it loads.
#[derive(AssetCollection, Resource)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Resource))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
pub struct SoundEffectPacketAssetCollection {
    #[asset(path = "DARKOMEN/SOUND/H", collection(mapped, typed))]
    packets: HashMap<String, Handle<PacketAsset>>,
}

impl SoundEffectPacketAssetCollection {
    #[inline(always)]
    pub fn get(&self, name: &str) -> Option<Handle<PacketAsset>> {
        self.packets.get(name).cloned()
    }
}

pub trait SoundEffectKey {
    fn get_packet_path(&self) -> String;
    fn get_sound_effect_id(&self) -> u8;
}

#[derive(Clone, Component, Copy, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component, Default))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
pub(crate) struct SoundEffectId(pub(crate) SfxId);

#[derive(Clone, Component, Copy, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component, Default))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
pub(crate) struct RandomLoopingSoundEffect;

#[derive(Clone, Component, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component, Default))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
pub struct SpatialSoundEffect {
    pub(crate) packet: Handle<PacketAsset>,
    pub(crate) id: SfxId,
}

impl SpatialSoundEffect {
    /// Creates a new spatial sound effect component.
    pub fn new(packet: Handle<PacketAsset>, id: SfxId) -> Self {
        SpatialSoundEffect { packet, id }
    }
}
