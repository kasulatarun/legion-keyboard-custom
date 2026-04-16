use error::{RangeError, RangeErrorKind, Result};
use hidapi::{HidApi, HidDevice};
use std::{
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub mod error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolType {
    Legacy, // 33-byte, ID 0xcc
    C2,     // 64-byte, ID 0x01
}

pub const KNOWN_DEVICE_INFOS: [(u16, u16, u16, u16, ProtocolType); 18] = [
    // --- Priority 1: High-Compatibility Legacy Interface (Slow but Reliable) ---
    (0x048d, 0xc993, 0xff89, 0x00cc, ProtocolType::Legacy), // 2024 LOQ (Legacy - 93)
    (0x048d, 0xc996, 0xff89, 0x00cc, ProtocolType::Legacy), // 2024 LOQ (Legacy - 96)
    (0x048d, 0xc995, 0xff89, 0x00cc, ProtocolType::Legacy), // 2024 Pro
    (0x048d, 0xc994, 0xff89, 0x00cc, ProtocolType::Legacy), // 2024
    (0x048d, 0xc985, 0xff89, 0x00cc, ProtocolType::Legacy), // 2023 Pro
    (0x048d, 0xc984, 0xff89, 0x00cc, ProtocolType::Legacy), // 2023
    (0x048d, 0xc983, 0xff89, 0x00cc, ProtocolType::Legacy), // 2023 LOQ
    (0x048d, 0xc975, 0xff89, 0x00cc, ProtocolType::Legacy), // 2022
    (0x048d, 0xc973, 0xff89, 0x00cc, ProtocolType::Legacy), // 2022 Ideapad
    (0x048d, 0xc965, 0xff89, 0x00cc, ProtocolType::Legacy), // 2021
    (0x048d, 0xc963, 0xff89, 0x00cc, ProtocolType::Legacy), // 2021 Ideapad
    (0x048d, 0xc955, 0xff89, 0x00cc, ProtocolType::Legacy), // 2020
    // --- Priority 2: High-Frequency C2 Interface (New 2024 Standard) ---
    (0x048d, 0xc993, 0xffc2, 0x0004, ProtocolType::C2), // 2024 LOQ (C2 - 93)
    (0x048d, 0xc996, 0xffc2, 0x0004, ProtocolType::C2), // 2024 LOQ (C2 - 96)
    // --- Priority 3: Alternatives (0x0010 / 0x0007) ---
    (0x048d, 0xc993, 0xff89, 0x0010, ProtocolType::Legacy), // 2024 LOQ (Alternative)
    (0x048d, 0xc996, 0xff89, 0x0010, ProtocolType::Legacy), // 2024 LOQ (Lighting)
    (0x048d, 0xc993, 0xff89, 0x0007, ProtocolType::Legacy), // 2024 LOQ (Extra)
    (0x048d, 0xc996, 0xff89, 0x0007, ProtocolType::Legacy), // 2024 LOQ (Extra)
];

pub const SPEED_RANGE: std::ops::RangeInclusive<u8> = 1..=4;
pub const BRIGHTNESS_RANGE: std::ops::RangeInclusive<u8> = 1..=2;
pub const ZONE_RANGE: std::ops::RangeInclusive<u8> = 0..=3;

pub enum BaseEffects {
    Static,
    Breath,
    Smooth,
    LeftWave,
    RightWave,
}

pub struct LightingState {
    effect_type: BaseEffects,
    speed: u8,
    brightness: u8,
    rgb_values: [u8; 12],
}

pub struct Keyboard {
    keyboard_hid: HidDevice,
    protocol: ProtocolType,
    current_state: LightingState,
    stop_signal: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl Keyboard {
    fn build_legacy_payload(&self) -> Result<[u8; 33]> {
        let keyboard_state = &self.current_state;

        let mut payload: [u8; 33] = [0; 33];
        payload[0] = 0xcc;
        payload[1] = 0x16;
        payload[2] = match keyboard_state.effect_type {
            BaseEffects::Static => 0x01,
            BaseEffects::Breath => 0x03,
            BaseEffects::Smooth => 0x06,
            BaseEffects::LeftWave => {
                payload[19] = 0x1;
                0x04
            }
            BaseEffects::RightWave => {
                payload[18] = 0x1;
                0x04
            }
        };

        payload[3] = keyboard_state.speed;
        payload[4] = keyboard_state.brightness;

        if let BaseEffects::Static | BaseEffects::Breath = keyboard_state.effect_type {
            payload[5..(12 + 5)].copy_from_slice(&keyboard_state.rgb_values[..12]);
        };

        Ok(payload)
    }

    fn build_c2_payload(&self) -> Result<[u8; 65]> {
        let keyboard_state = &self.current_state;

        let mut payload: [u8; 65] = [0; 65];
        payload[0] = 0x01; // Report ID
        println!("🧪 Building C2 Payload (Header: 0x0b)");
        payload[1] = 0x0b; // Primary Command ID: Set Lighting
        payload[2] = 0x01; // Mode: Standard/Manual
        payload[3] = 0x01; // Action: Apply

        // RGB Zones (Bytes 4-15)
        payload[4..16].copy_from_slice(&keyboard_state.rgb_values[..12]);

        // Some 2024 models use 0x02 as the command header instead
        // We will send both or allow the caller to specify. For now, we use a hybrid approach
        // where we verify the primary and fallback if it fails.
        if let Err(e) = self.keyboard_hid.write(&payload) {
            println!("⚠️ Header 0x0b failed: {}. Retrying with Header 0x02...", e);
            payload[1] = 0x02; // Fallback Command ID: Specific for some 2024 ITEs
            let _ = self.keyboard_hid.write(&payload);
        }

        // Brightness (Byte 32 or 16? Standard C2 is 32)
        payload[32] = match keyboard_state.brightness {
            1 => 50,
            2 => 100,
            _ => 100,
        };

        Ok(payload)
    }

    pub fn refresh(&mut self) -> Result<()> {
        match self.protocol {
            ProtocolType::Legacy => {
                let payload = self.build_legacy_payload()?;
                if let Err(e) = self.keyboard_hid.send_feature_report(&payload) {
                    eprintln!("🔴 HID Feature Report Failed: {}", e);
                    let _ = std::io::stderr().flush();
                }
            }
            ProtocolType::C2 => {
                let payload = self.build_c2_payload()?;
                if let Err(e) = self.keyboard_hid.write(&payload) {
                    // Check for "0x000003E5" (Overlapped I/O operation is in progress)
                    if e.to_string().contains("0x000003E5") || e.to_string().contains("in progress") {
                        // Hardware is busy, wait and retry once
                        std::thread::sleep(std::time::Duration::from_millis(15));
                        if let Err(e2) = self.keyboard_hid.write(&payload) {
                            eprintln!("🔴 HID Write Retry Failed: {}", e2);
                        } else {
                            println!("🟢 HID Recovery Successful");
                        }
                    } else {
                        eprintln!("🔴 HID Write Failed (C2): {}", e);
                    }
                    let _ = std::io::stderr().flush();
                }
            }
        }

        Ok(())
    }

    /// Unlocks standard software control mode with a brute-force sweep of common ITE codes.
    pub fn handshake(&mut self) -> Result<()> {
        if self.protocol == ProtocolType::C2 {
            // We try multiple common 'Unlock' codes in sequence
            // Codes: 0x01 (Standard), 0x02 (Alt), 0x08 (New 2024), 0x04 (Pro)
            let codes = [0x01, 0x08, 0x02, 0x04];

            for code in codes {
                let mut payload: [u8; 65] = [0; 65];
                payload[0] = 0x01; // Report ID
                payload[1] = 0x01; // Command: Set Mode
                payload[2] = code; // Sub-Command Sweep
                payload[3] = 0x01; // Action: Apply

                if let Err(e) = self.keyboard_hid.write(&payload) {
                    eprintln!("⚠️ C2 Handshake attempt (0x{:02x}) failed: {}", code, e);
                    let _ = std::io::stderr().flush();
                } else {
                    println!("🧪 Sent C2 handshake code: 0x{:02x}", code);
                    let _ = std::io::stdout().flush();
                }
                // Slow down for 2024 controllers to process the state switch
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        } else if self.protocol == ProtocolType::Legacy {
            // Legacy handshake: Reset to standard mode
            let mut payload: [u8; 33] = [0; 33];
            payload[1] = 0x02; // Command: Switch
            payload[2] = 0x01; // Manual mode
            let _ = self.keyboard_hid.send_feature_report(&payload);
            println!("🧪 Sent Legacy handshake (Manual Mode)");
            let _ = std::io::stdout().flush();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        Ok(())
    }

    pub fn set_effect(&mut self, effect: BaseEffects) -> Result<()> {
        self.current_state.effect_type = effect;
        self.refresh()?;

        Ok(())
    }

    pub fn set_speed(&mut self, speed: u8) -> Result<()> {
        if !SPEED_RANGE.contains(&speed) {
            return Err(RangeError { kind: RangeErrorKind::Speed }.into());
        }

        self.current_state.speed = speed;
        self.refresh()?;

        Ok(())
    }

    pub fn set_brightness(&mut self, brightness: u8) -> Result<()> {
        if !BRIGHTNESS_RANGE.contains(&brightness) {
            return Err(RangeError { kind: RangeErrorKind::Brightness }.into());
        }
        let brightness = brightness.clamp(BRIGHTNESS_RANGE.min().unwrap(), BRIGHTNESS_RANGE.max().unwrap());
        self.current_state.brightness = brightness;
        self.refresh()?;

        Ok(())
    }

    pub fn set_zone_by_index(&mut self, zone_index: u8, new_values: [u8; 3]) -> Result<()> {
        if !ZONE_RANGE.contains(&zone_index) {
            return Err(RangeError { kind: RangeErrorKind::Zone }.into());
        }

        for (i, _) in new_values.iter().enumerate() {
            let full_index = (zone_index * 3 + i as u8) as usize;
            self.current_state.rgb_values[full_index] = new_values[i];
        }
        self.refresh()?;

        Ok(())
    }

    pub fn set_colors_to(&mut self, new_values: &[u8; 12]) -> Result<()> {
        if let BaseEffects::Static | BaseEffects::Breath = self.current_state.effect_type {
            for (i, _) in new_values.iter().enumerate() {
                self.current_state.rgb_values[i] = new_values[i];
            }
            self.refresh()?;
        }

        Ok(())
    }

    pub fn solid_set_colors_to(&mut self, new_values: [u8; 3]) -> Result<()> {
        if let BaseEffects::Static | BaseEffects::Breath = self.current_state.effect_type {
            for i in (0..12).step_by(3) {
                self.current_state.rgb_values[i] = new_values[0];
                self.current_state.rgb_values[i + 1] = new_values[1];
                self.current_state.rgb_values[i + 2] = new_values[2];
            }
            self.refresh()?;
        }

        Ok(())
    }

    pub fn transition_colors_to(&mut self, target_colors: &[u8; 12], steps: u8, delay_between_steps: u64) -> Result<()> {
        if let BaseEffects::Static | BaseEffects::Breath = self.current_state.effect_type {
            let mut new_values = self.current_state.rgb_values.map(f32::from);
            let mut color_differences: [f32; 12] = [0.0; 12];
            for index in 0..12 {
                color_differences[index] = (f32::from(target_colors[index]) - f32::from(self.current_state.rgb_values[index])) / f32::from(steps);
            }
            if !self.stop_signal.load(Ordering::SeqCst) {
                for _step_num in 1..=steps {
                    if self.stop_signal.load(Ordering::SeqCst) {
                        break;
                    }
                    for (index, _) in color_differences.iter().enumerate() {
                        new_values[index] += color_differences[index];
                    }
                    self.current_state.rgb_values = new_values.map(|val| val as u8);

                    self.refresh()?;
                    thread::sleep(Duration::from_millis(delay_between_steps));
                }
                self.set_colors_to(target_colors)?;
            }
        }

        Ok(())
    }
}

pub fn get_keyboard(stop_signal: Arc<AtomicBool>) -> Result<Keyboard> {
    let api: HidApi = HidApi::new()?;
    let devices: Vec<_> = api.device_list().collect();

    println!("🔍 Scanning for Legion Keyboard...");
    let _ = std::io::stdout().flush();

    for d in devices.iter().filter(|d| d.vendor_id() == 0x048d) {
        println!(
            "  - Found Lenovo Device: PID=0x{:04x}, UsagePage=0x{:04x}, Usage=0x{:04x}, Interface={}",
            d.product_id(),
            d.usage_page(),
            d.usage(),
            d.interface_number()
        );
        let _ = std::io::stdout().flush();
    }

    // 1. Try our known high-priority list first
    for known in KNOWN_DEVICE_INFOS.iter() {
        if let Some(info) = devices.iter().find(|d| {
            #[cfg(target_os = "windows")]
            {
                let info_tuple = (d.vendor_id(), d.product_id(), d.usage_page(), d.usage());
                info_tuple == (known.0, known.1, known.2, known.3)
            }
            #[cfg(target_os = "linux")]
            {
                (d.vendor_id(), d.product_id()) == (known.0, known.1)
            }
        }) {
            let keyboard_hid: HidDevice = info.open_device(&api)?;

            // --- Peace Period ---
            // Allow the OS to finish any exclusive locks/ownership handoffs
            std::thread::sleep(std::time::Duration::from_millis(150));

            let current_state: LightingState = LightingState {
                effect_type: BaseEffects::Static,
                speed: 1,
                brightness: 1,
                rgb_values: [0; 12],
            };

            let mut keyboard = Keyboard {
                keyboard_hid,
                protocol: known.4,
                current_state,
                stop_signal,
            };

            keyboard.handshake()?;
            keyboard.refresh()?;
            println!("✅ Legion Keyboard Detected! (Known Model) Protocol: {:?}", keyboard.protocol);
            let _ = std::io::stdout().flush();
            return Ok(keyboard);
        }
    }

    // 2. Fallback: Try ANY device that looks like a C2 or Legacy lighting interface
    println!("⚠️ Known IDs failed. Attempting aggressive fallback scan...");
    let _ = std::io::stdout().flush();
    for d in devices.iter().filter(|d| d.vendor_id() == 0x048d) {
        let protocol = match (d.usage_page(), d.usage()) {
            (0xffc2, 0x0004) => Some(ProtocolType::C2),
            (0xff89, 0x00cc) => Some(ProtocolType::Legacy),
            (0xff89, 0x0010) => Some(ProtocolType::Legacy),
            _ => None,
        };

        if let Some(proto) = protocol {
            println!("🧪 Attempting to bind to potential {} interface...", if proto == ProtocolType::C2 { "C2" } else { "Legacy" });
            let _ = std::io::stdout().flush();
            if let Ok(keyboard_hid) = d.open_device(&api) {
                let mut keyboard = Keyboard {
                    keyboard_hid,
                    protocol: proto,
                    current_state: LightingState {
                        effect_type: BaseEffects::Static,
                        speed: 1,
                        brightness: 1,
                        rgb_values: [0; 12],
                    },
                    stop_signal: stop_signal.clone(),
                };

                let _ = keyboard.handshake();
                if keyboard.refresh().is_ok() {
                    println!("🎉 Aggressive fallback SUCCESS! Protocol: {:?}", proto);
                    let _ = std::io::stdout().flush();
                    return Ok(keyboard);
                }
            }
        }
    }

    Err(error::Error::DeviceNotFound.into())
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub usage_page: u16,
    pub usage: u16,
    pub manufacturer: String,
    pub product: String,
}

pub fn scan_it829x_devices() -> Result<Vec<DeviceInfo>> {
    let api = HidApi::new()?;
    let mut devices = Vec::new();

    for device in api.device_list() {
        if device.vendor_id() == 0x048d {
            devices.push(DeviceInfo {
                vendor_id: device.vendor_id(),
                product_id: device.product_id(),
                usage_page: device.usage_page(),
                usage: device.usage(),
                manufacturer: device.manufacturer_string().unwrap_or("Unknown").to_string(),
                product: device.product_string().unwrap_or("Unknown").to_string(),
            });
        }
    }

    Ok(devices)
}
