#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use hidapi::{HidApi, HidDevice};
use once_cell::sync::Lazy;
use rand::Rng;
use rdev::{listen, Event, EventType, Key};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tauri::{Manager, State};
use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

const NUM_LOGICAL_ZONES: usize = 24;
const NUM_PHYSICAL_ZONES: usize = 4;
const LEGACY_PIDS: [u16; 7] = [0xc993, 0xc996, 0xc995, 0xc994, 0xc985, 0xc981, 0xc693];
const SAFE_METHODS: [WriteMethod; 6] = [
    WriteMethod::FeatureReport(0xCC),
    WriteMethod::FeatureReport(0x07),
    WriteMethod::FeatureReport(0x00),
    WriteMethod::OutputReport(0x01),
    WriteMethod::OutputReport(0x00),
    WriteMethod::OutputReport(0xCC),
];

static KEY_EVENTS: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn black() -> Self {
        Self::new(0, 0, 0)
    }

    fn white() -> Self {
        Self::new(255, 255, 255)
    }

    fn scale(self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        Self::new(
            (self.r as f32 * amount).round().clamp(0.0, 255.0) as u8,
            (self.g as f32 * amount).round().clamp(0.0, 255.0) as u8,
            (self.b as f32 * amount).round().clamp(0.0, 255.0) as u8,
        )
    }

    fn perceptual_scale(self, brightness: f32) -> Self {
        let multiplier = if brightness <= 0.0 {
            0.0
        } else {
            brightness.clamp(0.0, 1.0).powf(1.0 / 2.2)
        };
        self.scale(multiplier)
    }

    fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
        )
    }

    fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = (h.rem_euclid(360.0)) / 60.0;
        let i = h.floor() as i32 % 6;
        let f = h - h.floor();
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);
        let (r, g, b) = match i {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        Self::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WriteMethod {
    FeatureReport(u8),
    OutputReport(u8),
}

impl WriteMethod {
    fn report_id(self) -> u8 {
        match self {
            WriteMethod::FeatureReport(id) | WriteMethod::OutputReport(id) => id,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ParameterConfig {
    pub name: String,
    pub label: String,
    pub param_type: ParameterType,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub step: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ParameterType {
    Float,
    #[serde(rename = "Color")]
    Color { r: u8, g: u8, b: u8 },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PresetMetadata {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub parameters: Vec<ParameterConfig>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "value")]
pub enum ParameterValue {
    Float(f32),
    Color { r: u8, g: u8, b: u8 },
}

struct OutputTarget {
    device: HidDevice,
    method: WriteMethod,
    label: String,
}

pub struct LedController {
    outputs: Vec<OutputTarget>,
    interface_info: String,
    brightness: f32,
    frame_buffer: [RgbColor; NUM_LOGICAL_ZONES],
    ui_frame: Arc<Mutex<Vec<RgbColor>>>,
}

impl LedController {
    fn new(ui_frame: Arc<Mutex<Vec<RgbColor>>>) -> Self {
        Self {
            outputs: Vec::new(),
            interface_info: String::new(),
            brightness: 1.0,
            frame_buffer: [RgbColor::black(); NUM_LOGICAL_ZONES],
            ui_frame,
        }
    }

    fn connect(&mut self) -> Result<(), String> {
        match find_outputs() {
            Ok(outputs) if !outputs.is_empty() => {
                self.interface_info = outputs
                    .iter()
                    .map(|output| output.label.clone())
                    .collect::<Vec<_>>()
                    .join(" | ");
                self.outputs = outputs;
                Ok(())
            }
            _ => {
                self.outputs.clear();
                self.interface_info.clear();
                Err("No compatible 4-zone Legion/LOQ interface found".to_string())
            }
        }
    }

    fn is_connected(&self) -> bool {
        !self.outputs.is_empty()
    }

    fn interface_label(&self) -> String {
        if self.interface_info.is_empty() {
            "No device".to_string()
        } else {
            self.interface_info.clone()
        }
    }

    fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness.clamp(0.0, 1.0);
    }

    fn set_physical_zones(&mut self, zones: [RgbColor; NUM_PHYSICAL_ZONES]) {
        let span = NUM_LOGICAL_ZONES / NUM_PHYSICAL_ZONES;
        for (physical_index, color) in zones.into_iter().enumerate() {
            let start = physical_index * span;
            let end = start + span;
            for logical_index in start..end {
                self.frame_buffer[logical_index] = color;
            }
        }
    }

    fn fill(&mut self, color: RgbColor) {
        self.frame_buffer = [color; NUM_LOGICAL_ZONES];
    }

    fn get_frame_vec(&self) -> Vec<RgbColor> {
        self.frame_buffer
            .iter()
            .map(|color| color.perceptual_scale(self.brightness))
            .collect()
    }

    fn flush_buffered(&mut self) -> Result<(), String> {
        if self.outputs.is_empty() {
            return Err("Keyboard not connected".to_string());
        }
        let physical = downsample_to_four(&self.frame_buffer, self.brightness);
        let mut success_count = 0usize;

        for output in &self.outputs {
            if send_legacy_colors(&output.device, output.method, &physical, self.brightness).is_ok() {
                success_count += 1;
            }
        }

        if success_count == 0 {
            return Err("No connected lighting endpoints accepted the frame".to_string());
        }

        *self.ui_frame.lock().unwrap() = self.get_frame_vec();
        Ok(())
    }

    fn sync_preview_only(&self) {
        *self.ui_frame.lock().unwrap() = self.get_frame_vec();
    }

    fn clear(&mut self) {
        self.fill(RgbColor::black());
        if self.flush_buffered().is_err() {
            self.sync_preview_only();
        }
    }
}

trait Effect: Send {
    fn start(&mut self, _controller: &mut LedController) {}
    fn update(&mut self, controller: &mut LedController, time: f32, delta: f32);
    fn stop(&mut self, controller: &mut LedController) {
        controller.clear();
    }
}

fn start_key_listener() {
    thread::spawn(|| {
        if let Err(error) = listen(handle_key_event) {
            eprintln!("key listener failed: {:?}", error);
        }
    });
}

fn handle_key_event(event: Event) {
    if let EventType::KeyPress(key) = event.event_type {
        let zone = map_key_to_zone(key);
        if let Ok(mut events) = KEY_EVENTS.lock() {
            events.push(zone);
            if events.len() > 24 {
                let excess = events.len() - 24;
                events.drain(0..excess);
            }
        }
    }
}

fn map_key_to_zone(key: Key) -> usize {
    match key {
        Key::Escape
        | Key::BackQuote
        | Key::Tab
        | Key::CapsLock
        | Key::ShiftLeft
        | Key::ControlLeft
        | Key::Num1
        | Key::KeyQ
        | Key::KeyA
        | Key::KeyZ
        | Key::MetaLeft => 0,
        Key::Num2
        | Key::Num3
        | Key::Num4
        | Key::KeyW
        | Key::KeyE
        | Key::KeyR
        | Key::KeyS
        | Key::KeyD
        | Key::KeyF
        | Key::KeyX
        | Key::KeyC
        | Key::KeyV
        | Key::Alt
        | Key::Space => 1,
        Key::Num5
        | Key::Num6
        | Key::Num7
        | Key::KeyT
        | Key::KeyY
        | Key::KeyU
        | Key::KeyG
        | Key::KeyH
        | Key::KeyJ
        | Key::KeyB
        | Key::KeyN
        | Key::KeyM
        | Key::Comma
        | Key::AltGr => 2,
        _ => 3,
    }
}

struct StaticEffect {
    color: RgbColor,
}

impl Effect for StaticEffect {
    fn start(&mut self, controller: &mut LedController) {
        controller.set_physical_zones([self.color; NUM_PHYSICAL_ZONES]);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }

    fn update(&mut self, _controller: &mut LedController, _time: f32, _delta: f32) {}
}

struct BreathingEffect {
    color: RgbColor,
    speed: f32,
}

impl Effect for BreathingEffect {
    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        let glow = ((time * self.speed * std::f32::consts::TAU).sin() + 1.0) * 0.5;
        controller.set_physical_zones([self.color.scale(0.18 + glow * 0.82); NUM_PHYSICAL_ZONES]);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct PulseEffect {
    color: RgbColor,
    speed: f32,
}

impl Effect for PulseEffect {
    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        let pulse = ((time * self.speed * std::f32::consts::TAU).sin() + 1.0) * 0.5;
        let edge = self.color.scale(0.08 + pulse * 0.28);
        let center = self.color.scale(0.2 + pulse * 0.8);
        controller.set_physical_zones([edge, center, center, edge]);

        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

#[derive(Clone, Copy)]
struct ZonePulse {
    origin: usize,
    started_at: f32,
}

struct ReactiveRippleEffect {
    base_color: RgbColor,
    accent_color: RgbColor,
    speed: f32,
    pulses: Vec<ZonePulse>,
}

impl ReactiveRippleEffect {
    fn new(base_color: RgbColor, accent_color: RgbColor, speed: f32) -> Self {
        Self {
            base_color,
            accent_color,
            speed,
            pulses: Vec::new(),
        }
    }
}

impl Effect for ReactiveRippleEffect {
    fn start(&mut self, _controller: &mut LedController) {
        self.pulses.clear();
    }

    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        if let Ok(mut events) = KEY_EVENTS.lock() {
            for zone in events.drain(..) {
                self.pulses.push(ZonePulse {
                    origin: zone.min(NUM_PHYSICAL_ZONES - 1),
                    started_at: time,
                });
            }
        }

        self.pulses.retain(|pulse| time - pulse.started_at < 1.1);

        let zones = std::array::from_fn(|zone| {
            let mut color = self.base_color.scale(0.22);
            for pulse in &self.pulses {
                let age = time - pulse.started_at;
                let wave_pos = age * self.speed * 1.8;
                let dist = (zone as f32 - pulse.origin as f32).abs();
                let band = (1.0 - (dist - wave_pos).abs()).clamp(0.0, 1.0).powf(1.6);
                let core = (1.0 - dist * 0.85).clamp(0.0, 1.0) * (1.0 - age / 1.1).clamp(0.0, 1.0);
                let strength = band.max(core);
                color = color.lerp(self.accent_color, strength);
            }
            color
        });

        controller.set_physical_zones(zones);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct ReactiveChainEffect {
    base_color: RgbColor,
    accent_color: RgbColor,
    speed: f32,
    pulses: Vec<ZonePulse>,
}

impl ReactiveChainEffect {
    fn new(base_color: RgbColor, accent_color: RgbColor, speed: f32) -> Self {
        Self {
            base_color,
            accent_color,
            speed,
            pulses: Vec::new(),
        }
    }
}

impl Effect for ReactiveChainEffect {
    fn start(&mut self, _controller: &mut LedController) {
        self.pulses.clear();
    }

    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        if let Ok(mut events) = KEY_EVENTS.lock() {
            for zone in events.drain(..) {
                self.pulses.push(ZonePulse {
                    origin: zone.min(NUM_PHYSICAL_ZONES - 1),
                    started_at: time,
                });
            }
        }

        self.pulses.retain(|pulse| time - pulse.started_at < 1.25);

        let zones = std::array::from_fn(|zone| {
            let mut color = self.base_color.scale(0.2);
            for pulse in &self.pulses {
                let age = time - pulse.started_at;
                let head = pulse.origin as f32 + age * self.speed * if pulse.origin < 2 { 1.0 } else { -1.0 };
                let dist = (zone as f32 - head).abs();
                let strength = (1.0 - dist / 1.1).clamp(0.0, 1.0).powf(1.8)
                    * (1.0 - age / 1.25).clamp(0.0, 1.0);
                color = color.lerp(self.accent_color, strength);
            }
            color
        });

        controller.set_physical_zones(zones);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct RainbowCycleEffect {
    speed: f32,
    saturation: f32,
}

impl Effect for RainbowCycleEffect {
    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        let hue = (time * self.speed * 70.0).rem_euclid(360.0);
        let color = RgbColor::from_hsv(hue, self.saturation, 1.0);
        controller.set_physical_zones([color; NUM_PHYSICAL_ZONES]);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct RainbowWaveEffect {
    speed: f32,
}

impl Effect for RainbowWaveEffect {
    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        let zones = std::array::from_fn(|zone| {
            let pos = zone as f32 / NUM_PHYSICAL_ZONES as f32;
            let hue = ((pos * 360.0) + time * self.speed * 110.0).rem_euclid(360.0);
            RgbColor::from_hsv(hue, 0.95, 1.0)
        });
        controller.set_physical_zones(zones);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct AuroraEffect {
    speed: f32,
    intensity: f32,
}

impl Effect for AuroraEffect {
    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        let zones = std::array::from_fn(|zone| {
            let pos = zone as f32 / NUM_PHYSICAL_ZONES as f32;
            let flow = (pos + time * self.speed * 0.15).fract();
            let brightness = 0.18 + (1.0 - smoothstep(0.0, 0.55, (flow - 0.5).abs())) * self.intensity;
            let hue = 155.0 + flow * 110.0 + (time * self.speed * 8.0).sin() * 8.0;
            RgbColor::from_hsv(hue, 0.82, brightness.clamp(0.0, 1.0))
        });
        controller.set_physical_zones(zones);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct FireFlowEffect {
    speed: f32,
    intensity: f32,
}

impl Effect for FireFlowEffect {
    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        let mut rng = rand::thread_rng();
        let zones = std::array::from_fn(|zone| {
            let pos = zone as f32 / NUM_PHYSICAL_ZONES as f32;
            let wave = ((time * self.speed * 2.8) + pos * 4.5).sin() * 0.5 + 0.5;
            let flicker: f32 = rng.gen_range(0.82..1.0);
            let heat = (0.25 + wave * self.intensity).clamp(0.0, 1.0) * flicker;
            let hue = 12.0 + heat * 28.0;
            let value = (0.15 + heat * 0.95).clamp(0.0, 1.0);
            RgbColor::from_hsv(hue, 1.0, value)
        });
        controller.set_physical_zones(zones);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct OceanWaveEffect {
    speed: f32,
    depth: f32,
}

impl Effect for OceanWaveEffect {
    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        let zones = std::array::from_fn(|zone| {
            let pos = zone as f32 / NUM_PHYSICAL_ZONES as f32;
            let wave_a = ((pos * 9.0) + time * self.speed * 2.2).sin() * 0.5 + 0.5;
            let wave_b = ((pos * 14.0) - time * self.speed * 1.6).sin() * 0.5 + 0.5;
            let blend = ((wave_a * 0.65) + (wave_b * 0.35)).clamp(0.0, 1.0);
            let hue = 192.0 + blend * 34.0;
            let value = (0.18 + blend * self.depth).clamp(0.0, 1.0);
            RgbColor::from_hsv(hue, 0.78, value)
        });
        controller.set_physical_zones(zones);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct StillGradientEffect {
    color_a: RgbColor,
    color_b: RgbColor,
    midpoint: f32,
}

impl Effect for StillGradientEffect {
    fn start(&mut self, controller: &mut LedController) {
        let midpoint = (self.midpoint / (NUM_LOGICAL_ZONES - 1) as f32).clamp(0.0, 1.0);
        let zones = std::array::from_fn(|zone| {
            let pos = zone as f32 / (NUM_PHYSICAL_ZONES - 1) as f32;
            let t = if pos <= midpoint {
                if midpoint <= 0.0 { 0.0 } else { (pos / midpoint) * 0.5 }
            } else {
                let span = (1.0 - midpoint).max(0.001);
                0.5 + ((pos - midpoint) / span) * 0.5
            };
            self.color_a.lerp(self.color_b, t.clamp(0.0, 1.0))
        });
        controller.set_physical_zones(zones);

        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }

    fn update(&mut self, _controller: &mut LedController, _time: f32, _delta: f32) {}
}

struct SparkleEffect {
    density: f32,
    color: RgbColor,
}

impl Effect for SparkleEffect {
    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        let mut rng = rand::thread_rng();
        let base = RgbColor::new(4, 4, 8);
        let zones = std::array::from_fn(|zone| {
            let shimmer = (((zone as f32 * 0.9) + time * 1.4).sin() * 0.5 + 0.5) * 0.12;
            let mut color = base.lerp(self.color, shimmer);
            if rng.gen_bool(self.density.clamp(0.0, 1.0) as f64) {
                let boost: f32 = rng.gen_range(0.55..1.0);
                color = self.color.scale(boost);
            }
            color
        });
        controller.set_physical_zones(zones);
        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct MeteorEffect {
    color: RgbColor,
    speed: f32,
}

impl Effect for MeteorEffect {
    fn update(&mut self, controller: &mut LedController, time: f32, _delta: f32) {
        let head = (time * self.speed * 2.5) % (NUM_PHYSICAL_ZONES as f32 + 2.0) - 1.0;
        let zones = std::array::from_fn(|zone| {
            let distance = (zone as f32 - head).abs();
            let strength = (1.0 - distance / 1.6).clamp(0.0, 1.0).powf(2.0);
            self.color.scale(strength)
        });
        controller.set_physical_zones(zones);

        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct FerrariRpmEffect {
    pos: f32,
    direction: f32,
    rpm: f32,
    shift_flash: u8,
    backfire: u8,
}

impl FerrariRpmEffect {
    fn new() -> Self {
        Self {
            pos: -6.0,
            direction: 1.0,
            rpm: 0.24,
            shift_flash: 0,
            backfire: 0,
        }
    }
}

impl Effect for FerrariRpmEffect {
    fn start(&mut self, _controller: &mut LedController) {
        self.pos = -6.0;
        self.direction = 1.0;
        self.rpm = 0.24;
        self.shift_flash = 0;
        self.backfire = 0;
    }

    fn update(&mut self, controller: &mut LedController, time: f32, delta: f32) {
        let mut rng = rand::thread_rng();
        let throttle = (((time * 2.0).sin() + 1.0) * 0.5).powf(1.25);
        let target_rpm = 0.22 + throttle * 0.58;
        self.rpm += (target_rpm - self.rpm) * 0.05;

        if self.rpm >= 0.98 {
            self.rpm -= 0.12;
            self.shift_flash = 2;
            self.backfire = rng.gen_range(1..=3);
        }

        self.pos += self.direction * self.rpm * delta * 72.0;

        if self.pos > NUM_LOGICAL_ZONES as f32 + 5.0 {
            self.direction = -1.0;
            self.backfire = 2;
        } else if self.pos < -5.0 {
            self.direction = 1.0;
            self.backfire = 2;
        }

        let mut zones = [RgbColor::black(); NUM_PHYSICAL_ZONES];

        for zone in 0..NUM_PHYSICAL_ZONES {
            let distance = (zone as f32 - self.pos).abs();
            if distance < 2.5 {
                let heat = (1.0 - distance / 2.5).clamp(0.0, 1.0).powf(1.5);
                let hue = 6.0 + heat * 36.0;
                let value = (0.2 + heat * (0.7 + self.rpm * 0.45)).min(1.0);
                zones[zone] = RgbColor::from_hsv(hue, 1.0, value);
            }
        }

        if self.shift_flash > 0 {
            let center = self.pos.round() as i32;
            for offset in -1..=1 {
                let index = center + offset;
                if (0..NUM_PHYSICAL_ZONES as i32).contains(&index) {
                    zones[index as usize] = RgbColor::white();
                }
            }
            self.shift_flash -= 1;
        }

        if self.backfire > 0 {
            let tail = (self.pos - 2.0 * self.direction).round() as i32;
            if (0..NUM_PHYSICAL_ZONES as i32).contains(&tail) {
                let color = match rng.gen_range(0..3) {
                    0 => RgbColor::new(255, 132, 44),
                    1 => RgbColor::new(255, 186, 96),
                    _ => RgbColor::white(),
                };
                zones[tail as usize] = color;
            }
            self.backfire -= 1;
        }

        controller.set_physical_zones(zones);

        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

struct ThermalProEffect {
    system: System,
    components: Components,
    nvml: Option<Nvml>,
    smoothing: f32,
    cpu_color: RgbColor,
    gpu_color: RgbColor,
    cpu_level: f32,
    gpu_level: f32,
}

impl ThermalProEffect {
    fn new(smoothing: f32, cpu_color: RgbColor, gpu_color: RgbColor) -> Self {
        Self {
            system: System::new_with_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            ),
            components: Components::new_with_refreshed_list(),
            nvml: Nvml::init().ok(),
            smoothing,
            cpu_color,
            gpu_color,
            cpu_level: 0.0,
            gpu_level: 0.0,
        }
    }
}

impl Effect for ThermalProEffect {
    fn update(&mut self, controller: &mut LedController, _time: f32, _delta: f32) {
        self.system.refresh_cpu_all();
        self.components.refresh(false);

        let cpu_usage = self.system.global_cpu_usage() / 100.0;
        let cpu_temp = self
            .components
            .iter()
            .find(|component| component.label().to_ascii_lowercase().contains("cpu"))
            .map(|component| (component.temperature().unwrap_or(45.0) / 100.0).clamp(0.0, 1.0))
            .unwrap_or(cpu_usage);

        let gpu_temp = self
            .nvml
            .as_ref()
            .and_then(|nvml| nvml.device_by_index(0).ok())
            .and_then(|device| device.temperature(TemperatureSensor::Gpu).ok())
            .map(|temp| (temp as f32 / 100.0).clamp(0.0, 1.0))
            .unwrap_or_else(|| {
                self.system.used_memory() as f32 / self.system.total_memory().max(1) as f32
            });

        let alpha = (self.smoothing / 10.0).clamp(0.05, 0.6);

        self.cpu_level += (cpu_temp - self.cpu_level) * alpha;
        self.gpu_level += (gpu_temp - self.gpu_level) * alpha;

        let cpu_left = self.cpu_color.scale(0.18 + self.cpu_level * 0.82);
        let cpu_right = self.cpu_color.scale(0.1 + self.cpu_level * 0.56);
        let gpu_left = self.gpu_color.scale(0.1 + self.gpu_level * 0.56);
        let gpu_right = self.gpu_color.scale(0.18 + self.gpu_level * 0.82);
        let zones = [cpu_left, cpu_right, gpu_left, gpu_right];
        controller.set_physical_zones(zones);

        if controller.flush_buffered().is_err() {
            controller.sync_preview_only();
        }
    }
}

pub struct AppState {
    controller: Mutex<LedController>,
    ui_frame: Arc<Mutex<Vec<RgbColor>>>,
    effect_running: Arc<AtomicBool>,
    current_effect: Mutex<Option<Box<dyn Effect>>>,
    current_preset_name: Mutex<String>,
    current_params: Mutex<HashMap<String, ParameterValue>>,
}

fn send_legacy_colors(
    device: &HidDevice,
    method: WriteMethod,
    zones: &[RgbColor; NUM_PHYSICAL_ZONES],
    brightness: f32,
) -> Result<(), String> {
    send_legacy_packet(device, method, 0x01, 0x01, hardware_brightness_byte(brightness), zones)
}

fn hardware_brightness_byte(brightness: f32) -> u8 {
    if brightness >= 0.5 { 0x01 } else { 0x02 }
}

fn send_legacy_packet(
    device: &HidDevice,
    method: WriteMethod,
    mode: u8,
    speed: u8,
    brightness: u8,
    zones: &[RgbColor; NUM_PHYSICAL_ZONES],
) -> Result<(), String> {
    let mut buf = [0u8; 33];
    buf[0] = method.report_id();
    buf[1] = 0x16;
    buf[2] = mode;
    buf[3] = speed;
    buf[4] = brightness;

    for (index, color) in zones.iter().enumerate() {
        let base = 5 + index * 3;
        buf[base] = color.r;
        buf[base + 1] = color.g;
        buf[base + 2] = color.b;
    }

    match method {
        WriteMethod::FeatureReport(_) => device
            .send_feature_report(&buf)
            .map(|_| ())
            .map_err(|err| err.to_string()),
        WriteMethod::OutputReport(_) => device
            .write(&buf)
            .map(|_| ())
            .map_err(|err| err.to_string()),
    }
}

fn downsample_to_four(
    logical: &[RgbColor; NUM_LOGICAL_ZONES],
    brightness: f32,
) -> [RgbColor; NUM_PHYSICAL_ZONES] {
    std::array::from_fn(|zone| {
        let start = zone * (NUM_LOGICAL_ZONES / NUM_PHYSICAL_ZONES);
        let end = start + (NUM_LOGICAL_ZONES / NUM_PHYSICAL_ZONES);
        let mut r = 0u32;
        let mut g = 0u32;
        let mut b = 0u32;

        for color in &logical[start..end] {
            let scaled = color.perceptual_scale(brightness);
            r += scaled.r as u32;
            g += scaled.g as u32;
            b += scaled.b as u32;
        }

        let count = (end - start) as u32;
        RgbColor::new((r / count) as u8, (g / count) as u8, (b / count) as u8)
    })
}

fn is_lenovo_candidate(candidate: &hidapi::DeviceInfo) -> bool {
    candidate.vendor_id() == 0x048d && LEGACY_PIDS.contains(&candidate.product_id())
}

fn candidate_label(candidate: &hidapi::DeviceInfo) -> String {
    format!(
        "PID 0x{:04x} IF {} USAGE 0x{:04x}",
        candidate.product_id(),
        candidate.interface_number(),
        candidate.usage_page()
    )
}

fn find_outputs() -> Result<Vec<OutputTarget>, String> {
    let api = HidApi::new().map_err(|err| err.to_string())?;
    let mut outputs = Vec::new();
    let probe_zones = [RgbColor::black(); NUM_PHYSICAL_ZONES];

    for candidate in api.device_list() {
        if is_lenovo_candidate(candidate) {
            if let Ok(device) = candidate.open_device(&api) {
                for method in SAFE_METHODS {
                    if send_legacy_colors(&device, method, &probe_zones, 1.0).is_ok() {
                        outputs.push(OutputTarget {
                            device,
                            method,
                            label: format!("{} RID 0x{:02X}", candidate_label(candidate), method.report_id()),
                        });
                        break;
                    }
                }
            }
        }
    }

    Ok(outputs)
}

fn build_presets() -> Vec<PresetMetadata> {
    vec![
        PresetMetadata {
            name: "staticColor".to_string(),
            display_name: "Static Color".to_string(),
            description: "Clean single-color fill across the full logical underglow preview.".to_string(),
            parameters: vec![color_param("color", "Color", 0, 229, 255)],
        },
        PresetMetadata {
            name: "aurora".to_string(),
            display_name: "Aurora".to_string(),
            description: "Flowing cold-spectrum ribbons that still translate nicely to 4 physical zones.".to_string(),
            parameters: vec![
                float_param("speed", "Speed", 0.2, 3.0, 0.9, 0.1),
                float_param("intensity", "Intensity", 0.2, 1.0, 0.75, 0.05),
            ],
        },
        PresetMetadata {
            name: "rpm".to_string(),
            display_name: "Ferrari RPM".to_string(),
            description: "A fast-moving performance sweep with white shift flashes and warm tail pops.".to_string(),
            parameters: vec![],
        },
        PresetMetadata {
            name: "thermalPro".to_string(),
            display_name: "Thermal Pro".to_string(),
            description: "Zones 1-2 visualize CPU heat and zones 3-4 visualize GPU heat with separate colors.".to_string(),
            parameters: vec![
                float_param("smoothing", "Smoothing", 1.0, 10.0, 5.0, 0.5),
                color_param("cpu_color", "CPU Color", 255, 96, 48),
                color_param("gpu_color", "GPU Color", 0, 229, 255),
            ],
        },
        PresetMetadata {
            name: "breathing".to_string(),
            display_name: "Breathing".to_string(),
            description: "A slow studio-style pulse for one chosen color.".to_string(),
            parameters: vec![
                float_param("speed", "Speed", 0.2, 4.0, 1.0, 0.1),
                color_param("color", "Color", 255, 255, 255),
            ],
        },
        PresetMetadata {
            name: "pulse".to_string(),
            display_name: "Pulse Center".to_string(),
            description: "Focused center pulse that stays readable after 24-to-4 averaging.".to_string(),
            parameters: vec![
                float_param("speed", "Speed", 0.2, 4.0, 1.2, 0.1),
                color_param("color", "Color", 255, 72, 72),
            ],
        },
        PresetMetadata {
            name: "ripple".to_string(),
            display_name: "Typing Ripple".to_string(),
            description: "Base color stays on, and the accent color blooms in the zone you type with a smooth ripple.".to_string(),
            parameters: vec![
                float_param("speed", "Speed", 0.6, 3.0, 1.4, 0.1),
                color_param("base_color", "Base Color", 255, 32, 32),
                color_param("accent_color", "Effect Color", 0, 229, 255),
            ],
        },
        PresetMetadata {
            name: "chain".to_string(),
            display_name: "Typing Chain".to_string(),
            description: "A key press launches the accent color as a short chain sweep from that zone over a base backlight.".to_string(),
            parameters: vec![
                float_param("speed", "Speed", 0.6, 3.4, 1.6, 0.1),
                color_param("base_color", "Base Color", 24, 24, 28),
                color_param("accent_color", "Effect Color", 80, 255, 140),
            ],
        },
        PresetMetadata {
            name: "meteor".to_string(),
            display_name: "Meteor".to_string(),
            description: "A sharp moving head and tail effect that reads clearly even after 24-to-4 downsampling.".to_string(),
            parameters: vec![
                float_param("speed", "Speed", 0.5, 5.0, 1.8, 0.1),
                color_param("color", "Accent", 255, 96, 48),
            ],
        },
        PresetMetadata {
            name: "rainbowCycle".to_string(),
            display_name: "Rainbow Cycle".to_string(),
            description: "Whole keyboard shifts hue together with a clean studio-style color wheel.".to_string(),
            parameters: vec![
                float_param("speed", "Speed", 0.2, 4.0, 1.0, 0.1),
                float_param("saturation", "Saturation", 0.2, 1.0, 0.95, 0.05),
            ],
        },
        PresetMetadata {
            name: "rainbowWave".to_string(),
            display_name: "Rainbow Wave".to_string(),
            description: "A left-to-right rainbow motion rendered logically across all 24 zones.".to_string(),
            parameters: vec![float_param("speed", "Speed", 0.2, 4.0, 1.1, 0.1)],
        },
        PresetMetadata {
            name: "stillGradient".to_string(),
            display_name: "Still Gradient".to_string(),
            description: "Static two-color blend across the preview for a calmer desk-light look.".to_string(),
            parameters: vec![
                color_param("color_a", "Color A", 0, 229, 255),
                color_param("color_b", "Color B", 255, 84, 112),
                float_param("middle", "Midpoint", 0.0, 23.0, 11.5, 0.5),
            ],
        },
        PresetMetadata {
            name: "fireFlow".to_string(),
            display_name: "Fire Flow".to_string(),
            description: "Warm animated embers and heat shimmer tuned for a 4-zone keyboard.".to_string(),
            parameters: vec![
                float_param("speed", "Speed", 0.2, 4.0, 1.0, 0.1),
                float_param("intensity", "Intensity", 0.2, 1.0, 0.75, 0.05),
            ],
        },
        PresetMetadata {
            name: "ocean".to_string(),
            display_name: "Ocean Wave".to_string(),
            description: "Layered blue-green tides moving across the logical underglow strip.".to_string(),
            parameters: vec![
                float_param("speed", "Speed", 0.2, 4.0, 1.0, 0.1),
                float_param("depth", "Depth", 0.2, 1.0, 0.72, 0.05),
            ],
        },
        PresetMetadata {
            name: "sparkle".to_string(),
            display_name: "Sparkle".to_string(),
            description: "Low-intensity sparkle accents over a dark base without hardware-native glitches.".to_string(),
            parameters: vec![
                float_param("density", "Density", 0.02, 0.35, 0.08, 0.01),
                color_param("color", "Accent", 255, 255, 255),
            ],
        },
    ]
}

fn float_param(name: &str, label: &str, min: f32, max: f32, default: f32, step: f32) -> ParameterConfig {
    ParameterConfig {
        name: name.to_string(),
        label: label.to_string(),
        param_type: ParameterType::Float,
        min,
        max,
        default,
        step,
    }
}

fn color_param(name: &str, label: &str, r: u8, g: u8, b: u8) -> ParameterConfig {
    ParameterConfig {
        name: name.to_string(),
        label: label.to_string(),
        param_type: ParameterType::Color { r, g, b },
        min: 0.0,
        max: 255.0,
        default: 0.0,
        step: 1.0,
    }
}

fn get_float(parameters: &HashMap<String, ParameterValue>, name: &str, fallback: f32) -> f32 {
    match parameters.get(name) {
        Some(ParameterValue::Float(value)) => *value,
        _ => fallback,
    }
}

fn get_color(parameters: &HashMap<String, ParameterValue>, name: &str, fallback: RgbColor) -> RgbColor {
    match parameters.get(name) {
        Some(ParameterValue::Color { r, g, b }) => RgbColor::new(*r, *g, *b),
        _ => fallback,
    }
}

fn create_effect(
    preset_name: &str,
    parameters: &HashMap<String, ParameterValue>,
) -> Result<Box<dyn Effect>, String> {
    match preset_name.to_lowercase().as_str() {
        "staticcolor" => Ok(Box::new(StaticEffect {
            color: get_color(parameters, "color", RgbColor::new(0, 229, 255)),
        })),
        "aurora" => Ok(Box::new(AuroraEffect {
            speed: get_float(parameters, "speed", 0.9),
            intensity: get_float(parameters, "intensity", 0.75),
        })),
        "rpm" => Ok(Box::new(FerrariRpmEffect::new())),
        "thermalpro" => Ok(Box::new(ThermalProEffect::new(
            get_float(parameters, "smoothing", 5.0),
            get_color(parameters, "cpu_color", RgbColor::new(255, 96, 48)),
            get_color(parameters, "gpu_color", RgbColor::new(0, 229, 255)),
        ))),
        "breathing" => Ok(Box::new(BreathingEffect {
            color: get_color(parameters, "color", RgbColor::white()),
            speed: get_float(parameters, "speed", 1.0),
        })),
        "pulse" => Ok(Box::new(PulseEffect {
            color: get_color(parameters, "color", RgbColor::new(255, 72, 72)),
            speed: get_float(parameters, "speed", 1.2),
        })),
        "ripple" => Ok(Box::new(ReactiveRippleEffect::new(
            get_color(parameters, "base_color", RgbColor::new(255, 32, 32)),
            get_color(parameters, "accent_color", RgbColor::new(0, 229, 255)),
            get_float(parameters, "speed", 1.4),
        ))),
        "chain" => Ok(Box::new(ReactiveChainEffect::new(
            get_color(parameters, "base_color", RgbColor::new(24, 24, 28)),
            get_color(parameters, "accent_color", RgbColor::new(80, 255, 140)),
            get_float(parameters, "speed", 1.6),
        ))),
        "meteor" => Ok(Box::new(MeteorEffect {
            color: get_color(parameters, "color", RgbColor::new(255, 96, 48)),
            speed: get_float(parameters, "speed", 1.8),
        })),
        "rainbowcycle" => Ok(Box::new(RainbowCycleEffect {
            speed: get_float(parameters, "speed", 1.0),
            saturation: get_float(parameters, "saturation", 0.95),
        })),
        "rainbowwave" => Ok(Box::new(RainbowWaveEffect {
            speed: get_float(parameters, "speed", 1.1),
        })),
        "stillgradient" => Ok(Box::new(StillGradientEffect {
            color_a: get_color(parameters, "color_a", RgbColor::new(0, 229, 255)),
            color_b: get_color(parameters, "color_b", RgbColor::new(255, 84, 112)),
            midpoint: get_float(parameters, "middle", 11.5),
        })),
        "fireflow" => Ok(Box::new(FireFlowEffect {
            speed: get_float(parameters, "speed", 1.0),
            intensity: get_float(parameters, "intensity", 0.75),
        })),
        "ocean" => Ok(Box::new(OceanWaveEffect {
            speed: get_float(parameters, "speed", 1.0),
            depth: get_float(parameters, "depth", 0.72),
        })),
        "sparkle" => Ok(Box::new(SparkleEffect {
            density: get_float(parameters, "density", 0.08),
            color: get_color(parameters, "color", RgbColor::white()),
        })),
        _ => Err(format!("Unknown preset: {preset_name}")),
    }
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[tauri::command]
fn get_connection_status(state: State<AppState>) -> bool {
    state.controller.lock().unwrap().is_connected()
}

#[tauri::command]
fn get_interface_info(state: State<AppState>) -> String {
    state.controller.lock().unwrap().interface_label()
}

#[tauri::command]
fn reconnect(state: State<AppState>) -> Result<String, String> {
    let mut controller = state.controller.lock().unwrap();
    controller.connect()?;
    controller.flush_buffered()?;
    Ok(format!("Connected via {}", controller.interface_label()))
}

#[tauri::command]
fn hardware_test(state: State<AppState>, color: RgbColor) -> Result<String, String> {
    let mut controller = state.controller.lock().unwrap();
    if !controller.is_connected() {
        controller.connect()?;
    }
    controller.set_physical_zones([color; NUM_PHYSICAL_ZONES]);
    controller.flush_buffered()?;
    Ok(format!(
        "Hardware test sent: #{:02x}{:02x}{:02x}",
        color.r, color.g, color.b
    ))
}

#[tauri::command]
fn get_frame(state: State<AppState>) -> Vec<RgbColor> {
    state.ui_frame.lock().unwrap().clone()
}

#[tauri::command]
fn get_preset_metadata() -> Vec<PresetMetadata> {
    build_presets()
}

#[tauri::command]
fn set_brightness(brightness: f32, state: State<AppState>) -> Result<String, String> {
    let mut controller = state.controller.lock().unwrap();
    controller.set_brightness(brightness);
    if controller.is_connected() {
        controller.flush_buffered()?;
        Ok("Brightness updated on keyboard".to_string())
    } else {
        controller.sync_preview_only();
        Ok("Brightness updated in preview only".to_string())
    }
}

#[tauri::command]
fn set_preset(
    preset_name: String,
    parameters: HashMap<String, ParameterValue>,
    state: State<AppState>,
) -> Result<String, String> {
    let mut next_effect = create_effect(&preset_name, &parameters)?;

    {
        let mut controller = state.controller.lock().unwrap();
        let mut current_effect = state.current_effect.lock().unwrap();
        if let Some(mut effect) = current_effect.take() {
            effect.stop(&mut controller);
        }
        next_effect.start(&mut controller);
        *current_effect = Some(next_effect);
    }

    *state.current_preset_name.lock().unwrap() = preset_name.clone();
    *state.current_params.lock().unwrap() = parameters;

    Ok(format!("Loaded preset: {preset_name}"))
}

#[tauri::command]
fn adjust_preset_parameter(
    preset_name: String,
    param_name: String,
    value: ParameterValue,
    state: State<AppState>,
) -> Result<(), String> {
    let mut parameters = state.current_params.lock().unwrap().clone();
    parameters.insert(param_name, value);
    set_preset(preset_name, parameters, state).map(|_| ())
}

fn run_effect_loop(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        let start = Instant::now();
        let mut last_frame = Instant::now();

        loop {
            if !app_handle.state::<AppState>().effect_running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(40));
                continue;
            }

            {
                let state = app_handle.state::<AppState>();
                let mut current_effect = state.current_effect.lock().unwrap();
                if let Some(effect) = current_effect.as_mut() {
                    let now = Instant::now();
                    let time = (now - start).as_secs_f32();
                    let delta = (now - last_frame).as_secs_f32();
                    last_frame = now;

                    let mut controller = state.controller.lock().unwrap();
                    effect.update(&mut controller, time, delta);
                }
            }

            {
                let frame = app_handle.state::<AppState>().ui_frame.lock().unwrap().clone();
                let _ = app_handle.emit_all("new-colors", frame);
            }

            thread::sleep(Duration::from_millis(40));
        }
    });
}

fn main() {
    let ui_frame = Arc::new(Mutex::new(vec![RgbColor::black(); NUM_LOGICAL_ZONES]));
    let mut controller = LedController::new(ui_frame.clone());
    let _ = controller.connect();
    controller.sync_preview_only();
    start_key_listener();

    tauri::Builder::default()
        .manage(AppState {
            controller: Mutex::new(controller),
            ui_frame: ui_frame.clone(),
            effect_running: Arc::new(AtomicBool::new(true)),
            current_effect: Mutex::new(None),
            current_preset_name: Mutex::new(String::new()),
            current_params: Mutex::new(HashMap::new()),
        })
        .setup(|app| {
            run_effect_loop(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            adjust_preset_parameter,
            get_connection_status,
            get_frame,
            get_interface_info,
            get_preset_metadata,
            hardware_test,
            reconnect,
            set_brightness,
            set_preset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
