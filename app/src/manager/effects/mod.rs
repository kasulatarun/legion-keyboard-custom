use default_ui::{show_brightness, show_direction, show_effect_settings};
use eframe::egui::{self, ComboBox, Slider};
use strum::IntoEnumIterator;

use crate::{
    enums::{Effects, SwipeMode},
    manager::profile::Profile,
};

pub mod aesthetic;
pub mod ambient;

pub mod audio;
pub mod custom;
pub mod cyber;
pub mod default_ui;
pub mod heartbeat;
pub mod lightning;
pub mod nature;
pub mod prism_shift;
pub mod ripple;
pub mod swipe;
pub mod vhs;
pub mod zones;
pub mod synth;
pub mod matrix;
pub mod glitch;

pub fn show_effect_ui(ui: &mut egui::Ui, profile: &mut Profile, update_lights: &mut bool, theme: &crate::gui::style::Theme) {
    let mut effect = profile.effect;

    match &mut effect {
        Effects::SmoothWave { mode, clean_with_black } => {
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = theme.spacing.default;

                show_brightness(ui, profile, update_lights);
                show_direction(ui, profile, update_lights);
                show_effect_settings(ui, profile, update_lights);
                ComboBox::from_label("Swipe mode").width(30.0).selected_text(format!("{:?}", mode)).show_ui(ui, |ui| {
                    for swipe_mode in SwipeMode::iter() {
                        *update_lights |= ui.selectable_value(mode, swipe_mode, format!("{:?}", swipe_mode)).changed();
                    }
                });
                *update_lights |= ui.add_enabled(matches!(mode, SwipeMode::Fill), egui::Checkbox::new(clean_with_black, "Clean with black")).changed();
            });
        }
        Effects::AudioVisualizer { sensitivity, random_colors } => {
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = theme.spacing.default;
                show_brightness(ui, profile, update_lights);
                ui.horizontal(|ui| {
                    *update_lights |= ui.add(Slider::new(sensitivity, 0.0..=100.0)).changed();
                    ui.label("Sensitivity");
                });
                *update_lights |= ui.checkbox(random_colors, "Randomize colors").changed();
            });
        }
        Effects::PrismShift { speed }
        | Effects::LightLeak { speed }
        | Effects::VHSRetro { speed }
        | Effects::NeonDream { speed }
        | Effects::SummerRain { speed }
        | Effects::AuroraBorealis { speed }
        | Effects::CyberPulse { speed }
        | Effects::StarryNight { speed }
        | Effects::SoftBloom { speed }
        | Effects::SunsetGlow { speed }
        | Effects::Synthwave { speed }
        | Effects::Matrix { speed }
        | Effects::Glitch { speed } => {
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = theme.spacing.default;
                show_brightness(ui, profile, update_lights);
                ui.horizontal(|ui| {
                    let s = 1..=10;
                    *update_lights |= ui.add(egui::Slider::new(speed, s)).changed();
                    ui.label("Speed / Intensity");
                });
            });
        }
        Effects::AmbientLight { fps, vibrance } => {
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = theme.spacing.default;
                show_brightness(ui, profile, update_lights);
                ui.horizontal(|ui| {
                    *update_lights |= ui.add(egui::Slider::new(fps, 1..=90)).changed();
                    ui.label("Ambient FPS");
                });
                ui.horizontal(|ui| {
                    *update_lights |= ui.add(egui::Slider::new(vibrance, 0..=255)).changed();
                    ui.label("Vibrance (Saturation)");
                });
            });
        }
        Effects::Candlelight => {
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = theme.spacing.default;
                show_brightness(ui, profile, update_lights);
            });
        }
        _ => {
            default_ui::show(ui, profile, update_lights, &theme.spacing);
        }
    }

    profile.effect = effect;
}
