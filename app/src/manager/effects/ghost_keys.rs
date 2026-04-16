use crate::manager::{
    profile::Profile,
    {effects::zones::KEY_ZONES, Inner},
};

pub fn update(manager: &mut Inner, profile: &Profile) {
    // 1. Process new key presses
    while let Ok(key) = manager.key_rx.try_recv() {
        for (i, zone) in KEY_ZONES.iter().enumerate() {
            if zone.contains(&key) {
                // Immediate hit on the zone
                let current = manager.ghost_key_state.get(&i).cloned().unwrap_or(0.0);
                manager.ghost_key_state.insert(i, (current + 0.8).min(1.0));

                // Subtle predictive glow on neighbors
                if i > 0 {
                    let n = manager.ghost_key_state.get(&(i - 1)).cloned().unwrap_or(0.0);
                    manager.ghost_key_state.insert(i - 1, (n + 0.2).min(0.6));
                }
                if i < 3 {
                    let n = manager.ghost_key_state.get(&(i + 1)).cloned().unwrap_or(0.0);
                    manager.ghost_key_state.insert(i + 1, (n + 0.2).min(0.6));
                }
            }
        }
    }

    // 2. Exponential organic decay
    for i in 0..4 {
        let current = manager.ghost_key_state.get(&i).cloned().unwrap_or(0.0);
        manager.ghost_key_state.insert(i, (current * 0.92).max(0.0));
    }

    // 3. Render frame
    let mut final_arr: [u8; 12] = [0; 12];
    let rgb_array = profile.rgb_array();

    for i in 0..4 {
        let intensity = manager.ghost_key_state.get(&i).cloned().unwrap_or(0.0);

        final_arr[i * 3] = (rgb_array[i * 3] as f32 * intensity) as u8;
        final_arr[i * 3 + 1] = (rgb_array[i * 3 + 1] as f32 * intensity) as u8;
        final_arr[i * 3 + 2] = (rgb_array[i * 3 + 2] as f32 * intensity) as u8;
    }

    manager.paint(profile, &final_arr);
}
