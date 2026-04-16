use std::{sync::atomic::Ordering, thread, time::{Duration, Instant}};
use rand::{rng, Rng};
use crate::manager::{profile::Profile, Inner};

pub fn play_summer_rain(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let mut rng = rng();
    let mut drops = [0.0f32; 4];
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut final_arr: [u8; 12] = [0; 12];
        
        // Randomly spark a new drop
        if rng.random_bool(0.05 * profile.speed as f64) {
            let zone = rng.random_range(0..4);
            drops[zone] = 1.0;
        }

        for i in 0..4 {
            let val = drops[i];
            
            // Soft Blue/Cyan raindrop with brighter base
            let r = (20.0 * val) as u8;
            let g = (100.0 + 150.0 * val * val) as u8;
            let b = (150.0 + 105.0 * val) as u8;
            
            final_arr[i*3] = r;
            final_arr[i*3+1] = g;
            final_arr[i*3+2] = b;
            
            // Organic fade
            drops[i] = (drops[i] - 0.05).max(0.0);
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16));
    }
}

pub fn play_aurora_borealis(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        let speed_mult = profile.speed as f32 * 0.4;
        
        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..4 {
            // Complex perlin-like wave pattern
            let phase1 = elapsed * speed_mult + (i as f32 * 0.5);
            let phase2 = elapsed * speed_mult * 1.5 - (i as f32 * 0.2);
            
            let val1 = (phase1.sin() + 1.0) / 2.0;
            let val2 = (phase2.cos() + 1.0) / 2.0;

            // Greens and deep purples, with a minimum brightness
            let r = (80.0 * val2).clamp(30.0, 255.0) as u8;
            let g = (255.0 * val1 + 50.0).clamp(50.0, 255.0) as u8;
            let b = (150.0 * val2 + 50.0).clamp(50.0, 255.0) as u8;
            
            final_arr[i*3] = r;
            final_arr[i*3+1] = g;
            final_arr[i*3+2] = b;
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16));
    }
}

pub fn play_starry_night(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let mut rng = rng();
    let mut twinkles = [0.0f32; 4];
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut final_arr: [u8; 12] = [0; 12];
        
        if rng.random_bool(0.02 * profile.speed as f64) {
            let zone = rng.random_range(0..4);
            twinkles[zone] = 1.0;
        }

        for i in 0..4 {
            let val = twinkles[i];
            
            // White / bright blue stars on dark blue background
            let r = (50.0 + 200.0 * val * val * val) as u8; 
            let g = (50.0 + 200.0 * val * val * val) as u8;
            let b = (80.0 + 175.0 * val * val) as u8;
            
            final_arr[i*3] = r;
            final_arr[i*3+1] = g;
            final_arr[i*3+2] = b; // Solid blue night sky background
            
            // Slow fade
            twinkles[i] = (twinkles[i] - 0.02).max(0.0);
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16));
    }
}
