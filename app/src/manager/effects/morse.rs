use std::{sync::atomic::Ordering, thread, time::Duration};

use crate::manager::{profile::Profile, Inner};

pub fn play(manager: &mut Inner, profile: &Profile) {
    let stop_signals = manager.stop_signals.clone();

    // Default message "HELLOL5P" for the Morse code showcase
    let message = "LEGION";
    let morse_sequence = string_to_morse(message);

    let mut seq_idx = 0;

    while !stop_signals.manager_stop_signal.load(Ordering::SeqCst) {
        let (is_on, wait_ms) = match morse_sequence.get(seq_idx) {
            Some(Pulse::Dot) => (true, 200),
            Some(Pulse::Dash) => (true, 600),
            Some(Pulse::Gap) => (false, 200),
            Some(Pulse::LetterSpace) => (false, 600),
            None => {
                seq_idx = 0;
                (false, 2000) // 2s pause before repeating
            }
        };

        let mut final_arr: [u8; 12] = [0; 12];
        if is_on {
            final_arr = profile.rgb_array();
        }

        manager.paint(profile, &final_arr);
        thread::sleep(Duration::from_millis(wait_ms));

        // Always add a gap between pulses if we just showed a pulse
        if is_on {
            let black = [0; 12];
            manager.paint(profile, &black);
            thread::sleep(Duration::from_millis(200));
        }

        seq_idx += 1;
    }
}

#[derive(Clone, Copy)]
enum Pulse {
    Dot,
    Dash,
    Gap,
    LetterSpace,
}

fn string_to_morse(s: &str) -> Vec<Pulse> {
    let mut vec = Vec::new();
    for c in s.to_uppercase().chars() {
        let pulse = match c {
            'A' => vec![Pulse::Dot, Pulse::Dash],
            'B' => vec![Pulse::Dash, Pulse::Dot, Pulse::Dot, Pulse::Dot],
            'C' => vec![Pulse::Dash, Pulse::Dot, Pulse::Dash, Pulse::Dot],
            'D' => vec![Pulse::Dash, Pulse::Dot, Pulse::Dot],
            'E' => vec![Pulse::Dot],
            'F' => vec![Pulse::Dot, Pulse::Dot, Pulse::Dash, Pulse::Dot],
            'G' => vec![Pulse::Dash, Pulse::Dash, Pulse::Dot],
            'H' => vec![Pulse::Dot, Pulse::Dot, Pulse::Dot, Pulse::Dot],
            'I' => vec![Pulse::Dot, Pulse::Dot],
            'J' => vec![Pulse::Dot, Pulse::Dash, Pulse::Dash, Pulse::Dash],
            'K' => vec![Pulse::Dash, Pulse::Dot, Pulse::Dash],
            'L' => vec![Pulse::Dot, Pulse::Dash, Pulse::Dot, Pulse::Dot],
            'M' => vec![Pulse::Dash, Pulse::Dash],
            'N' => vec![Pulse::Dash, Pulse::Dot],
            'O' => vec![Pulse::Dash, Pulse::Dash, Pulse::Dash],
            'P' => vec![Pulse::Dot, Pulse::Dash, Pulse::Dash, Pulse::Dot],
            'Q' => vec![Pulse::Dash, Pulse::Dash, Pulse::Dot, Pulse::Dash],
            'R' => vec![Pulse::Dot, Pulse::Dash, Pulse::Dot],
            'S' => vec![Pulse::Dot, Pulse::Dot, Pulse::Dot],
            'T' => vec![Pulse::Dash],
            'U' => vec![Pulse::Dot, Pulse::Dot, Pulse::Dash],
            'V' => vec![Pulse::Dot, Pulse::Dot, Pulse::Dot, Pulse::Dash],
            'W' => vec![Pulse::Dot, Pulse::Dash, Pulse::Dash],
            'X' => vec![Pulse::Dash, Pulse::Dot, Pulse::Dot, Pulse::Dash],
            'Y' => vec![Pulse::Dash, Pulse::Dot, Pulse::Dash, Pulse::Dash],
            'Z' => vec![Pulse::Dash, Pulse::Dash, Pulse::Dot, Pulse::Dot],
            ' ' => vec![Pulse::LetterSpace],
            _ => vec![],
        };
        for p in pulse {
            vec.push(p);
            vec.push(Pulse::Gap);
        }
        vec.push(Pulse::LetterSpace);
    }
    vec
}
