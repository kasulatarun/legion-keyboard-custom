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
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut final_arr: [u8; 12] = [0; 12];
        
        for i in 0..4 {
            // Randomly choose between Red, Orange, Yellow
            let r = rng.random_range(200..=255);
            let g = rng.random_range(0..=150);
            let b = 0;
            
            final_arr[i * 3] = r;
            final_arr[i * 3 + 1] = g;
            final_arr[i * 3 + 2] = b;
        }

        apply_brightness_and_send(manager, profile, &final_arr);
        
        let wait = (150 / profile.speed.max(1)) as u64;
        thread::sleep(Duration::from_millis(wait));
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
            // Sine wave based on position and time
            let phase = elapsed * speed_mult + (i as f32 * 0.8);
            let val = (phase.sin() + 1.0) / 2.0; // 0..1
            
            // Color: Deep Blue [0, 0, 150] to Teal [0, 200, 255]
            let r = 0;
            let g = (200.0 * val) as u8;
            let b = (150.0 + 105.0 * val) as u8;
            
            final_arr[i * 3] = r;
            final_arr[i * 3 + 1] = g;
            final_arr[i * 3 + 2] = b;
        }

        apply_brightness_and_send(manager, profile, &final_arr);
        thread::sleep(Duration::from_millis(33)); // ~30 FPS
    }
}

pub fn play_meteor(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    
    let meteor_color = profile.rgb_zones[0].rgb; // Use first zone color as meteor color

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        let speed_mult = profile.speed as f32 * 1.5;
        
        // Bounce position: 0 -> 3 -> 0
        let pos = (elapsed * speed_mult).sin().abs() * 3.0;
        
        let mut final_arr: [u8; 12] = [0; 12];
        
        for i in 0..4 {
            let dist = (i as f32 - pos).abs();
            let intensity = (1.0 - dist).max(0.1); // Trail effect
            
            final_arr[i * 3] = (meteor_color[0] as f32 * intensity) as u8;
            final_arr[i * 3 + 1] = (meteor_color[1] as f32 * intensity) as u8;
            final_arr[i * 3 + 2] = (meteor_color[2] as f32 * intensity) as u8;
        }

        apply_brightness_and_send(manager, profile, &final_arr);
        thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }
}

fn apply_brightness_and_send(manager: &mut Inner, profile: &Profile, arr: &[u8; 12]) {
    let mut final_arr = *arr;
    let brightness_mult = match profile.brightness {
        crate::enums::Brightness::Low => 0.5,
        crate::enums::Brightness::High => 1.0,
    };

    for i in 0..12 {
        final_arr[i] = (final_arr[i] as f32 * brightness_mult) as u8;
    }

    manager.keyboard.set_colors_to(&final_arr).unwrap();
}
