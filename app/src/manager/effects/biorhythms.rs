use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::manager::{profile::Profile, Inner};
use chrono::{Local, Timelike};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let now = Local::now();
        let hour = now.hour() as f32 + (now.minute() as f32 / 60.0);

        // Map 0..24 to a specific color palette
        // 0-4 (Deep Night): Dark Blue
        // 4-8 (Sunrise): Deep Red -> Orange -> Yellow
        // 8-16 (Daylight): Bright White/Cyan
        // 16-20 (Sunset): Golden -> Purple
        // 20-24 (Evening): Deep Blue

        let color = get_biorhythm_color(hour);

        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..4 {
            final_arr[i * 3] = color[0];
            final_arr[i * 3 + 1] = color[1];
            final_arr[i * 3 + 2] = color[2];
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_secs(60)); // Only refresh once per minute
    }
}

fn get_biorhythm_color(hour: f32) -> [u8; 3] {
    if hour < 5.0 || hour >= 22.0 {
        [0, 5, 40] // Midnight Blue
    } else if hour < 7.0 {
        // Sunrise: 5..7
        let factor = (hour - 5.0) / 2.0;
        let r = (200.0 * factor) as u8;
        let b = (40.0 * (1.0 - factor)) as u8;
        [r, 50, b]
    } else if hour < 9.0 {
        // Morning: 7..9
        let factor = (hour - 7.0) / 2.0;
        let g = (50.0 + 150.0 * factor) as u8;
        [255, g, 50] // Golden yellow
    } else if hour < 17.0 {
        // High Sun: 9..17
        [200, 230, 255] // Cool White
    } else if hour < 19.0 {
        // Sunset: 17..19
        let factor = (hour - 17.0) / 2.0;
        let b = (255.0 * factor) as u8;
        [255, 100, b] // Pink/Orange
    } else {
        // Twilight: 19..22
        let factor = (hour - 19.0) / 3.0;
        let r = (255.0 * (1.0 - factor)) as u8;
        [r, 50, 200] // Purple -> Blue
    }
}
