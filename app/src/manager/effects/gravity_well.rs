use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();

        // The "Well" moves back and forth across 4 zones
        let well_pos = ((elapsed * 0.8).sin() + 1.0) * 1.5; // 0.0 to 3.0

        let mut final_arr: [u8; 12] = [0; 12];
        let base_rgb = profile.rgb_array();

        for i in 0..4 {
            let dist = (i as f32 - well_pos).abs();
            // Higher intensity closer to the well
            let gravity = (1.0 - (dist / 2.0)).max(0.1);

            // "Pulls" color from neighbors (simulated by increased brightness and saturation at the well)
            final_arr[i * 3] = (base_rgb[i * 3] as f32 * gravity).min(255.0) as u8;
            final_arr[i * 3 + 1] = (base_rgb[i * 3 + 1] as f32 * gravity).min(255.0) as u8;
            final_arr[i * 3 + 2] = (base_rgb[i * 3 + 2] as f32 * gravity).min(255.0) as u8;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(40));
    }
}
