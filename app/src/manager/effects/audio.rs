use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rand::{rng, Rng};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile, sensitivity: f32) {
    let stop_signals = manager.stop_signals.clone();
    
    let volume = Arc::new(std::sync::Mutex::new(0.0f32));
    let volume_clone = volume.clone();
    
    let host = cpal::default_host();
    
    #[cfg(target_os = "windows")]
    let device = {
        let mut devices = host.output_devices().expect("No output devices");
        devices.find(|d| d.name().map(|n| n.contains("Default") || n.contains("Stereo Mix")).unwrap_or(false))
            .or_else(|| host.default_output_device())
    };
        
    #[cfg(not(target_os = "windows"))]
    let device = host.default_input_device();

    let device = match device {
        Some(d) => d,
        None => return,
    };

    let config = device.default_output_config().expect("Failed to get audio config");
    
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut max_abs = 0.0f32;
            for &sample in data {
                max_abs = max_abs.max(sample.abs());
            }
            if let Ok(mut v) = volume_clone.lock() {
                // Smoothing the input volume slightly to avoid jitter
                *v = (*v * 0.4) + (max_abs * 0.6);
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    ).expect("Failed to build audio stream");

    stream.play().expect("Failed to start audio stream");

    let mut rng = rng();
    let mut zone_intensities = [0.0f32; 4];
    let mut zone_colors = [[0u8; 3]; 4];
    
    // Initialize random colors
    for i in 0..4 {
        zone_colors[i] = [rng.random_range(0..=255), rng.random_range(0..=255), rng.random_range(0..=255)];
    }

    let mut last_update = Instant::now();
    let mut vol_history = Vec::with_capacity(30);
    let mut beat_cooldown = Instant::now();

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let now = Instant::now();
        let dt = now.duration_since(last_update).as_secs_f32();
        last_update = now;

        let current_volume = if let Ok(v) = volume.lock() { *v } else { 0.0 };
        
        // Rolling average for beat detection
        vol_history.push(current_volume);
        if vol_history.len() > 30 { vol_history.remove(0); }
        let avg_vol: f32 = vol_history.iter().sum::<f32>() / vol_history.len() as f32;

        // Peak detection (Beat)
        // Sensitivity slider adjusts the threshold (0-100, default 1.0)
        // We'll treat 1.0 as a sane default.
        let threshold_multiplier = 1.2 + (50.0 / (sensitivity + 0.1)); 
        let is_beat = current_volume > avg_vol * 1.3 && current_volume > 0.05 && beat_cooldown.elapsed() > Duration::from_millis(80);
        
        if is_beat {
            beat_cooldown = now;
            for i in 0..4 {
                zone_intensities[i] = 1.0;
                // Full randomization on beat
                zone_colors[i] = [rng.random_range(0..=255), rng.random_range(0..=255), rng.random_range(0..=255)];
            }
        }

        // Decay logic
        let decay_rate = 6.0; // Rapid decay for "staccato" feel
        let sens_adj = (sensitivity / 25.0).max(0.1); // Normalize sensitivity
        
        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..4 {
            // Decay intensity but don't drop below current scaled volume
            zone_intensities[i] = (zone_intensities[i] - dt * decay_rate).max(current_volume * sens_adj);
            
            let intensity = (zone_intensities[i]).clamp(0.0, 1.0);
            
            // Apply brightness from profile as well
            let brightness_mult = match profile.brightness {
                crate::enums::Brightness::Low => 0.5,
                crate::enums::Brightness::High => 1.0,
            };

            final_arr[i * 3] = (zone_colors[i][0] as f32 * intensity * brightness_mult) as u8;
            final_arr[i * 3 + 1] = (zone_colors[i][1] as f32 * intensity * brightness_mult) as u8;
            final_arr[i * 3 + 2] = (zone_colors[i][2] as f32 * intensity * brightness_mult) as u8;
        }

        manager.keyboard.set_colors_to(&final_arr).unwrap();
        
        // ~60 FPS update rate for smooth visuals
        thread::sleep(Duration::from_millis(16));
    }
    
    drop(stream);
}
