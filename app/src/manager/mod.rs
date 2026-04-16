use crate::{
    enums::{AutomationRule, Effects, Message},
    gui::GuiMessage,
};

use crossbeam_channel::{Receiver, Sender};
use device_query::Keycode;
use device_query::{DeviceEvents, DeviceEventsHandler};
use effects::{
    aesthetic, ambient, audio, custom, cyber, heartbeat, lightning, nature, prism_shift, ripple, swipe, vhs,
};
use error_stack::{Result, ResultExt};
use legion_rgb_driver::{BaseEffects, Keyboard, SPEED_RANGE};
use profile::Profile;
use rand::{rng, rngs::ThreadRng};
use single_instance::SingleInstance;

use std::sync::Arc;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
use thiserror::Error;

use self::custom_effect::{CustomEffect, EffectType};

pub mod custom_effect;
pub mod effects;
pub mod profile;

pub use effects::show_effect_ui;

#[derive(Debug, Error, PartialEq)]
#[error("Could not create keyboard manager")]
pub enum ManagerCreationError {
    #[error("There was an error getting a valid keyboard")]
    AcquireKeyboard,
    #[error("An instance of the program is already running")]
    InstanceAlreadyRunning,
}

pub struct EffectManager {
    tx: Sender<Message>,
    pub handle: Option<thread::JoinHandle<()>>,
    _stop_signals: StopSignals,
}

pub(crate) struct Inner {
    keyboard: Keyboard,
    rx: Receiver<Message>,
    key_rx: Receiver<Keycode>,
    gui_tx: Option<Sender<GuiMessage>>,
    stop_signals: StopSignals,
    pub profiles: Vec<Profile>,
    pub master_off: bool,
    pub automation_rules: Vec<AutomationRule>,
    _single_instance: SingleInstance,

    // Animation States
    ripple_state: Vec<ActiveRipple>,
    heartbeat_start: Option<Instant>,
}

#[derive(Clone, Copy)]
pub struct ActiveRipple {
    pub origin_zone: usize,
    pub start_time: Instant,
}

#[derive(Clone, Copy)]
pub enum OperationMode {
    Cli,
    Gui,
}

impl EffectManager {
    pub fn new(
        operation_mode: OperationMode, profiles: Vec<Profile>, automation_rules: Vec<AutomationRule>, master_off: bool, gui_tx: Option<Sender<GuiMessage>>,
    ) -> Result<Self, ManagerCreationError> {
        let stop_signals = StopSignals {
            manager_stop_signal: Arc::new(AtomicBool::new(false)),
            keyboard_stop_signal: Arc::new(AtomicBool::new(false)),
        };

        let single_instance = SingleInstance::new(env!("CARGO_PKG_NAME")).unwrap();
        if !single_instance.is_single() {
            return Err(ManagerCreationError::InstanceAlreadyRunning.into());
        }

        let keyboard = legion_rgb_driver::get_keyboard(stop_signals.keyboard_stop_signal.clone()).change_context(ManagerCreationError::AcquireKeyboard)?;

        let (tx, rx) = crossbeam_channel::unbounded::<Message>();
        let (key_tx, key_rx) = crossbeam_channel::unbounded::<Keycode>();

        // --- Global Keyboard Event Listener ---
        thread::spawn(move || {
            let event_handler = DeviceEventsHandler::new(Duration::from_millis(10)).unwrap_or(DeviceEventsHandler {});
            let _press_guard = event_handler.on_key_down(move |key| {
                let _ = key_tx.send(*key);
            });
            loop {
                thread::sleep(Duration::from_millis(50));
            }
        });

        let mut inner = Inner {
            keyboard,
            rx,
            key_rx,
            gui_tx,
            stop_signals: stop_signals.clone(),
            profiles,
            master_off,
            automation_rules,
            _single_instance: single_instance,
            ripple_state: Vec::new(),
            heartbeat_start: None,
        };

        // --- Automation Monitor Thread ---
        let tx_monitor = tx.clone();
        let stop_signals_monitor = stop_signals.clone();
        let mon_rules = inner.automation_rules.clone();
        thread::spawn(move || {
            use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW};
            let mut last_title = String::new();
            let current_rules = mon_rules;

            while !stop_signals_monitor.manager_stop_signal.load(Ordering::SeqCst) {
                unsafe {
                    let hwnd = GetForegroundWindow();
                    if !hwnd.is_null() {
                        let mut buffer: [u16; 512] = [0; 512];
                        let len = GetWindowTextW(hwnd, buffer.as_mut_ptr(), 512);
                        if len > 0 {
                            let title = String::from_utf16_lossy(&buffer[..len as usize]);
                            if title != last_title {
                                for rule in &current_rules {
                                    if title.contains(&rule.title_contains) {
                                        let _ = tx_monitor.try_send(Message::AutoProfile { name: rule.profile_name.clone() });
                                        break;
                                    }
                                }
                                last_title = title;
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_millis(2000));
            }
        });

        let handle = thread::spawn(move || {
            let mut rng = rng();
            let mut current_profile = inner.profiles.get(0).cloned().unwrap_or_default();

            // --- INERTIAL INITIALIZATION ---
            // Ensure hardware is in the correct mode immediately
            inner.set_profile(current_profile.clone(), &mut rng);

            loop {
                // 1. Process latest GUI message (Non-blocking)
                let msg = match operation_mode {
                    OperationMode::Cli => inner.rx.recv().ok(),
                    OperationMode::Gui => inner.rx.try_iter().last(),
                };

                if let Some(message) = msg {
                    match message {
                        Message::Profile { profile } => {
                            current_profile = profile;
                            inner.set_profile(current_profile.clone(), &mut rng);
                        }
                        Message::AutoProfile { name } => {
                            if let Some(p) = inner.profiles.iter().find(|p| p.name.as_deref() == Some(&name)) {
                                current_profile = p.clone();
                                inner.set_profile(current_profile.clone(), &mut rng);
                            }
                        }
                        Message::UpdateAutomationRules { rules } => {
                            inner.automation_rules = rules;
                        }
                        Message::UpdateMasterPower { off } => {
                            inner.master_off = off;
                        }
                        Message::CustomEffect { effect } => {
                            inner.custom_effect(&effect);
                        }
                        Message::Exit => break,
                    }
                }

                // 2. Tick current effect (Non-blocking)
                if !inner.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                    inner.apply_effect(&mut current_profile, &mut rng);
                }

                // 3. Constant frame rate (33ms/30fps for stability and hardware sync)
                thread::sleep(Duration::from_millis(33));
            }
        });

        Ok(Self {
            tx,
            handle: Some(handle),
            _stop_signals: stop_signals,
        })
    }

    pub fn set_profile(&self, profile: Profile) {
        self._stop_signals.manager_stop_signal.store(true, Ordering::SeqCst);
        let _ = self.tx.send(Message::Profile { profile });
    }

    pub fn custom_effect(&self, effect: CustomEffect) {
        self._stop_signals.manager_stop_signal.store(true, Ordering::SeqCst);
        let _ = self.tx.send(Message::CustomEffect { effect });
    }

    pub fn shutdown(mut self) {
        let _ = self.tx.send(Message::Exit);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Inner {
    pub(crate) fn paint(&mut self, profile: &Profile, colors: &[u8; 12]) {
        if self.master_off {
            let _ = self.keyboard.set_colors_to(&[0u8; 12]);
            return;
        }

        let mut final_arr = *colors;
        let brightness_mult = match profile.brightness {
            crate::enums::Brightness::Low => 0.5,  // 50%
            crate::enums::Brightness::High => 1.0, // 100%
        };

        for i in 0..12 {
            final_arr[i] = (final_arr[i] as f32 * brightness_mult) as u8;
        }

        let _ = self.keyboard.set_colors_to(&final_arr);

        // Stream frame back to GUI for visualizer
        if let Some(tx) = &self.gui_tx {
            let _ = tx.send(GuiMessage::LiveFrame(final_arr));
        }
    }

    fn set_profile(&mut self, profile: Profile, _rng: &mut ThreadRng) {
        // Reset stop signal so the new effect can actually run
        self.stop_signals.manager_stop_signal.store(false, Ordering::SeqCst);
        if profile.effect.is_built_in() {
            let clamped_speed = profile.speed.clamp(SPEED_RANGE.min().unwrap(), SPEED_RANGE.max().unwrap());
            self.keyboard.set_speed(clamped_speed).unwrap();
        } else {
            // Software effects REQUIRE the keyboard to be in Static mode
            // to accept custom RGB buffers.
            self.keyboard.set_effect(BaseEffects::Static).unwrap();
        }
        self.keyboard.set_brightness(profile.brightness as u8 + 1).unwrap();
    }

    fn apply_effect(&mut self, profile: &mut Profile, rng: &mut ThreadRng) {
        if self.master_off {
            self.keyboard.set_colors_to(&[0u8; 12]).unwrap();
            return;
        }

        match profile.effect {
            Effects::Lightning => lightning::play(self, profile, rng),
            Effects::SmoothWave { mode, clean_with_black } => swipe::play(self, profile, mode, clean_with_black),
            Effects::Ripple => ripple::update(self, profile),
            Effects::RippleLit => ripple::update(self, profile),
            Effects::AudioVisualizer { sensitivity, random_colors } => audio::play(self, profile, sensitivity, random_colors),
            Effects::Fire => custom::play_fire(self, profile),
            Effects::OceanWave => custom::play_ocean(self, profile),
            Effects::Meteor => custom::play_meteor(self, profile),
            Effects::AmbientLight { .. } => ambient::play(self, profile),
            Effects::Heartbeat => heartbeat::update(self, profile),
            Effects::PrismShift { .. } => prism_shift::play(self, profile),
            Effects::LightLeak { .. } => aesthetic::play_light_leak(self, profile),
            Effects::VHSRetro { .. } => vhs::play_vhs_retro(self, profile),
            Effects::NeonDream { .. } => aesthetic::play_neon_dream(self, profile),
            Effects::SummerRain { .. } => nature::play_summer_rain(self, profile),
            Effects::AuroraBorealis { .. } => nature::play_aurora_borealis(self, profile),
            Effects::Candlelight => aesthetic::play_candlelight(self, profile),
            Effects::CyberPulse { .. } => cyber::play_cyber_pulse(self, profile),
            Effects::StarryNight { .. } => nature::play_starry_night(self, profile),
            Effects::SoftBloom { .. } => aesthetic::play_soft_bloom(self, profile),
            Effects::SunsetGlow { .. } => aesthetic::play_sunset_glow(self, profile),
        }
    }

    fn custom_effect(&mut self, custom_effect: &CustomEffect) {
        self.stop_signals.store_false();
        loop {
            if self.master_off {
                self.keyboard.set_colors_to(&[0u8; 12]).unwrap();
                thread::sleep(Duration::from_millis(500));
                if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                    return;
                }
                continue;
            }
            for step in &custom_effect.effect_steps {
                self.keyboard.set_brightness(step.brightness).unwrap();
                match step.step_type {
                    EffectType::Set => {
                        self.keyboard.set_colors_to(&step.rgb_array).unwrap();
                    }
                    _ => {
                        self.keyboard.transition_colors_to(&step.rgb_array, step.steps, step.delay_between_steps).unwrap();
                    }
                }
                if self.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
                    return;
                }
                thread::sleep(Duration::from_millis(step.sleep));
            }
            if !custom_effect.should_loop {
                break;
            }
        }
    }
}

impl Drop for EffectManager {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::Exit);
    }
}

#[derive(Clone)]
pub struct StopSignals {
    pub manager_stop_signal: Arc<AtomicBool>,
    pub keyboard_stop_signal: Arc<AtomicBool>,
}

impl StopSignals {
    #[allow(dead_code)]
    pub fn store_true(&self) {
        self.keyboard_stop_signal.store(true, Ordering::SeqCst);
        self.manager_stop_signal.store(true, Ordering::SeqCst);
    }
    pub fn store_false(&self) {
        self.keyboard_stop_signal.store(false, Ordering::SeqCst);
        self.manager_stop_signal.store(false, Ordering::SeqCst);
    }
}
