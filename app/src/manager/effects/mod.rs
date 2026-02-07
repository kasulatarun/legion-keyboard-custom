use default_ui::{show_brightness, show_direction, show_effect_settings};
use eframe::egui::{self, ComboBox, Slider};
use strum::IntoEnumIterator;

use crate::{
    enums::{Effects, SwipeMode},
    manager::profile::Profile,
};

pub mod ambient;
pub mod default_ui;
pub mod audio;
pub mod frequency_audio;
pub mod lightning;
pub mod ripple;
pub mod swipe;
pub mod system_monitor;
pub mod wpm;
pub mod custom;
pub mod pomodoro;
pub mod zones;

pub fn show_effect_ui(ui: &mut egui::Ui, profile: &mut Profile, update_lights: &mut bool, theme: &crate::gui::style::Theme) {
    let mut effect = profile.effect;

    match &mut effect {
        Effects::SmoothWave { mode, clean_with_black } | Effects::Swipe { mode, clean_with_black } => {
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
        Effects::AmbientLight { fps, saturation_boost } => {
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = theme.spacing.default;

                show_brightness(ui, profile, update_lights);
                show_direction(ui, profile, update_lights);

                ui.horizontal(|ui| {
                    *update_lights |= ui.add(Slider::new(fps, 1..=60)).changed();
                    ui.label("FPS");
                });
                ui.horizontal(|ui| {
                    *update_lights |= ui.add(Slider::new(saturation_boost, 0.0..=1.0)).changed();
                    ui.label("Saturation Boost");
                });
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
        Effects::Pomodoro { duration_mins } => {
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = theme.spacing.default;
                show_brightness(ui, profile, update_lights);
                ui.horizontal(|ui| {
                    *update_lights |= ui.add(Slider::new(duration_mins, 1..=60)).changed();
                    ui.label("Duration (mins)");
                });
            });
        }
        Effects::FrequencyVisualizer { sensitivity } => {
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = theme.spacing.default;
                show_brightness(ui, profile, update_lights);
                ui.horizontal(|ui| {
                    *update_lights |= ui.add(Slider::new(sensitivity, 0.0..=100.0)).changed();
                    ui.label("Sensitivity");
                });
            });
        }
        _ => {
            default_ui::show(ui, profile, update_lights, &theme.spacing);
        }
    }

    profile.effect = effect;
}
