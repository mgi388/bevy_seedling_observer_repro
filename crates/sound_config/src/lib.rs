use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_kira_audio::prelude::Decibels;
#[cfg(feature = "reflect")]
use bevy_reflect::prelude::*;
use serde::{Deserialize, Serialize};

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{SoundConfig, SoundConfigPlugin};
}

pub struct SoundConfigPlugin;

impl Plugin for SoundConfigPlugin {
    fn build(&self, #[allow(unused_variables)] app: &mut App) {
        #[cfg(feature = "reflect")]
        app.register_type::<SoundConfig>();

        app.insert_resource(SoundConfig::default());
    }
}

#[derive(Clone, Default, Deserialize, Resource, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Resource, Serialize)
)]
#[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
pub struct SoundConfig {
    pub global_volume_decibels: f32,
    original_global_volume_decibels: Option<f32>,
    pub music_volume_decibels: f32,
    pub sound_effect_volume_decibels: f32,
    pub voice_over_volume_decibels: f32,
    pub dialogue_volume_decibels: f32,
}

impl SoundConfig {
    pub fn toggle_mute(&mut self) {
        if self.original_global_volume_decibels.is_some() {
            self.unmute();
        } else {
            self.mute();
        }
    }

    pub fn mute(&mut self) {
        self.original_global_volume_decibels = Some(self.global_volume_decibels);
        self.global_volume_decibels = Decibels::SILENCE.0;
    }

    pub fn unmute(&mut self) {
        let Some(original_global_volume_decibels) = self.original_global_volume_decibels else {
            return;
        };
        self.global_volume_decibels = original_global_volume_decibels;
        self.original_global_volume_decibels = None;
    }

    #[inline(always)]
    pub fn music_volume(&self, base: impl Into<Decibels>) -> Decibels {
        Decibels(self.effective_music_volume().0 + base.into().0)
    }

    #[inline(always)]
    fn effective_music_volume(&self) -> Decibels {
        Decibels(self.effective_music_volume_decibels())
    }

    #[inline(always)]
    fn effective_music_volume_decibels(&self) -> f32 {
        self.effective_volume(self.music_volume_decibels)
    }

    #[inline(always)]
    pub fn sound_effect_volume(&self, base: impl Into<Decibels>) -> Decibels {
        Decibels(self.effective_sound_effect_volume().0 + base.into().0)
    }

    #[inline(always)]
    fn effective_sound_effect_volume(&self) -> Decibels {
        Decibels(self.effective_sound_effect_volume_decibels())
    }

    #[inline(always)]
    fn effective_sound_effect_volume_decibels(&self) -> f32 {
        self.effective_volume(self.sound_effect_volume_decibels)
    }

    #[inline(always)]
    pub fn voice_over_volume(&self, base: impl Into<Decibels>) -> Decibels {
        Decibels(self.effective_voice_over_volume().0 + base.into().0)
    }

    #[inline(always)]
    fn effective_voice_over_volume(&self) -> Decibels {
        Decibels(self.effective_voice_over_volume_decibels())
    }

    #[inline(always)]
    fn effective_voice_over_volume_decibels(&self) -> f32 {
        self.effective_volume(self.voice_over_volume_decibels)
    }

    #[inline(always)]
    pub fn dialogue_volume(&self, base: impl Into<Decibels>) -> Decibels {
        Decibels(self.effective_dialogue_volume().0 + base.into().0)
    }

    #[inline(always)]
    fn effective_dialogue_volume(&self) -> Decibels {
        Decibels(self.effective_dialogue_volume_decibels())
    }

    #[inline(always)]
    fn effective_dialogue_volume_decibels(&self) -> f32 {
        self.effective_volume(self.dialogue_volume_decibels)
    }

    #[inline(always)]
    fn effective_volume(&self, volume_decibels: f32) -> f32 {
        self.global_volume_decibels + volume_decibels
    }
}
