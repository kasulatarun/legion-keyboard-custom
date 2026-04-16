use crate::manager::{profile::Profile, Inner};
use std::time::Instant;

pub fn update(manager: &mut Inner, p: &Profile) {
    if manager.heartbeat_start.is_none() {
        manager.heartbeat_start = Some(Instant::now());
    }

    let elapsed = manager.heartbeat_start.unwrap().elapsed().as_millis() % 2000;

    // Heartbeat rhythm logic:
    // 0-400ms: LUB (Big Pulse)
    // 400-500ms: Pause
    // 500-800ms: DUB (Small Pulse)
    // 800-2000ms: Rest

    let intensity = if elapsed < 400 {
        // LUB pulse
        let t = (elapsed as f32 / 400.0) * std::f32::consts::PI;
        t.sin().powi(2) * 1.0
    } else if (500..800).contains(&elapsed) {
        // DUB pulse
        let t = ((elapsed - 500) as f32 / 300.0) * std::f32::consts::PI;
        t.sin().powi(2) * 0.6
    } else {
        0.0
    };

    // Primary color comes from the profile center zones, secondary uses base_color
    // (similar to RippleLit's primary/base split).
    let heart_color_left = p.rgb_zones[1].rgb;
    let heart_color_right = p.rgb_zones[2].rgb;
    let bg_color = p.base_color;

    let mut final_arr: [u8; 12] = [0; 12];
    for i in 0..4 {
        let color = if i == 1 {
            lerp_color(bg_color, heart_color_left, intensity)
        } else if i == 2 {
            lerp_color(bg_color, heart_color_right, intensity)
        } else {
            // Keep side zones stable to preserve the center optical illusion.
            bg_color
        };

        final_arr[i * 3] = color[0];
        final_arr[i * 3 + 1] = color[1];
        final_arr[i * 3 + 2] = color[2];
    }

    manager.paint(p, &final_arr);
}

fn lerp_color(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [
        (a[0] as f32 * (1.0 - t) + b[0] as f32 * t) as u8,
        (a[1] as f32 * (1.0 - t) + b[1] as f32 * t) as u8,
        (a[2] as f32 * (1.0 - t) + b[2] as f32 * t) as u8,
    ]
}
