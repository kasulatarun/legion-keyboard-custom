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
    
    // RMS / Peak volume calculation
    let volume = Arc::new(std::sync::Mutex::new(0.0f32));
    let volume_clone = volume.clone();
    
    let host = cpal::default_host();
    
    // On Windows, we try to use the loopback device for system audio
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
        None => {
            eprintln!("No audio device found");
            return;
        }
    };

    // For loopback on Windows, we need to use a special config or just Stereo Mix if available
    let config = device.default_output_config().expect("Failed to get audio config");
    
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut max_abs = 0.0f32;
            for &sample in data {
                max_abs = max_abs.max(sample.abs());
            }
            if let Ok(mut v) = volume_clone.lock() {
                *v = max_abs;
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    ).expect("Failed to build audio stream");

    stream.play().expect("Failed to start audio stream");

    let mut last_color_change = Instant::now();
    let mut current_colors: [[u8; 3]; 4] = [[0; 3]; 4];
    let mut rng = rng();

    // Initialize with random colors
    for i in 0..4 {
        current_colors[i] = [rng.random_range(0..=255), rng.random_range(0..=255), rng.random_range(0..=255)];
    }

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let current_volume = if let Ok(v) = volume.lock() {
            *v
        } else {
            0.0
        };

        // Scale volume by sensitivity
        let intensity = (current_volume * sensitivity).clamp(0.0, 1.0);

        // Periodically change base colors if volume is above a threshold (optional)
        if last_color_change.elapsed() > Duration::from_secs(2) && current_volume > 0.05 {
            for i in 0..4 {
                current_colors[i] = [rng.random_range(0..=255), rng.random_range(0..=255), rng.random_range(0..=255)];
            }
            last_color_change = Instant::now();
        }

        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..4 {
            final_arr[i * 3] = (current_colors[i][0] as f32 * intensity) as u8;
            final_arr[i * 3 + 1] = (current_colors[i][1] as f32 * intensity) as u8;
            final_arr[i * 3 + 2] = (current_colors[i][2] as f32 * intensity) as u8;
        }

        manager.keyboard.set_colors_to(&final_arr).unwrap();
        
        // Match the update rate (approx 50ms)
        thread::sleep(Duration::from_millis(50));
    }
    
    drop(stream);
}
