use std::{sync::atomic::Ordering, thread, time::Duration};

use sysinfo::{Components, RefreshKind, System};

use crate::manager::Inner;

pub fn play_cpu(manager: &mut Inner) {
    let mut sys = System::new_with_specifics(RefreshKind::new().with_cpu());

    let blue = [0, 0, 255];
    let green = [0, 255, 0];
    let yellow = [255, 255, 0];
    let red = [255, 0, 0];

    loop {
        sys.refresh_cpu();
        let usage = sys.global_cpu_info().cpu_usage();

        // CPU Usage thresholds:
        // < 40: Blue
        // 40 - 70: Green
        // 70 - 80: Yellow
        // > 80: Red

        let color = if usage < 40.0 {
            blue
        } else if usage < 70.0 {
            green
        } else if usage < 81.0 {
            yellow
        } else {
            red
        };

        // Transition to the color
        let target = [
            color[0], color[1], color[2],
            color[0], color[1], color[2],
            color[0], color[1], color[2],
            color[0], color[1], color[2],
        ];

        if let Err(_) = manager.keyboard.transition_colors_to(&target, 20, 0) {
           break;
        }

        if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            break;
        }

        thread::sleep(Duration::from_millis(1000));
    }
}

pub fn play_gpu(manager: &mut Inner) {
    // GPU Temp thresholds:
    // < 45: Blue
    // 45 - 55: Green
    // 55 - 65(?): Yellow
    // > 65(?): Red (User said "hot area" starting 55, so let's make 55 yellow and maybe 65 red, or strictly follow "approaching hot")
    
    // User Instructions: 
    // "GPU temperature low (below 45 Degrees C) it will be Blue, medium (45 to 55) will be green then yellow as it approaches the hot area and hot (55 degrees C and above) red"
    // Interpretation:
    // < 45: Blue
    // 45 - 55: Green
    // 55 - 65: Yellow (Transition zone)
    // > 65: Red
    // Or strictly: "hot (55 degrees C and above) red".
    // "yellow as it approaches the hot area" -> this implies the range before 55 is yellow?
    // "medium (45 to 55) will be green then yellow as it approaches the hot area"
    // So 45-50 Green, 50-55 Yellow, 55+ Red. Let's try that.

    let blue = [0, 0, 255];
    let green = [0, 255, 0];
    let yellow = [255, 255, 0];
    let red = [255, 0, 0];

    let mut components = Components::new_with_refreshed_list();

    loop {
        components.refresh(true);
        
        // Try to find a GPU component. Common names include "GPU", "edge", "junction", "mem".
        // On Windows via sysinfo, it might show up differently.
        // We will look for anything containing "GPU" regardless of case.
        let mut found_temp = None;
        
        for component in &components {
            let label = component.label().to_uppercase();
            if label.contains("GPU") {
                 let temp = component.temperature();
                 if let Some(t) = temp {
                     found_temp = Some(t);
                     break;
                 }
            }
        }

        if let Some(temp) = found_temp {
            let color = if temp < 45.0 {
                blue
            } else if temp < 50.0 {
                green
            } else if temp < 55.0 {
                yellow
            } else {
                red
            };

            let target = [
                color[0], color[1], color[2],
                color[0], color[1], color[2],
                color[0], color[1], color[2],
                color[0], color[1], color[2],
            ];
             if let Err(_) = manager.keyboard.transition_colors_to(&target, 20, 0) {
               break;
            }
        } else {
            // GPU not found, maybe flash red/white to indicate error? Or just stay static.
            // Let's hold current color or default to Blue.
             let target = [
                0, 0, 255,
                0, 0, 255,
                0, 0, 255,
                0, 0, 255,
            ];
            manager.keyboard.transition_colors_to(&target, 20, 0).ok();
        }

        if manager.stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
            break;
        }

        thread::sleep(Duration::from_millis(1000));
    }
}
