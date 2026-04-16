use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();

    let mut smoothed_arr: [f32; 12] = [0.0; 12];

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();

        // Very slow breathing: 10s period
        let t = ((elapsed * 0.6).sin() + 1.0) / 2.0;

        // Interpolate between Deep Blue [0, 10, 50] and Forest Green [0, 40, 10]
        let r = 0.0;
        let g = 10.0 + (30.0 * t);
        let b = 50.0 - (40.0 * t);

        let target_rgb = [r as f32, g as f32, b as f32];

        for i in 0..4 {
            for j in 0..3 {
                let idx = i * 3 + j;
                smoothed_arr[idx] += (target_rgb[j] - smoothed_arr[idx]) * 0.05;
                // Extra smooth
            }
        }

        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..12 {
            final_arr[i] = smoothed_arr[i] as u8;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(50));
    }
}
