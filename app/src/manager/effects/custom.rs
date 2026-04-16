use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use rand::{rng, Rng};

use crate::manager::{profile::Profile, Inner};

pub fn play_fire(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let mut rng = rng();
    let mut heat = [0.0f32; 4];

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut final_arr: [u8; 12] = [0; 12];

        for i in 0..4 {
            // Add or subtract random heat
            let spark = rng.random_range(-0.2..0.3);
            heat[i] = (heat[i] + spark).clamp(0.0, 1.0);
            
            // Random ember pops
            if rng.random_bool((profile.speed as f64) * 0.02) {
                heat[i] = 1.0; 
            }

            // Map heat to color (Black -> Red -> Orange -> Yellow -> White)
            let val = heat[i];
            
            let r = (255.0 * val.powf(0.5)).clamp(0.0, 255.0) as u8;
            let g = (180.0 * val * val).clamp(0.0, 255.0) as u8;
            let b = (50.0 * val * val * val).clamp(0.0, 255.0) as u8;

            final_arr[i * 3] = r.max(20); // base red glow
            final_arr[i * 3 + 1] = g;
            final_arr[i * 3 + 2] = b;
        }

        manager.paint(profile, &final_arr);

        // Standardize to 60fps instead of speed-based sleep for smoother look
        let wait = (100 / profile.speed.max(1)) as u64;
        thread::sleep(Duration::from_millis(16.max(wait))); 
    }
}

pub fn play_ocean(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        let speed_mult = profile.speed as f32 * 0.5;

        let mut final_arr: [u8; 12] = [0; 12];

        for i in 0..4 {
            // Base sine wave based on position and time
            let phase = elapsed * speed_mult + (i as f32 * 0.8);
            let val = (phase.sin() + 1.0) / 2.0;

            // Extra high-frequency shimmer for "sunlight reflection"
            let shimmer_phase = elapsed * speed_mult * 3.0 + (i as f32 * 2.0);
            let shimmer = ((shimmer_phase.sin() + 1.0) / 2.0).powf(4.0) * 0.3; 

            // Color: Deep Blue [0, 0, 150] to Teal [0, 200, 255] + Shimmer
            let r = (255.0 * shimmer).clamp(0.0, 255.0) as u8;
            let g = (200.0 * val + 255.0 * shimmer).clamp(0.0, 255.0) as u8;
            let b = (150.0 + 105.0 * val + 255.0 * shimmer).clamp(0.0, 255.0) as u8;

            final_arr[i * 3] = r;
            final_arr[i * 3 + 1] = g;
            final_arr[i * 3 + 2] = b;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16)); // 60 FPS
    }
}

pub fn play_meteor(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    let base_rgb = profile.rgb_array();

    let meteor_color = profile.rgb_zones[0].rgb; // Use first zone color as meteor color

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        let speed_mult = profile.speed as f32 * 1.5;

        // Bounce position: 0 -> 3 -> 0 using smooth interpolation
        let pos = (elapsed * speed_mult).sin().abs() * 3.0;

        let mut final_arr: [u8; 12] = [0; 12];

        for i in 0..4 {
            let dist = (i as f32 - pos).abs();
            // Sharper dropoff for modern trail effect
            let intensity = (1.0 - (dist / 1.5)).clamp(0.0, 1.0).powf(1.5); 

            // Add back a small amount of the base color so the keyboard isn't dark
            final_arr[i * 3] = ((meteor_color[0] as f32 * intensity) + (base_rgb[i*3] as f32 * 0.1)).clamp(0.0, 255.0) as u8;
            final_arr[i * 3 + 1] = ((meteor_color[1] as f32 * intensity) + (base_rgb[i*3+1] as f32 * 0.1)).clamp(0.0, 255.0) as u8;
            final_arr[i * 3 + 2] = ((meteor_color[2] as f32 * intensity) + (base_rgb[i*3+2] as f32 * 0.1)).clamp(0.0, 255.0) as u8;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }
}
