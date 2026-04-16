use std::{
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

use rand::{rng, Rng};
use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let mut rng = rng();
    
    // Each of the 4 zones can have a "raindrop" with a certain intensity
    let mut intensities: [f32; 4] = [0.0; 4];
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let mut final_arr: [u8; 12] = [0; 12];
        
        // Randomly spark a new drop in a zone
        if rng.random_bool(0.3) {
            let zone = rng.random_range(0..4);
            intensities[zone] = 1.0;
        }

        for i in 0..4 {
            let intensity = intensities[i];
            
            // Neon Green color: [0, 255, 0]
            // We'll add a bit of white for the "head" of the drop
            let r = (50.0 * intensity * intensity * intensity) as u8; // Head sparkle
            let g = (255.0 * intensity) as u8;
            let b = (50.0 * intensity * intensity * intensity) as u8;
            
            final_arr[i * 3] = r;
            final_arr[i * 3 + 1] = g;
            final_arr[i * 3 + 2] = b;
            
            // Fade out the intensity
            intensities[i] = (intensities[i] - 0.1).max(0.0);
        }

        apply_brightness_and_send(manager, profile, &final_arr);
        
        // Speed control
        let wait = match profile.effect {
            crate::enums::Effects::Matrix { speed } => (150 / speed.max(1)) as u64,
            _ => 100,
        };
        
        thread::sleep(Duration::from_millis(wait));
    }
}

fn apply_brightness_and_send(manager: &mut Inner, profile: &Profile, arr: &[u8; 12]) {
    let mut final_arr = *arr;
    let brightness_mult = match profile.brightness {
        crate::enums::Brightness::Low => 0.5,
        crate::enums::Brightness::High => 1.0,
    };

    for i in 0..12 {
        final_arr[i] = (final_arr[i] as f32 * brightness_mult) as u8;
    }

    manager.keyboard.set_colors_to(&final_arr).unwrap();
}
