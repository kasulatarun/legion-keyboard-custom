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
        let speed = match profile.effect {
            crate::enums::Effects::PulseStatic { .. } => (profile.speed as f32).max(0.5),
            _ => 1.0,
        };

        // Sine wave for breathing effect (0.4 to 1.0 intensity range)
        let intensity = (((elapsed * speed).sin() + 1.0) / 2.0) * 0.6 + 0.4;

        let mut final_arr: [u8; 12] = [0; 12];
        let rgb_array = profile.rgb_array();

        for i in 0..12 {
            final_arr[i] = (rgb_array[i] as f32 * intensity) as u8;
        }

        manager.paint(profile, &final_arr);

        thread::sleep(Duration::from_millis(33)); // ~30 FPS
    }
}
