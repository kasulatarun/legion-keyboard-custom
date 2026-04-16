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
        let cycle_duration = 2.0;
        let t = (elapsed % cycle_duration) / cycle_duration; // 0..1 loop

        let mut final_arr: [u8; 12] = [0; 12];
        let rgb_array = profile.rgb_array();

        if t < 0.8 {
            // "Approach" phase: two points moving towards the center
            let pos1 = (t / 0.8) * 1.5; // moves from 0 to 1.5
            let pos2 = 3.0 - (t / 0.8) * 1.5; // moves from 3 to 1.5

            for i in 0..4 {
                let d1 = (i as f32 - pos1).abs();
                let d2 = (i as f32 - pos2).abs();
                let intensity = ((1.0 - d1).max(0.0) + (1.0 - d2).max(0.0)).min(1.0);

                final_arr[i * 3] = (rgb_array[i * 3] as f32 * intensity) as u8;
                final_arr[i * 3 + 1] = (rgb_array[i * 3 + 1] as f32 * intensity) as u8;
                final_arr[i * 3 + 2] = (rgb_array[i * 3 + 2] as f32 * intensity) as u8;
            }
        } else {
            // "Explosion" phase (last 20% of cycle)
            let explosion_t = (t - 0.8) / 0.2; // 0..1
            let intensity = 1.0 - explosion_t;

            for i in 0..4 {
                // Flash brighter colors during explosion
                final_arr[i * 3] = ((rgb_array[i * 3] as f32 + 100.0 * intensity).min(255.0) * intensity) as u8;
                final_arr[i * 3 + 1] = ((rgb_array[i * 3 + 1] as f32 + 50.0 * intensity).min(255.0) * intensity) as u8;
                final_arr[i * 3 + 2] = ((rgb_array[i * 3 + 2] as f32 + 200.0 * intensity).min(255.0) * intensity) as u8;
            }
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(33)); // ~30 FPS
    }
}
