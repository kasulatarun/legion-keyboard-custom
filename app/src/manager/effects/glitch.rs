use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::manager::{profile::Profile, Inner};
use rand::Rng;

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let mut rng = rand::rng();

    let base_rgb = profile.rgb_array();

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut final_arr: [u8; 12] = [0; 12];

        let glitch_type = rng.random_range(0..100);

        if glitch_type < 10 {
            // "Static burst": High frequency white/bright flicker
            for i in 0..12 {
                final_arr[i] = rng.random_range(150..255);
            }
            manager.paint(profile, &final_arr);
            thread::sleep(Duration::from_millis(rng.random_range(20..60)));
        } else if glitch_type < 25 {
            // "Scanline": One zone goes dark or shifts color radically
            let zone = rng.random_range(0..4);
            for i in 0..4 {
                if i == zone {
                    final_arr[i * 3] = 255;
                    final_arr[i * 3 + 1] = 0;
                    final_arr[i * 3 + 2] = 255; // Magenta glitch
                } else {
                    final_arr[i * 3] = base_rgb[i * 3];
                    final_arr[i * 3 + 1] = base_rgb[i * 3 + 1];
                    final_arr[i * 3 + 2] = base_rgb[i * 3 + 2];
                }
            }
            manager.paint(profile, &final_arr);
            thread::sleep(Duration::from_millis(100));
        } else {
            // Normal base state with subtle jitter
            for i in 0..12 {
                let jitter = rng.random_range(0.9..1.1);
                final_arr[i] = (base_rgb[i] as f32 * jitter).clamp(0.0, 255.0) as u8;
            }
            manager.paint(profile, &final_arr);
            thread::sleep(Duration::from_millis(150));
        }
    }
}
