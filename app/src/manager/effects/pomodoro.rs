use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile, duration_mins: u32) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    let total_duration = Duration::from_secs(duration_mins as u64 * 60);

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed();
        let progress = (elapsed.as_secs_f32() / total_duration.as_secs_f32()).min(1.0);
        
        // Green (Start) -> Red (End)
        let mut final_arr: [u8; 12] = [0; 12];
        
        for i in 0..4 {
            let zone_start = i as f32 / 4.0;
            let zone_end = (i + 1) as f32 / 4.0;
            
            if progress > zone_end {
                // Completed zone: Solid Red
                final_arr[i * 3] = 255;
                final_arr[i * 3 + 1] = 0;
                final_arr[i * 3 + 2] = 0;
            } else if progress > zone_start {
                // Currently filling zone: Blend Green to Red
                let zone_progress = (progress - zone_start) / (zone_end - zone_start);
                final_arr[i * 3] = (255.0 * zone_progress) as u8;
                final_arr[i * 3 + 1] = (255.0 * (1.0 - zone_progress)) as u8;
                final_arr[i * 3 + 2] = 0;
            } else {
                // Future zone: Solid Green
                final_arr[i * 3] = 0;
                final_arr[i * 3 + 1] = 255;
                final_arr[i * 3 + 2] = 0;
            }
        }

        // Apply brightness
        let brightness_mult = match profile.brightness {
            crate::enums::Brightness::Low => 0.5,
            crate::enums::Brightness::High => 1.0,
        };

        for i in 0..12 {
            final_arr[i] = (final_arr[i] as f32 * brightness_mult) as u8;
        }

        manager.keyboard.set_colors_to(&final_arr).unwrap();

        if progress >= 1.0 {
            // Timer finished, flash red
            loop {
                if stop_signals.manager_stop_signal.load(Ordering::SeqCst) { break; }
                
                let red = [255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0];
                manager.keyboard.set_colors_to(&red).unwrap();
                thread::sleep(Duration::from_millis(500));
                
                let off = [0; 12];
                manager.keyboard.set_colors_to(&off).unwrap();
                thread::sleep(Duration::from_millis(500));
            }
            break;
        }

        thread::sleep(Duration::from_millis(1000)); // Update every second
    }
}
