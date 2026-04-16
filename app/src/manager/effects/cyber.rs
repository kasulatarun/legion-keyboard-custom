use std::{sync::atomic::Ordering, thread, time::{Duration, Instant}};
use crate::manager::{profile::Profile, Inner};

pub fn play_cyber_pulse(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();
    let start_time = Instant::now();
    
    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let elapsed = start_time.elapsed().as_secs_f32();
        let speed_mult = profile.speed as f32 * 1.5; // Fast-paced but optimized
        
        let mut final_arr: [u8; 12] = [0; 12];
        
        let global_pulse = (elapsed * speed_mult).sin(); 
        
        for i in 0..4 {
            let local_pulse = ((elapsed * speed_mult) + i as f32).cos();
            let combined = ((global_pulse + local_pulse) / 2.0).abs();
            
            // Cyan edge, Magenta core
            let is_cyan = i % 2 == 0;
            
            if is_cyan {
                final_arr[i*3] = 0;
                final_arr[i*3+1] = (255.0 * combined) as u8;
                final_arr[i*3+2] = (255.0 * combined) as u8;
            } else {
                final_arr[i*3] = (255.0 * combined) as u8;
                final_arr[i*3+1] = 0;
                final_arr[i*3+2] = (255.0 * combined) as u8;
            }
        }
        
        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(16));
    }
}
