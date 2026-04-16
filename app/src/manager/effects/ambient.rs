use std::{ptr::null_mut, sync::atomic::Ordering, thread, time::Duration};

use winapi::um::wingdi::{GetDeviceCaps, GetPixel, HORZRES, VERTRES};
use winapi::um::winuser::{GetDC, ReleaseDC};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();

    let (fps, vibrance) = match profile.effect {
        crate::enums::Effects::AmbientLight { fps, vibrance } => (fps, vibrance),
        _ => (60, 100),
    };

    let frame_duration = Duration::from_millis(1000 / fps.max(1) as u64);
    let vibrance_mult = vibrance as f32 / 100.0;

    unsafe {
        let hdc_screen = GetDC(null_mut());
        if hdc_screen.is_null() {
            return;
        }

        let width = GetDeviceCaps(hdc_screen, HORZRES);
        let height = GetDeviceCaps(hdc_screen, VERTRES);

        let mut smoothed_arr: [f32; 12] = [0.0; 12];

        while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            let mut target_arr: [f32; 12] = [0.0; 12];

            for zone in 0..4 {
                let start_x = (width / 4) * zone;
                let end_x = (width / 4) * (zone + 1);

                let mut r_total: u64 = 0;
                let mut g_total: u64 = 0;
                let mut b_total: u64 = 0;
                let sample_count = 9;

                for row in 0..3 {
                    for col in 0..3 {
                        let x = start_x + ((end_x - start_x) / 4) * (col + 1);
                        let y = (height / 4) * (row + 1);

                        let color = GetPixel(hdc_screen, x, y);
                        r_total += (color & 0xFF) as u64;
                        g_total += ((color >> 8) & 0xFF) as u64;
                        b_total += ((color >> 16) & 0xFF) as u64;
                    }
                }

                // Average
                let mut r = (r_total / sample_count) as f32;
                let mut g = (g_total / sample_count) as f32;
                let mut b = (b_total / sample_count) as f32;

                // Apply Vibrance
                let avg = (r + g + b) / 3.0;
                r = ((r - avg) * vibrance_mult + avg).clamp(0.0, 255.0);
                g = ((g - avg) * vibrance_mult + avg).clamp(0.0, 255.0);
                b = ((b - avg) * vibrance_mult + avg).clamp(0.0, 255.0);

                target_arr[zone as usize * 3] = r;
                target_arr[zone as usize * 3 + 1] = g;
                target_arr[zone as usize * 3 + 2] = b;
            }

            // Lerp Smoothing (20% per frame)
            for i in 0..12 {
                smoothed_arr[i] += (target_arr[i] - smoothed_arr[i]) * 0.2;
            }

            let mut final_arr: [u8; 12] = [0; 12];
            for i in 0..12 {
                final_arr[i] = smoothed_arr[i] as u8;
            }

            manager.paint(profile, &final_arr);
            thread::sleep(frame_duration);
        }

        ReleaseDC(null_mut(), hdc_screen);
    }
}
