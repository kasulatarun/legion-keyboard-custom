use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use crate::manager::{profile::Profile, Inner};
use sysinfo::Networks;

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let mut networks = Networks::new_with_refreshed_list();

    let start_time = Instant::now();
    let mut last_rx: u64 = 0;

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        networks.refresh(true);

        // Sum total received bytes across all interfaces
        let mut current_rx: u64 = 0;
        for (_, network) in &networks {
            current_rx += network.total_received();
        }

        let delta = if last_rx > 0 { current_rx.saturating_sub(last_rx) } else { 0 };
        last_rx = current_rx;

        // Normalize speed (0 to 1MB/s as a baseline)
        let mb_per_sec = (delta as f32) / (1024.0 * 1024.0);
        let speed_factor = (mb_per_sec / 2.0).min(1.0); // Caps at 2MB/s for visual intensity

        let elapsed = start_time.elapsed().as_secs_f32();
        let flow_speed = 0.5 + (speed_factor * 10.0);

        let mut final_arr: [u8; 12] = [0; 12];
        let base_rgb = profile.rgb_array();

        for i in 0..4 {
            // Sine wave horizontal flow
            let intensity = (((elapsed * flow_speed + (i as f32 * 0.8)).sin() + 1.0) / 2.0) * 0.7 + 0.3;

            final_arr[i * 3] = (base_rgb[i * 3] as f32 * intensity) as u8;
            final_arr[i * 3 + 1] = (base_rgb[i * 3 + 1] as f32 * intensity) as u8;
            final_arr[i * 3 + 2] = (base_rgb[i * 3 + 2] as f32 * intensity) as u8;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(50));
    }
}
