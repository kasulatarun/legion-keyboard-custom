use std::{
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

use crate::{
    enums::EmergencyType,
    manager::{profile::Profile, Inner},
};

pub fn play(manager: &mut Inner, profile: &Profile, emergency_type: EmergencyType) {
    let stop_signals = manager.stop_signals.clone();
    
    // speed is 1..=4 (clamped in manager)
    // We'll use it to scale the strobe frequency
    let speed = profile.speed.max(1) as u64;
    let delay = Duration::from_millis(500 / speed);

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        match emergency_type {
            EmergencyType::Police => {
                // Alternating Red (Left) and Blue (Right)
                let mut red_left = [0u8; 12];
                // Zone 0 & 1: Red
                red_left[0] = 255; red_left[3] = 255;
                
                let mut blue_right = [0u8; 12];
                // Zone 2 & 3: Blue
                blue_right[8] = 255; blue_right[11] = 255;

                manager.keyboard.set_colors_to(&red_left).unwrap();
                thread::sleep(delay / 2);
                
                if stop_signals.manager_stop_signal.load(Ordering::SeqCst) { break; }
                
                manager.keyboard.set_colors_to(&blue_right).unwrap();
                thread::sleep(delay / 2);
            }
            EmergencyType::Ambulance => {
                // High frequency pulsing White and Red
                let white = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
                let red = [255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0];

                manager.keyboard.set_colors_to(&white).unwrap();
                thread::sleep(delay / 4);
                
                if stop_signals.manager_stop_signal.load(Ordering::SeqCst) { break; }
                
                manager.keyboard.set_colors_to(&red).unwrap();
                thread::sleep(delay / 4);
            }
        }
    }
}
