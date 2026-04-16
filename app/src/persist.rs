use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::enums::AutomationRule;
use crate::manager::{custom_effect::CustomEffect, profile::Profile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    pub profiles: Vec<Profile>,
    pub effects: Vec<CustomEffect>,
    #[serde(alias = "ui_state")]
    pub current_profile: Profile,
    #[serde(default)]
    pub master_off: bool,
    #[serde(default)]
    pub automation_rules: Vec<AutomationRule>,
}

impl Settings {
    pub fn new(profiles: Vec<Profile>, effects: Vec<CustomEffect>, current_profile: Profile, master_off: bool, automation_rules: Vec<AutomationRule>) -> Self {
        Self {
            profiles,
            effects,
            current_profile,
            master_off,
            automation_rules,
        }
    }

    /// Load the settings from the configured path or generate default ones if an error occurs
    pub fn load() -> Self {
        let mut persist: Self = Self::default();

        if let Ok(string) = fs::read_to_string(Self::get_location()) {
            persist = serde_json::from_str(&string).unwrap_or_default();
        }

        persist
    }

    /// Save the settings to the configured path
    pub fn save(&mut self) {
        match File::create(Self::get_location()) {
            Ok(mut file) => match serde_json::to_string(&self) {
                Ok(stringified_json) => {
                    if let Err(e) = file.write_all(stringified_json.as_bytes()) {
                        eprintln!("🔴 FAILED TO SAVE SETTINGS: {}", e);
                    }
                }
                Err(e) => eprintln!("🔴 FAILED TO SERIALIZE SETTINGS: {}", e),
            },
            Err(e) => eprintln!("🔴 FAILED TO ACCESS SETTINGS FILE: {}", e),
        }
    }

    fn get_location() -> PathBuf {
        let mut path = if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            let mut p = PathBuf::from(local_app_data);
            p.push("legion-kb-rgb");
            let _ = fs::create_dir_all(&p);
            p.push("settings.json");
            p
        } else {
            PathBuf::from("./settings.json")
        };

        if let Ok(maybe_path) = env::var("LEGION_KEYBOARD_CONFIG") {
            let env_path = PathBuf::from(maybe_path);
            if env_path.exists() && env_path.is_file() {
                path = env_path;
            }
        }

        path
    }
}
