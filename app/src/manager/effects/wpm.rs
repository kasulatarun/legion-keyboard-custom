use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use device_query::{DeviceEvents, DeviceEventsHandler};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let kill_thread = Arc::new(AtomicBool::new(false));
    let exit_thread = kill_thread.clone();

    let (tx, rx) = crossbeam_channel::unbounded::<Instant>();

    thread::spawn(move || {
        let event_handler = DeviceEventsHandler::new(Duration::from_millis(10)).unwrap_or(DeviceEventsHandler {});
        let tx_clone = tx.clone();

        let _press_guard = event_handler.on_key_down(move |_| {
            let _ = tx_clone.send(Instant::now());
        });

        while !exit_thread.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(50));
        }
    });

    let mut key_times = Vec::new();
    let window_duration = Duration::from_secs(5); // 5 second window for WPM

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        // Collect new key presses
        while let Ok(time) = rx.try_recv() {
            key_times.push(time);
        }

        // Remove old key presses
        let now = Instant::now();
        key_times.retain(|&t| now.duration_since(t) < window_duration);

        // Calculate WPM
        // (keys / 5) / (window_mins)
        let keys = key_times.len() as f32;
        let wpm = (keys / 5.0) / (window_duration.as_secs_f32() / 60.0);

        // Map WPM to color (0 to 120 WPM scale)
        let color = get_wpm_color(wpm);

        let mut final_arr: [u8; 12] = [0; 12];
        for i in 0..4 {
            final_arr[i * 3] = color[0];
            final_arr[i * 3 + 1] = color[1];
            final_arr[i * 3 + 2] = color[2];
        }

        // Apply brightness
        let brightness_mult = match profile.brightness {
            crate::enums::Brightness::Low => 0.5,
            crate::enums::Brightness::High => 1.0,
        };

        for i in 0..12 {
            final_arr[i] = (final_arr[i] as f32 * brightness_mult) as u8;
        }

        manager.keyboard.set_colors_to(&final_arr).unwrap();

        thread::sleep(Duration::from_millis(100));
    }

    kill_thread.store(true, Ordering::SeqCst);
}

fn get_wpm_color(wpm: f32) -> [u8; 3] {
    let capped_wpm = wpm.min(100.0);
    
    if capped_wpm < 40.0 {
        // Blue to Purple/Light Blue
        let factor = capped_wpm / 40.0;
        [ (100.0 * factor) as u8, (100.0 * (1.0 - factor)) as u8, 255]
    } else if capped_wpm < 80.0 {
        // Purple to Orange
        let factor = (capped_wpm - 40.0) / 40.0;
        [ 255, (165.0 * factor) as u8, (255.0 * (1.0 - factor)) as u8]
    } else {
        // Orange to Burning Red
        let factor = (capped_wpm - 80.0) / 20.0;
        [255, (165.0 * (1.0 - factor)) as u8, 0]
    }
}
