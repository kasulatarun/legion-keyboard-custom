use std::{sync::atomic::Ordering, thread, time::Duration};

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();

    let mut sys = System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()).with_memory(MemoryRefreshKind::everything()));

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let cpu_usage = sys.global_cpu_usage(); // 0..100
        let mem_usage = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;

        let mut final_arr: [u8; 12] = [0; 12];

        // Zones 0 & 1: CPU
        let cpu_color = get_usage_color(cpu_usage);
        final_arr[0..6].copy_from_slice(&[cpu_color[0], cpu_color[1], cpu_color[2], cpu_color[0], cpu_color[1], cpu_color[2]]);

        // Zones 2 & 3: RAM
        let mem_color = get_usage_color(mem_usage);
        final_arr[6..12].copy_from_slice(&[mem_color[0], mem_color[1], mem_color[2], mem_color[0], mem_color[1], mem_color[2]]);

        // Apply brightness
        let brightness_mult = match profile.brightness {
            crate::enums::Brightness::Low => 0.5,
            crate::enums::Brightness::High => 1.0,
        };

        for i in 0..12 {
            final_arr[i] = (final_arr[i] as f32 * brightness_mult) as u8;
        }

        manager.keyboard.set_colors_to(&final_arr).unwrap();

        thread::sleep(Duration::from_millis(500)); // Refresh every 500ms
    }
}

fn get_usage_color(usage: f32) -> [u8; 3] {
    if usage < 50.0 {
        // Green to Yellow: [0, 255, 0] -> [255, 255, 0]
        let factor = usage / 50.0;
        [(255.0 * factor) as u8, 255, 0]
    } else {
        // Yellow to Red: [255, 255, 0] -> [255, 0, 0]
        let factor = (usage - 50.0) / 50.0;
        [255, (255.0 * (1.0 - factor)) as u8, 0]
    }
}
