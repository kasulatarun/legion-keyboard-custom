use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{FftPlanner, num_complex::Complex};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile, sensitivity: f32) {
    let stop_signals = manager.stop_signals.clone();
    
    // Buffer for FFT (must be power of 2)
    const FFT_SIZE: usize = 1024;
    let audio_buffer = Arc::new(Mutex::new(vec![0.0f32; FFT_SIZE]));
    let audio_buffer_clone = audio_buffer.clone();
    
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
    let sample_rate = config.sample_rate().0 as f32;
    
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if let Ok(mut buf) = audio_buffer_clone.lock() {
                // Shift existing samples left and add new ones
                let len = data.len().min(FFT_SIZE);
                buf.rotate_left(len);
                let start = FFT_SIZE - len;
                buf[start..].copy_from_slice(&data[..len]);
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    ).expect("Failed to build audio stream");

    stream.play().expect("Failed to start audio stream");

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    
    let mut last_intensities = [0.0f32; 4];
    let decay_rate = 5.0; // Decay factor for smooth transitions

    let mut last_update = Instant::now();

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let now = Instant::now();
        let dt = now.duration_since(last_update).as_secs_f32();
        last_update = now;

        let samples = if let Ok(buf) = audio_buffer.lock() {
            buf.clone()
        } else {
            vec![0.0; FFT_SIZE]
        };

        // Apply Hanning window to reduce leakage
        let mut complex_buffer: Vec<Complex<f32>> = samples.iter().enumerate().map(|(i, &s)| {
            let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos());
            Complex::new(s * window, 0.0)
        }).collect();

        fft.process(&mut complex_buffer);

        // Calculate magnitudes for bins
        // bin frequency = i * sample_rate / FFT_SIZE
        let magnitudes: Vec<f32> = complex_buffer[..FFT_SIZE/2].iter().map(|c| c.norm()).collect();

        // Split into 3 bands: Bass (0), Mid (1, 2), High (3)
        // Bass: 0 - 250 Hz
        // Mid: 250 - 4000 Hz
        // High: 4000+ Hz
        
        let mut bands = [0.0f32; 4];
        let hz_per_bin = sample_rate / FFT_SIZE as f32;

        for (i, &mag) in magnitudes.iter().enumerate() {
            let freq = i as f32 * hz_per_bin;
            if freq < 250.0 {
                bands[0] += mag;
            } else if freq < 4000.0 {
                bands[1] += mag;
                bands[2] += mag;
            } else {
                bands[3] += mag;
            }
        }

        // Normalize and scale by sensitivity
        let sens_adj = sensitivity * 0.01;
        bands[0] = (bands[0] * 0.05 * sens_adj).min(1.0);
        bands[1] = (bands[1] * 0.01 * sens_adj).min(1.0);
        bands[2] = (bands[1]); // Keep symetric for mids
        bands[3] = (bands[3] * 0.02 * sens_adj).min(1.0);

        let mut final_arr: [u8; 12] = [0; 12];
        let rgb_array = profile.rgb_array();

        for i in 0..4 {
            // Smooth transitions with decay
            if bands[i] > last_intensities[i] {
                last_intensities[i] = bands[i];
            } else {
                last_intensities[i] = (last_intensities[i] - decay_rate * dt).max(bands[i]);
            }
            
            let intensity = last_intensities[i];
            
            let brightness_mult = match profile.brightness {
                crate::enums::Brightness::Low => 0.5,
                crate::enums::Brightness::High => 1.0,
            };

            final_arr[i * 3] = (rgb_array[i * 3] as f32 * intensity * brightness_mult) as u8;
            final_arr[i * 3 + 1] = (rgb_array[i * 3 + 1] as f32 * intensity * brightness_mult) as u8;
            final_arr[i * 3 + 2] = (rgb_array[i * 3 + 2] as f32 * intensity * brightness_mult) as u8;
        }

        manager.keyboard.set_colors_to(&final_arr).unwrap();
        
        thread::sleep(Duration::from_millis(16)); // ~60FPS
    }
    
    drop(stream);
}
