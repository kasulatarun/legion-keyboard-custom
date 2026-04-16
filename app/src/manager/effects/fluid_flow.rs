use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::manager::{
    profile::Profile,
    {effects::zones::KEY_ZONES, Inner},
};
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

    // Simple fluid simulation: 4 buckets with "pressure"
    let mut zone_pressure: [f32; 4] = [0.0; 4];

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        // 1. Add pressure from keypresses
        while let Ok(key) = rx.try_recv() {
            for (i, zone) in KEY_ZONES.iter().enumerate() {
                if zone.contains(&key) {
                    zone_pressure[i] = (zone_pressure[i] + 0.4).min(1.2);
                }
            }
        }

        // 2. Diffuse pressure (Fluid Flow)
        let mut new_pressure = zone_pressure;
        for i in 0..4 {
            let diffusion_rate = 0.1;
            if i > 0 {
                let flow = zone_pressure[i] * diffusion_rate;
                new_pressure[i] -= flow;
                new_pressure[i - 1] += flow;
            }
            if i < 3 {
                let flow = zone_pressure[i] * diffusion_rate;
                new_pressure[i] -= flow;
                new_pressure[i + 1] += flow;
            }
        }

        // 3. Evaporate
        for i in 0..4 {
            zone_pressure[i] = (new_pressure[i] * 0.92).max(0.0);
        }

        let mut final_arr: [u8; 12] = [0; 12];
        let rgb_array = profile.rgb_array();

        for i in 0..4 {
            // Brighter, clearer fluid look with gentle ambient floor.
            let speed_boost = 1.15 + (profile.speed as f32 / 10.0) * 0.35;
            let pressure_intensity = (zone_pressure[i].min(1.2) * speed_boost).min(1.0);
            let visual_intensity = pressure_intensity.powf(0.8).max(0.14);
            final_arr[i * 3] = (rgb_array[i * 3] as f32 * visual_intensity) as u8;
            final_arr[i * 3 + 1] = (rgb_array[i * 3 + 1] as f32 * visual_intensity) as u8;
            final_arr[i * 3 + 2] = (rgb_array[i * 3 + 2] as f32 * visual_intensity) as u8;
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(33));
    }

    kill_thread.store(true, Ordering::SeqCst);
}
