use std::{sync::atomic::Ordering, thread, time::{Duration, Instant}};
use rand::{rng, Rng};
use crate::manager::{profile::Profile, Inner};

pub fn play_vhs_retro(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    let mut rng = rng();
    let base_rgb = profile.rgb_array();
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        
        let mut final_arr: [u8; 12] = [0; 12];
        
        // Scanline passing through
        let scan_zone = ((elapsed * (profile.speed as f32 * 2.0)) as usize) % 4;
        
        for i in 0..4 {
            let mut r = base_rgb[i*3] as f32;
            let mut g = base_rgb[i*3+1] as f32;
            let mut b = base_rgb[i*3+2] as f32;
            
            // Chromatic aberration (shift colors slightly based on neighbors)
            if rng.random_bool(0.1) {
                if i > 0 {
                    r = base_rgb[(i-1)*3] as f32; // R comes from left
                }
                if i < 3 {
                    b = base_rgb[(i+1)*3+2] as f32; // B comes from right
                }
            }

            // Scanline effect
            if i == scan_zone {
                r *= 1.15;
                g *= 1.15;
                b *= 1.15;
            } else {
                r *= 0.9;
                g *= 0.9;
                b *= 0.9;
            }
            
            // Random noise/tracking error
            if rng.random_bool(0.05) {
                r = rng.random_range(0.0..255.0);
                g = rng.random_range(0.0..255.0);
                b = rng.random_range(0.0..255.0);
            }
            
            final_arr[i*3] = r.clamp(0.0, 255.0) as u8;
            final_arr[i*3+1] = g.clamp(0.0, 255.0) as u8;
            final_arr[i*3+2] = b.clamp(0.0, 255.0) as u8;
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(30)); // 30fps is good for retro feel
    }
}
