use std::time::{Duration, Instant};
// use device_query::Keycode;
use crate::manager::{
    profile::Profile,
    {effects::zones::KEY_ZONES, ActiveRipple, Inner},
};

pub fn update(manager: &mut Inner, p: &Profile) {
    let is_ripple_lit = matches!(p.effect, crate::enums::Effects::RippleLit);
    let ripple_duration = Duration::from_millis(800);
    let propagation_speed = 150.0; // ms per zone (Ripple only)

    // 1. Process new key presses from the global receiver
    while let Ok(key) = manager.key_rx.try_recv() {
        for (i, zone) in KEY_ZONES.iter().enumerate() {
            if zone.contains(&key) {
                manager.ripple_state.push(ActiveRipple {
                    origin_zone: i,
                    start_time: Instant::now(),
                });
                break;
            }
        }
    }

    // 2. Clean up old ripples
    manager.ripple_state.retain(|r| r.start_time.elapsed() < ripple_duration);

    // 3. Calculate frame
    let mut zone_intensities: [f32; 4] = [0.0; 4];
    let now = Instant::now();

    for ripple in &manager.ripple_state {
        let elapsed = now.duration_since(ripple.start_time).as_millis() as f32;
        if is_ripple_lit {
            // True RippleLit:
            // - All zones stay on base color.
            // - Only the pressed/origin zone transitions to secondary/zone color.
            let life_progress = elapsed / 450.0;
            if life_progress < 1.0 {
                let primary_intensity = (-(life_progress - 0.25).powi(2) / 0.08).exp();
                zone_intensities[ripple.origin_zone] = (zone_intensities[ripple.origin_zone] + primary_intensity).min(1.0);
            }
        } else {
            // Ripple: keeps the original expanding wave behavior.
            for zone_idx in 0..4 {
                let distance = (zone_idx as f32 - ripple.origin_zone as f32).abs();
                let propagation_delay = distance * propagation_speed;

                if elapsed > propagation_delay {
                    let time_since_arrival = elapsed - propagation_delay;
                    let life_progress = time_since_arrival / 400.0;

                    if life_progress < 1.0 {
                        let wave_intensity = (-(life_progress - 0.3).powi(2) / 0.1).exp();
                        zone_intensities[zone_idx] = (zone_intensities[zone_idx] + wave_intensity).min(1.0);
                    }
                }
            }
        }
    }

    let rgb_array = p.rgb_array();
    let mut final_arr: [u8; 12] = [0; 12];
    let base_color = if is_ripple_lit { p.base_color } else { [0, 0, 0] };

    for i in 0..4 {
        let intensity = zone_intensities[i];
        let target_color = [rgb_array[i * 3], rgb_array[i * 3 + 1], rgb_array[i * 3 + 2]];

        final_arr[i * 3] = lerp(base_color[0], target_color[0], intensity);
        final_arr[i * 3 + 1] = lerp(base_color[1], target_color[1], intensity);
        final_arr[i * 3 + 2] = lerp(base_color[2], target_color[2], intensity);
    }

    manager.paint(p, &final_arr);
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 * (1.0 - t) + b as f32 * t) as u8
}
