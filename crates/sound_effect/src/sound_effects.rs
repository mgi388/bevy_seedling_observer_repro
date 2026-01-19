#[cfg(feature = "reflect")]
use bevy_reflect::prelude::*;

use crate::{SoundEffectKey as SoundEffectKeyTrait, SoundEffectKeyRequirements};

macro_rules! define_sound_effect_enum {
    ($($name:ident, $packet:expr, $id:expr),*) => {
        #[derive(Clone, Eq, Hash, PartialEq)]
        #[cfg_attr(feature = "debug", derive(Debug))]
        #[cfg_attr(feature = "reflect", derive(Reflect), reflect(Hash, PartialEq))]
        #[cfg_attr(all(feature = "reflect", feature = "debug"), reflect(Debug))]
        pub enum SoundEffectKey {
            $($name),*,
            Custom { packet_path: String, sound_effect_id: u8 }, // for dynamic values
        }

        impl SoundEffectKeyRequirements for SoundEffectKey {}

        impl Default for SoundEffectKey {
            fn default() -> Self {
                SoundEffectKey::ButtonPressed
            }
        }

        impl SoundEffectKeyTrait for SoundEffectKey {
            fn get_packet_path(&self) -> String {
                match self {
                    $(SoundEffectKey::$name => $packet.to_string()),*,
                    SoundEffectKey::Custom { packet_path, .. } => packet_path.clone(),
                }
            }

            fn get_sound_effect_id(&self) -> u8 {
                match self {
                    $(SoundEffectKey::$name => $id),*,
                    SoundEffectKey::Custom { sound_effect_id, .. } => *sound_effect_id,
                }
            }
        }

        impl SoundEffectKey {
            pub fn custom(packet_path: String, sound_effect_id: u8) -> Self {
                SoundEffectKey::Custom { packet_path, sound_effect_id }
            }
        }
    };
}

define_sound_effect_enum! {
    SanguineLoop, "DARKOMEN/SOUND/H/BATALL.H", 3,
    SteamWhistleCool, "DARKOMEN/SOUND/H/BATALL.H", 31,
    BattleAllies, "DARKOMEN/SOUND/H/BATALL.H", 34,

    FireArrow, "DARKOMEN/SOUND/H/BATGEN.H", 3,
    LandArrow, "DARKOMEN/SOUND/H/BATGEN.H", 4,
    CastFireball, "DARKOMEN/SOUND/H/BATGEN.H", 5,
    HornUrgok, "DARKOMEN/SOUND/H/BATGEN.H", 10,
    DispelMagic, "DARKOMEN/SOUND/H/BATGEN.H", 12,
    PlopSplish, "DARKOMEN/SOUND/H/BATGEN.H", 14,

    BattleUndead, "DARKOMEN/SOUND/H/BATUND.H", 3,
    BladeWindHit, "DARKOMEN/SOUND/H/BATUND.H", 10,
    RaiseDead, "DARKOMEN/SOUND/H/BATUND.H", 11,

    DaKrunchSplat, "DARKOMEN/SOUND/H/BATWARGH.H", 11,
    BattleWargh, "DARKOMEN/SOUND/H/BATWARGH.H", 12,
    SpiderCharge, "DARKOMEN/SOUND/H/BATWARGH.H", 18,
    ScorpCharge, "DARKOMEN/SOUND/H/BATWARGH.H", 20,

    MrDrippy, "DARKOMEN/SOUND/H/CAVERN.H", 0,

    FireWorks, "DARKOMEN/SOUND/H/FIREWORK.H", 0,

    MapJourney, "DARKOMEN/SOUND/H/GLUE.H", 0,
    BuyArmor, "DARKOMEN/SOUND/H/GLUE.H", 1,
    NextPage, "DARKOMEN/SOUND/H/GLUE.H", 2,
    BuyBlokes, "DARKOMEN/SOUND/H/GLUE.H", 3,
    BuyMagic, "DARKOMEN/SOUND/H/GLUE.H", 4,

    ButtonPressed, "DARKOMEN/SOUND/H/INTAFACE.H", 0, // used when pressing a button, i.e., pressed event
    ButtonReleased, "DARKOMEN/SOUND/H/INTAFACE.H", 1, // used when releasing a button, i.e., released event
    ButtonDisabled, "DARKOMEN/SOUND/H/INTAFACE.H", 2, // TODO: Rename, this is a "disabled" press sound
    WindsOfMagic, "DARKOMEN/SOUND/H/INTAFACE.H", 3,
    SelectRegiment, "DARKOMEN/SOUND/H/INTAFACE.H", 4, // used when clicking on a regiment banner in a battle
    ButtonAppear, "DARKOMEN/SOUND/H/INTAFACE.H", 5, // used for main menu button hover and toggle magic items panel
    InterfaceSfx6, "DARKOMEN/SOUND/H/INTAFACE.H", 6, // TODO: Rename, sounds like a "sword" click sound
    ReloadTick, "DARKOMEN/SOUND/H/INTAFACE.H", 7, // used for the "reload" dial in the HUD, also used for main menu button pressed event
    Money, "DARKOMEN/SOUND/H/INTAFACE.H", 8,
    NotDone, "DARKOMEN/SOUND/H/INTAFACE.H", 9,
    InterfaceSfx10, "DARKOMEN/SOUND/H/INTAFACE.H", 10, // TODO: Rename, sounds a bit like a high pitch piano key or ping

    SoundOfNight, "DARKOMEN/SOUND/H/NIGHT.H", 0,

    TwitterLoop, "DARKOMEN/SOUND/H/TWITTER.H", 0
}
