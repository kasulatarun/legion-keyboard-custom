use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let start = Instant::now();

    while !manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start.elapsed().as_secs_f32();
        let speed = match profile.effect {
            crate::enums::Effects::PrismShift { speed } => speed.max(1) as f32,
            _ => 1.0,
        };

        // Smooth hue flow with slight zone phase offsets.
        let phase = elapsed * 0.35 * speed;
        let mut arr = [0u8; 12];
        for zone in 0..4 {
            let hue = (phase + zone as f32 * 0.23).fract();
            let (r, g, b) = hsv_to_rgb(hue, 0.9, 1.0);
            let idx = zone * 3;
            arr[idx] = r;
            arr[idx + 1] = g;
            arr[idx + 2] = b;
        }

        manager.paint(profile, &arr);
        thread::sleep(Duration::from_millis(33));
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h6 = h * 6.0;
    let i = h6.floor() as i32;
    let f = h6 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
