use core::num::NonZeroU32;

use bevy_asset::prelude::*;
use bevy_seedling::prelude::*;
use darkomen::sound::sfx::Sound;
use rand::Rng;
use sound_config::prelude::*;

pub trait SoundExt {
    fn audio_sample_settings(
        &self,
        rng: &mut impl Rng,
        sound_config: &SoundConfig,
        sample_rate: NonZeroU32,
        source: &Handle<AudioSample>,
    ) -> (SamplePlayer, PlaybackSettings);
}

impl SoundExt for Sound {
    fn audio_sample_settings(
        &self,
        rng: &mut impl Rng,
        sound_config: &SoundConfig,
        sample_rate: NonZeroU32,
        source: &Handle<AudioSample>,
    ) -> (SamplePlayer, PlaybackSettings) {
        let mut player = SamplePlayer::new(source.clone()).with_volume(Volume::Decibels(
            sound_config
                .sound_effect_volume(Volume::Linear(self.linear_volume()).decibels())
                .0,
        ));

        if self.looped {
            player = player.looping();
        }

        (
            player,
            PlaybackSettings::default()
                .with_speed(self.random_playback_rate(rng, sample_rate) as f64),
        )
    }
}
