use core::marker::PhantomData;

use bevy_app::prelude::*;
use bevy_app_ext::prelude::*;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_rand::prelude::*;
#[cfg(feature = "reflect")]
use bevy_reflect::prelude::*;
use bevy_seedling::{
    context::SampleRate,
    firewheel::{
        dsp::distance_attenuation::{DistanceAttenuation, DistanceModel},
        nodes::spatial_basic::SpatialBasicNode,
    },
    prelude::*,
    sample_effects,
};
use bevy_transform::prelude::*;
use darkomen::prelude::*;
use rand::Rng;
use sound_config::prelude::*;
#[allow(unused_imports)]
use tracing::*;

use crate::{
    PacketAssetHandle, RandomLoopingSoundEffect, SoundEffectId, SoundEffectKeyRequirements,
    SoundEffectPacketAssetCollection, SpatialSoundEffect, asset::*, sound_extension::SoundExt as _,
};

pub(super) struct SoundEffectPlugin<SoundEffectKeyT: SoundEffectKeyRequirements>(
    PhantomData<SoundEffectKeyT>,
);

impl<SoundEffectKeyT: SoundEffectKeyRequirements> SoundEffectPlugin<SoundEffectKeyT> {
    pub(super) fn new() -> Self {
        SoundEffectPlugin(PhantomData)
    }
}

impl<SoundEffectKeyT: SoundEffectKeyRequirements> Plugin for SoundEffectPlugin<SoundEffectKeyT> {
    fn build(&self, app: &mut App) {
        app.try_add_plugins(EntropyPlugin::<WyRand>::default());
        app.try_add_plugins(SeedlingPlugin::default());

        app.init_resource::<Mode>();
        app.init_resource::<SpatialSoundEffectSettings>();

        #[cfg(feature = "reflect")]
        {
            app.register_type::<Mode>();
            app.register_type::<SpatialSoundEffectSettings>();

            app.register_type::<SoundEffectPlayer<SoundEffectKeyT>>();
            app.register_type::<RandomLoopingSoundPlayerMarker>();
        }

        app.add_observer(on_sound_effect_player_added::<SoundEffectKeyT>);
        app.add_observer(on_random_looping_sound_player_removed);
        app.add_observer(on_spatial_sound_effect_added);
    }
}

#[derive(Clone, Default, Resource)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Default, Resource))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
pub struct Mode {
    pub deferred: bool,
}

#[derive(Component)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
pub struct SoundEffectPlayer<K: SoundEffectKeyRequirements> {
    pub key: K,
}

impl<K: SoundEffectKeyRequirements> SoundEffectPlayer<K> {
    /// Creates a new sound effect player.
    pub fn new(key: K) -> SoundEffectPlayer<K> {
        SoundEffectPlayer { key }
    }
}

/// Settings for spatial sound effects.
///
/// This resource allows configuring the [`SpatialBasicNode`] used for spatial
/// audio effects.
#[derive(Clone, Resource)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Resource))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
struct SpatialSoundEffectSettings {
    /// Distance attenuation settings for spatial audio.
    distance_attenuation: DistanceAttenuation,
}

impl Default for SpatialSoundEffectSettings {
    fn default() -> Self {
        Self {
            distance_attenuation: DistanceAttenuation {
                distance_model: DistanceModel::Linear,
                reference_distance: 2.0,
                max_distance: 50.0,
                ..DistanceAttenuation::default()
            },
        }
    }
}

impl SpatialSoundEffectSettings {
    /// Creates a new [`SpatialBasicNode`] using these settings.
    fn create_node(&self) -> SpatialBasicNode {
        SpatialBasicNode {
            distance_attenuation: self.distance_attenuation,
            ..SpatialBasicNode::default()
        }
    }
}

#[cfg_attr(feature = "instrument", tracing::instrument(skip_all))]
fn on_sound_effect_player_added<SoundEffectKeyT>(
    add: On<Add, SoundEffectPlayer<SoundEffectKeyT>>,
    mut commands: Commands,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    sample_rate: Res<SampleRate>,
    sound_config: Res<SoundConfig>,
    packet_assets: Res<Assets<PacketAsset>>,
    packets: Res<SoundEffectPacketAssetCollection>,
    query: Query<&SoundEffectPlayer<SoundEffectKeyT>>,
) where
    SoundEffectKeyT: SoundEffectKeyRequirements,
{
    let Ok(player) = query.get(add.entity) else {
        error!("Sound effect player not found");
        return;
    };

    let (packet_path, sound_effect_id) = (
        player.key.get_packet_path(),
        player.key.get_sound_effect_id(),
    );

    let _span = info_span!("", packet_path, sound_effect_id).entered();

    debug!("Playing sound effect");

    let Some(packet_handle) = packets.get(&packet_path) else {
        error!("Packet does not exist");
        return;
    };
    let Some(packet) = packet_assets.get(packet_handle.id()) else {
        error!("Packet asset not loaded");
        return;
    };
    let Some(sound_effect) = packet.sound_effect(sound_effect_id) else {
        error!("Sound effect does not exist");
        return;
    };

    match sound_effect.typ {
        SfxType::RandomLooping => play_random_looping_sound_effect(
            commands.reborrow(),
            &mut rng,
            &sample_rate,
            &sound_config,
            packet_handle,
            packet,
            sound_effect,
            add.entity,
            false,
            None,
        ),
        _ => panic!("Sound effect type not supported yet"),
    }
}

/// Data needed to spawn a sound effect player child.
struct SoundPlayerData {
    sample_player: SamplePlayer,
    playback_settings: PlaybackSettings,
}

/// Loads a sound and returns the data needed to spawn a player, or `None` if
/// loading fails.
fn load_sound(
    rng: &mut impl Rng,
    sample_rate: &Res<SampleRate>,
    sound_config: &Res<SoundConfig>,
    packet: &PacketAsset,
    sound: &Sound,
) -> Option<SoundPlayerData> {
    let source = packet.audio_sample_handle(sound).or_else(|| {
        error!("Audio sample handle does not exist");
        None
    })?;

    let (sample_player, playback_settings) =
        sound.audio_sample_settings(rng, sound_config, sample_rate.get(), &source);

    Some(SoundPlayerData {
        sample_player,
        playback_settings,
    })
}

#[cfg_attr(feature = "instrument", tracing::instrument(skip_all))]
fn play_random_looping_sound_effect(
    mut commands: Commands,
    rng: &mut impl Rng,
    sample_rate: &Res<SampleRate>,
    sound_config: &Res<SoundConfig>,
    packet_handle: Handle<PacketAsset>,
    packet: &PacketAsset,
    sound_effect: &Sfx,
    entity: Entity,
    spatial: bool,
    spatial_settings: Option<&SpatialSoundEffectSettings>,
) {
    if sound_effect.sounds.is_empty() {
        debug!("Sound effect has no sounds");
        return;
    }

    let Some(sound) = sound_effect.random_sound(rng) else {
        error!("Random sound does not exist");
        return;
    };
    let Some(data) = load_sound(rng, sample_rate, sound_config, packet, &sound) else {
        return;
    };

    let playback_settings = data.playback_settings.with_on_complete(OnComplete::Remove);

    commands.entity(entity).try_insert((
        #[cfg(feature = "entity_names")]
        Name::new(format!(
            "Random looping sound effect {} - {}",
            packet.display_name(),
            sound.file_stem,
        )),
        PacketAssetHandle(packet_handle),
        SoundEffectId(sound_effect.id),
        RandomLoopingSoundEffect,
    ));

    spawn_random_looping_sound_player_child(
        &mut commands,
        entity,
        #[cfg(feature = "entity_names")]
        sound.file_stem.clone(),
        data.sample_player,
        playback_settings,
        spatial,
        spatial_settings,
    );
}

fn random_looping_sound_player_bundle(
    #[cfg(feature = "entity_names")] name: String,
    sample_player: SamplePlayer,
    playback_settings: PlaybackSettings,
) -> impl Bundle {
    (
        #[cfg(feature = "entity_names")]
        Name::new(name),
        RandomLoopingSoundPlayerMarker,
        sample_player,
        playback_settings,
    )
}

/// Spawns a random looping sound player child, optionally with spatial audio.
fn spawn_random_looping_sound_player_child(
    commands: &mut Commands,
    parent: Entity,
    #[cfg(feature = "entity_names")] name: String,
    sample_player: SamplePlayer,
    playback_settings: PlaybackSettings,
    spatial: bool,
    spatial_settings: Option<&SpatialSoundEffectSettings>,
) {
    if spatial {
        let spatial_node = spatial_settings
            .map(|s| s.create_node())
            .unwrap_or_default();
        commands.entity(parent).with_children(|parent| {
            parent.spawn((
                #[cfg(feature = "entity_names")]
                Name::new(name),
                RandomLoopingSoundPlayerMarker,
                sample_player,
                playback_settings,
                sample_effects![(
                    #[cfg(feature = "entity_names")]
                    Name::new("Spatial basic node"),
                    spatial_node,
                )],
                Transform::default(),
            ));
        });
    } else {
        commands.entity(parent).with_children(|parent| {
            parent.spawn(random_looping_sound_player_bundle(
                #[cfg(feature = "entity_names")]
                name,
                sample_player,
                playback_settings,
            ));
        });
    }
}

/// Marker component for the child entity that plays random looping sounds.
#[derive(Component)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Component))]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
struct RandomLoopingSoundPlayerMarker;

/// When the random looping sound player's [`SamplePlayer`] is removed (sound
/// finished), spawn a new child with a new random sound.
#[cfg_attr(feature = "instrument", tracing::instrument(skip_all))]
fn on_random_looping_sound_player_removed(
    remove: On<Remove, SamplePlayer>,
    mut commands: Commands,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    sample_rate: Res<SampleRate>,
    sound_config: Res<SoundConfig>,
    spatial_settings: Res<SpatialSoundEffectSettings>,
    packet_assets: Res<Assets<PacketAsset>>,
    child_of_query: Query<&ChildOf, With<RandomLoopingSoundPlayerMarker>>,
    parent_query: Query<
        (&PacketAssetHandle, &SoundEffectId, Has<SpatialSoundEffect>),
        With<RandomLoopingSoundEffect>,
    >,
) {
    // Check if the removed entity is a random looping sound player child (at
    // the least, it needs a parent to be so)
    let Ok(child_of) = child_of_query.get(remove.entity) else {
        return;
    };

    let parent_entity = child_of.parent();

    // Get the parent's packet and sound effect info.
    let Ok((packet_handle, sound_effect_id, is_spatial)) = parent_query.get(parent_entity) else {
        // Parent might have been despawned, that's fine.
        return;
    };

    let _span = info_span!("", sound_effect_id = sound_effect_id.0).entered();

    let Some(packet) = packet_assets.get(packet_handle.id()) else {
        error!("Sound effect packet asset not loaded");
        return;
    };
    let Some(sound_effect) = packet.sound_effect(sound_effect_id.0) else {
        error!("Sound effect does not exist");
        return;
    };
    let Some(sound) = sound_effect.random_sound(rng.as_mut()) else {
        error!("Random sound does not exist");
        return;
    };
    let Some(data) = load_sound(&mut *rng, &sample_rate, &sound_config, packet, &sound) else {
        return;
    };

    let playback_settings = data.playback_settings.with_on_complete(OnComplete::Remove);

    commands.entity(remove.entity).try_despawn();
    spawn_random_looping_sound_player_child(
        &mut commands,
        parent_entity,
        #[cfg(feature = "entity_names")]
        sound.file_stem.clone(),
        data.sample_player,
        playback_settings,
        is_spatial,
        Some(&*spatial_settings),
    );

    debug!(sound = sound.file_stem, "Playing next random looping sound");
}

#[cfg_attr(feature = "instrument", tracing::instrument(skip_all))]
fn on_spatial_sound_effect_added(
    add: On<Add, SpatialSoundEffect>,
    mut commands: Commands,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    sample_rate: Res<SampleRate>,
    sound_config: Res<SoundConfig>,
    spatial_settings: Res<SpatialSoundEffectSettings>,
    packet_assets: Res<Assets<PacketAsset>>,
    query: Query<&SpatialSoundEffect>,
) {
    let Ok(spatial_sound_effect) = query.get(add.entity) else {
        error!("Spatial sound effect not found");
        return;
    };

    let _span = info_span!(
        "",
        packet_path = ?spatial_sound_effect.packet.path(),
        sound_effect_id = spatial_sound_effect.id,
    )
    .entered();

    debug!("Playing spatial sound effect");

    let packet_handle = spatial_sound_effect.packet.clone();
    let Some(packet) = packet_assets.get(packet_handle.id()) else {
        error!("Sound effect packet asset not loaded");
        return;
    };
    let Some(sound_effect) = packet.sound_effect(spatial_sound_effect.id) else {
        error!("Sound effect does not exist");
        return;
    };

    match sound_effect.typ {
        SfxType::RandomLooping => play_random_looping_sound_effect(
            commands.reborrow(),
            &mut rng,
            &sample_rate,
            &sound_config,
            packet_handle,
            packet,
            sound_effect,
            add.entity,
            true,
            Some(&*spatial_settings),
        ),
        _ => panic!("Sound effect type not supported yet"),
    }
}
