use std::{sync::atomic::Ordering, thread, time::{Duration, Instant}};
use rand::{rng, Rng};
use crate::manager::{profile::Profile, Inner};

pub fn play_light_leak(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        let speed_mult = profile.speed as f32 * 0.8;
        
        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..4 {
            let phase = elapsed * speed_mult + (i as f32 * 1.5);
            let val = (phase.sin() + 1.0) / 2.0;

            let r = (150.0 + 105.0 * val) as u8;
            let g = (80.0 + 120.0 * val) as u8; // Amber/Orange
            let b = (10.0 + 30.0 * val) as u8;
            
            final_arr[i*3] = r; 
            final_arr[i*3+1] = g;
            final_arr[i*3+2] = b;
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16)); // ~60fps
    }
}

pub fn play_neon_dream(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        let speed_mult = profile.speed as f32 * 1.2;
        
        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..4 {
            // Oscillating between Pink [255, 50, 150], Purple [150, 50, 255], Cyan [50, 255, 255]
            let phase1 = elapsed * speed_mult + (i as f32 * 0.5);
            let phase2 = elapsed * speed_mult * 0.8 + (i as f32 * 0.8);
            
            let v1 = (phase1.sin() + 1.0) / 2.0;
            let v2 = (phase2.cos() + 1.0) / 2.0;

            let r = (255.0 * v1 + 100.0 * v2).clamp(50.0, 255.0) as u8;
            let g = (255.0 * v2).clamp(50.0, 255.0) as u8;
            let b = (150.0 * v1 + 255.0 * (1.0 - v2)).clamp(100.0, 255.0) as u8;
            
            final_arr[i*3] = r;
            final_arr[i*3+1] = g;
            final_arr[i*3+2] = b;
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16));
    }
}

pub fn play_soft_bloom(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    let base_rgb = profile.rgb_array();
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        let speed_mult = profile.speed as f32 * 0.5;
        
        let mut final_arr: [u8; 12] = [0; 12];
        // Global bloom
        let bloom_val = (elapsed * speed_mult).sin(); 
        let intensity = (bloom_val * bloom_val) * 0.6 + 0.4; // Soft expansion, high baseline

        for i in 0..12 {
            final_arr[i] = (base_rgb[i] as f32 * intensity).clamp(0.0, 255.0) as u8;
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16));
    }
}

pub fn play_sunset_glow(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        let speed_mult = profile.speed as f32 * 0.3;
        
        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..4 {
            let phase = elapsed * speed_mult + (i as f32 * 0.2); // Slow cross-zone shift
            let val = (phase.sin() + 1.0) / 2.0;

            // Deep Red to Purple to Orange with bright baseline
            let r = (200.0 + 55.0 * val.sin()).clamp(100.0, 255.0) as u8;
            let g = (50.0 + 100.0 * val).clamp(50.0, 255.0) as u8;
            let b = (50.0 + 120.0 * (1.0 - val)).clamp(50.0, 255.0) as u8;
            
            final_arr[i*3] = r;
            final_arr[i*3+1] = g;
            final_arr[i*3+2] = b;
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16));
    }
}

pub fn play_candlelight(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let mut rng = rng();
    let mut targets = [1.0f32; 4];
    let mut currents = [1.0f32; 4];
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut final_arr: [u8; 12] = [0; 12];
        
        for i in 0..4 {
            if rng.random_bool(0.1) {
                // Occasional dramatic flicker
                targets[i] = rng.random_range(0.3..0.6);
            } else if rng.random_bool(0.3) {
                // Micro flicker
                targets[i] = rng.random_range(0.8..1.0);
            } else {
                targets[i] += (1.0 - targets[i]) * 0.2; // Return to baseline
            }
            
            // Interpolate toward target for realism
            currents[i] += (targets[i] - currents[i]) * 0.3;
            
            let val = currents[i].max(0.4); // Minimum brightness so it's not "nothing"
            
            // Warm candle colors [255, 147, 41] scaled by flicker
            let r = (255.0 * val).clamp(100.0, 255.0) as u8;
            let g = (147.0 * val * val).clamp(50.0, 255.0) as u8; // G drops faster for redder flicker
            let b = (41.0 * val * val * val).clamp(10.0, 255.0) as u8; // B drops even faster
            
            final_arr[i*3] = r;
            final_arr[i*3+1] = g;
            final_arr[i*3+2] = b;
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(30)); // slightly slower tick for better random distribution
    }
}
