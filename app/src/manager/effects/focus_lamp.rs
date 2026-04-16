use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut final_arr: [u8; 12] = [0; 12];

        // Warm White Lamp: 3000K approx [255, 180, 100]
        let lamp_rgb = [255, 180, 100];
        let ambient_rgb = [20, 10, 5]; // Very dim edges

        // Zone 1: Dim
        final_arr[0] = ambient_rgb[0];
        final_arr[1] = ambient_rgb[1];
        final_arr[2] = ambient_rgb[2];
        // Zone 2: Bright Lamp
        final_arr[3] = lamp_rgb[0];
        final_arr[4] = lamp_rgb[1];
        final_arr[5] = lamp_rgb[2];
        // Zone 3: Bright Lamp
        final_arr[6] = lamp_rgb[0];
        final_arr[7] = lamp_rgb[1];
        final_arr[8] = lamp_rgb[2];
        // Zone 4: Dim
        final_arr[9] = ambient_rgb[0];
        final_arr[10] = ambient_rgb[1];
        final_arr[11] = ambient_rgb[2];

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(500)); // Static focus mode
    }
}
