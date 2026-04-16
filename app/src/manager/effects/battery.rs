use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::manager::{profile::Profile, Inner};
use battery::Manager;

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();

    let battery_manager = Manager::new().ok();

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut charge_ratio = 1.0; // Default to full

        if let Some(ref bm) = battery_manager {
            if let Ok(mut batteries) = bm.batteries() {
                if let Some(Ok(battery)) = batteries.next() {
                    charge_ratio = battery.state_of_charge().value;
                }
            }
        }

        let mut final_arr: [u8; 12] = [0; 12];
        let color = get_battery_color(charge_ratio);

        // Low battery pulsing (under 25%)
        let intensity = if charge_ratio < 0.25 {
            (((std::time::Instant::now().elapsed().as_secs_f32() * 5.0).sin() + 1.0) / 2.0) * 0.7 + 0.3
        } else {
            1.0
        };

        for i in 0..4 {
            final_arr[i * 3] = (color[0] as f32 * intensity) as u8;
            final_arr[i * 3 + 1] = (color[1] as f32 * intensity) as u8;
            final_arr[i * 3 + 2] = (color[2] as f32 * intensity) as u8;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(500)); // Refresh every 0.5s
    }
}

fn get_battery_color(ratio: f32) -> [u8; 3] {
    if ratio >= 0.8 {
        [0, 255, 0] // Green
    } else if ratio >= 0.3 {
        // Transition Green to Orange
        let factor = (ratio - 0.3) / 0.5; // 0..1
        let r = (255.0 * (1.0 - factor)) as u8;
        let g = 200; // Keep it bright green-ish orange
        [r, g, 0]
    } else {
        // Transition Orange to Red
        let factor = (ratio - 0.1) / 0.2; // 0..1
        let g = (200.0 * factor.max(0.0)) as u8;
        [255, g, 0]
    }
}
