use crate::manager::{custom_effect::CustomEffect, profile::Profile};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AutomationRule {
    pub title_contains: String,
    pub profile_name: String,
}

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Display, EnumIter, Debug, IntoStaticStr, Default)]
pub enum Effects {
    NeonDream {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    #[default]
    Lightning,
    SmoothWave {
        mode: SwipeMode,
        clean_with_black: bool,
    },
    Ripple,
    RippleLit,
    AudioVisualizer {
        #[serde(default = "default_sensitivity")]
        sensitivity: f32,
        #[serde(default = "default_random_colors")]
        random_colors: bool,
    },
    Fire,
    OceanWave,
    Meteor,
    AmbientLight {
        #[serde(default = "default_fps")]
        fps: u8,
        #[serde(default = "default_vibrance")]
        vibrance: u8,
    },
    Heartbeat,
    PrismShift {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    LightLeak {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    VHSRetro {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    SummerRain {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    AuroraBorealis {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    Candlelight,
    CyberPulse {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    StarryNight {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    SoftBloom {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    SunsetGlow {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    Synthwave {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    Matrix {
        #[serde(default = "default_speed")]
        speed: u8,
    },
    Glitch {
        #[serde(default = "default_speed")]
        speed: u8,
    },
}

impl Effects {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Lightning => "Random, crackling flashes of white and blue.",
            Self::SmoothWave { .. } => "A high-fidelity, smooth wave of colors.",
            Self::Ripple => "Responsive! Pressed zones light up and fade smoothly.",
            Self::RippleLit => "Responsive! Base color stays, pressed zones show secondary color.",
            Self::AudioVisualizer { .. } => "Reacts to the overall volume of your system audio.",
            Self::Fire => "Simulates flickering flames (Red/Orange/Yellow).",
            Self::OceanWave => "Simulates calming waves (Deep Blue/Cyan).",
            Self::Meteor => "A moving comet that bounces across the keyboard.",
            Self::AmbientLight { .. } => "Immersive: Matches keyboard colors to your screen.",
            Self::Heartbeat => "The Pulse: A rhythmic heartbeat using center-zone colors.",
            Self::PrismShift { .. } => "Improved dynamic effect: flowing rainbow prism with smooth blending.",
            Self::LightLeak { .. } => "Cinematic: Warm, cinematic amber glows moving smoothly.",
            Self::VHSRetro { .. } => "Retro: Enhanced glitch with color separation and scanlines.",
            Self::NeonDream { .. } => "Vibrant: Oscillating Pink, Purple, and Cyan gradients.",
            Self::SummerRain { .. } => "Nature: Organic, random blue pulses that fade like raindrops.",
            Self::AuroraBorealis { .. } => "Nature: Flowing green and purple waves mimicking the northern lights.",
            Self::Candlelight => "Nature: Realistic, irregular warm flickering like a candle.",
            Self::CyberPulse { .. } => "Action: Fast-paced Cyan and Magenta rhythms.",
            Self::StarryNight { .. } => "Nature: Twinkling dots on a dark background.",
            Self::SoftBloom { .. } => "Subtle: Breathing intensity across zones.",
            Self::SunsetGlow { .. } => "Vibrant: Slow-shifting deep reds and oranges.",
            Self::Synthwave { .. } => "Retro: Aesthetic pink and cyan waves.",
            Self::Matrix { .. } => "Hacker: Raining digital green.",
            Self::Glitch { .. } => "Cyber: Random chaotic flickers.",
        }
    }
}

fn default_speed() -> u8 {
    1
}



fn default_sensitivity() -> f32 {
    1.0
}

fn default_fps() -> u8 {
    60
}

fn default_vibrance() -> u8 {
    100
}

fn default_random_colors() -> bool {
    true
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, EnumIter, EnumString, PartialEq)]
pub enum SwipeMode {
    #[default]
    Change,
    Fill,
}

impl PartialEq for Effects {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[allow(dead_code)]
impl Effects {
    pub fn takes_color_array(self) -> bool {
        matches!(
            self,
            Self::Lightning
                | Self::Ripple
                | Self::RippleLit
                | Self::AudioVisualizer { .. }
                | Self::Fire
                | Self::OceanWave
                | Self::Meteor
                | Self::AmbientLight { .. }
                | Self::Heartbeat
                | Self::PrismShift { .. }
                | Self::LightLeak { .. }
                | Self::VHSRetro { .. }
                | Self::NeonDream { .. }
                | Self::SummerRain { .. }
                | Self::AuroraBorealis { .. }
                | Self::Candlelight
                | Self::CyberPulse { .. }
                | Self::StarryNight { .. }
                | Self::SoftBloom { .. }
                | Self::SunsetGlow { .. }
                | Self::Synthwave { .. }
                | Self::Matrix { .. }
                | Self::Glitch { .. }
        )
    }

    pub fn takes_direction(self) -> bool {
        matches!(self, Self::SmoothWave { .. })
    }

    pub fn takes_speed(self) -> bool {
        matches!(
            self,
            Self::Lightning
                | Self::SmoothWave { .. }
                | Self::Ripple
                | Self::RippleLit
                | Self::AudioVisualizer { .. }
                | Self::Fire
                | Self::OceanWave
                | Self::Meteor
                | Self::AmbientLight { .. }
                | Self::PrismShift { .. }
                | Self::LightLeak { .. }
                | Self::VHSRetro { .. }
                | Self::NeonDream { .. }
                | Self::SummerRain { .. }
                | Self::AuroraBorealis { .. }
                | Self::CyberPulse { .. }
                | Self::StarryNight { .. }
                | Self::SoftBloom { .. }
                | Self::SunsetGlow { .. }
                | Self::Synthwave { .. }
                | Self::Matrix { .. }
                | Self::Glitch { .. }
        )
    }

    pub fn is_built_in(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, EnumString, Serialize, Deserialize, Debug, EnumIter, IntoStaticStr, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Left,
    Right,
}

#[derive(PartialEq, Eq, EnumIter, IntoStaticStr, Clone, Copy, Default, Serialize, Deserialize, Debug, Display, EnumString)]
pub enum Brightness {
    #[default]
    Low,
    High,
}

#[derive(Debug)]
pub enum Message {
    CustomEffect { effect: CustomEffect },
    Profile { profile: Profile },
    AutoProfile { name: String },
    #[allow(dead_code)]
    UpdateAutomationRules { rules: Vec<AutomationRule> },
    #[allow(dead_code)]
    UpdateMasterPower { off: bool },
    Exit,
}
