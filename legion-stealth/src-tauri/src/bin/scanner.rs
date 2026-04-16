use hidapi::{HidApi, HidDevice};
use std::time::Duration;
use std::thread;

const LEGACY_PIDS: [u16; 7] = [0xc993, 0xc996, 0xc995, 0xc994, 0xc985, 0xc981, 0xc693];

#[derive(Debug, Clone, Copy, PartialEq)]
enum WriteMethod {
    FeatureReport(u8),
    OutputReport(u8),
}

impl WriteMethod {
    fn report_id(self) -> u8 {
        match self {
            WriteMethod::FeatureReport(id) | WriteMethod::OutputReport(id) => id,
        }
    }
    fn label(self) -> String {
        match self {
            WriteMethod::FeatureReport(id) => format!("feature 0x{id:02X}"),
            WriteMethod::OutputReport(id) => format!("output 0x{id:02X}"),
        }
    }
}

fn send_legacy_packet(
    device: &HidDevice,
    method: WriteMethod,
    mode: u8,
    speed: u8,
    brightness: u8,
) -> Result<(), String> {
    let mut buf = [0u8; 33];
    buf[0] = method.report_id();
    buf[1] = 0x16;
    buf[2] = mode;
    buf[3] = speed;
    buf[4] = brightness;
    
    // Fill with white for zones 1-4
    for i in 0..4 {
        let base = 5 + i * 3;
        buf[base] = 255;
        buf[base+1] = 255;
        buf[base+2] = 255;
    }

    match method {
        WriteMethod::FeatureReport(_) => device
            .send_feature_report(&buf)
            .map(|_| ())
            .map_err(|err| err.to_string()),
        WriteMethod::OutputReport(_) => device
            .write(&buf)
            .map(|_| ())
            .map_err(|err| err.to_string()),
    }
}

fn main() {
    println!("--- LEGION STEALTH BRUTE FORCE SCAN ---");
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            println!("Error initializing HidApi: {}", e);
            return;
        }
    };

    let methods = [
        WriteMethod::FeatureReport(0xCC),
        WriteMethod::FeatureReport(0x07),
        WriteMethod::FeatureReport(0x00),
        WriteMethod::OutputReport(0x01),
        WriteMethod::OutputReport(0x00),
        WriteMethod::OutputReport(0xCC),
    ];
    let modes = [0x01, 0x03, 0x04, 0x06];
    let mut found = false;
    for candidate in api.device_list() {
        if candidate.vendor_id() != 0x048d || !LEGACY_PIDS.contains(&candidate.product_id()) {
            continue;
        }
        found = true;
        
        println!("\nTesting Candidate: VID 0x{:04x} PID 0x{:04x} IF {} USAGE 0x{:04x}",
            candidate.vendor_id(), candidate.product_id(), candidate.interface_number(), candidate.usage_page());

        let device = match candidate.open_device(&api) {
            Ok(d) => d,
            Err(e) => {
                println!("  [!] Failed to open: {}", e);
                continue;
            }
        };

        let mut successes = 0;
        for method in methods {
            for mode in modes {
                if send_legacy_packet(&device, method, mode, 0x01, 0x01).is_ok() {
                    successes += 1;
                    println!("  [OK] {} mode=0x{:02X}", method.label(), mode);
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
        println!("  Total Successful Writes: {}", successes);
    }

    if !found { println!("\n[!] No Lenovo / ITE HID candidates found."); }
    println!("\n--- SCAN COMPLETE ---");
}
