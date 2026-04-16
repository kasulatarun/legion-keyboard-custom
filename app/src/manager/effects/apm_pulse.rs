use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::manager::{profile::Profile, Inner};
use device_query::{DeviceEvents, DeviceEventsHandler, Keycode};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();

    let kill_thread = Arc::new(AtomicBool::new(false));
    let exit_thread = kill_thread.clone();
    let (tx, rx) = crossbeam_channel::unbounded::<Keycode>();

    thread::spawn(move || {
        let event_handler = DeviceEventsHandler::new(Duration::from_millis(10)).unwrap_or(DeviceEventsHandler {});
        let tx_clone = tx.clone();
        let _press_guard = event_handler.on_key_down(move |key| {
            let _ = tx_clone.send(*key);
        });
        while !exit_thread.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }
    });

    let mut apm_intensity: f32 = 0.0;
    let mut smoothed_arr: [f32; 12] = [0.0; 12];

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        // Count keys and boost intensity
        while let Ok(_) = rx.try_recv() {
            apm_intensity = (apm_intensity + 0.15).min(1.0);
        }

        // Decay intensity
        apm_intensity = (apm_intensity * 0.96).max(0.0);

        // Color shifts from Cyan [0, 255, 255] to Neon Magenta [255, 0, 255]
        let r_target = 255.0 * apm_intensity;
        let g_target = 255.0 * (1.0 - apm_intensity);
        let b_target = 255.0;

        let target_rgb = [r_target, g_target, b_target];

        for i in 0..4 {
            let zone_lag = i as f32 * 0.1; // Slight zone delay for extra flair
            let factor = (0.2 - zone_lag).max(0.05);

            for j in 0..3 {
                let idx = i * 3 + j;
                smoothed_arr[idx] += (target_rgb[j] - smoothed_arr[idx]) * factor;
            }
        }

        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..12 {
            final_arr[i] = smoothed_arr[i] as u8;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(30));
    }

    kill_thread.store(true, Ordering::SeqCst);
}
