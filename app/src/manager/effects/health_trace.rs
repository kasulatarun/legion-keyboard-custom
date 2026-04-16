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

        // Rhythmic sweep: 4s period
        let t = ((elapsed * 1.5).sin() + 1.0) / 2.0;

        // Target color: Green [0, 255, 0] at peaks, Red [255, 0, 0] at valleys
        let r_target = 255.0 * (1.0 - t);
        let g_target = 255.0 * t;
        let b_target = 0.0;

        let target_rgb = [r_target, g_target, b_target];

        for i in 0..4 {
            for j in 0..3 {
                let idx = i * 3 + j;
                smoothed_arr[idx] += (target_rgb[j] - smoothed_arr[idx]) * 0.1;
            }
        }

        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..12 {
            final_arr[i] = smoothed_arr[i] as u8;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(40));
    }
}
