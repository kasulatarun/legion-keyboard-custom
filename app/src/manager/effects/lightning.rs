use std::{sync::atomic::Ordering, thread, time::Duration};

use rand::{rngs::ThreadRng, Rng};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, p: &Profile, rng: &mut ThreadRng) {
    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let profile_array = p.rgb_array();

        if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            break;
        }
        let zone_index = rng.random_range(0..4);
        let steps = rng.random_range(50..=200);

        let mut arr = [0; 12];
        let zone_start = zone_index * 3;

        arr[zone_start] = profile_array[zone_start];
        arr[zone_start + 1] = profile_array[zone_start + 1];
        arr[zone_start + 2] = profile_array[zone_start + 2];

        manager.keyboard.set_colors_to(&arr).unwrap();
        // First strike fade
        manager.keyboard.transition_colors_to(&[0; 12], steps / p.speed, 2).unwrap();
        
        // Short pause for double strike
        thread::sleep(Duration::from_millis(rng.random_range(20..80)));
        
        // Second strike (often weaker but faster) if chance is met
        if rng.random_bool(0.7) {
            manager.keyboard.set_colors_to(&arr).unwrap();
            manager.keyboard.transition_colors_to(&[0; 12], steps / (p.speed * 2).max(1), 3).unwrap();
        }

        let sleep_time = rng.random_range(100..=2000);
        thread::sleep(Duration::from_millis(sleep_time));
    }
}
