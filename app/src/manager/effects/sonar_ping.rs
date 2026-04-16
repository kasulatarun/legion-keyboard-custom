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
        let ping_period = 3.0;
        let t = (elapsed % ping_period) / ping_period; // 0..1 sweep

        let mut final_arr: [u8; 12] = [0; 12];
        let base_rgb = profile.rgb_array();

        let ping_zone = (t * 4.0) as usize;

        for i in 0..4 {
            let mut intensity = 0.05; // Faint background glow

            if i == ping_zone {
                intensity = 1.0;
            } else if i == ping_zone.wrapping_sub(1) {
                intensity = 0.4; // Trail
            }

            final_arr[i * 3] = (base_rgb[i * 3] as f32 * intensity) as u8;
            final_arr[i * 3 + 1] = (base_rgb[i * 3 + 1] as f32 * intensity) as u8;
            final_arr[i * 3 + 2] = (base_rgb[i * 3 + 2] as f32 * intensity) as u8;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(50));
    }
}
