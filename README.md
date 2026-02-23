# Legion RGB Control

Legion RGB Control is a tool for controlling the RGB backlight on Lenovo Legion, Lenovo LOQ and Ideapad laptops.

## Download
Builds are available on the [releases tab](https://github.com/KnightroParth/legion-keyboard-custom/releases).
Based on 4JX's [L5P-Keyboard-RGB](https://github.com/4JX/L5P-Keyboard-RGB)

## Available Effects
- **Static**: Selected colors stay constant.
- **Breath**: Pulses with selected colors.
- **Smooth**: Smooth transitions between colors.
- **Wave (Left/Right)**: Classic wave effect.
- **Lightning**: Random flashes of light.
- **AmbientLight**: Reacts to screen content.
- **SmoothWave**: Smooth wave implementation.
- **Swipe**: Transitions colors from side to side.
- **Ripple / RippleLit**: Interactive ripple effects.
- **Audio Visualizer**: Reacts to system audio.
- **Frequency Visualizer**: High-detail audio visualizer.
- **System Monitor**: Displays CPU/RAM usage.
- **WPM Heat Map**: Displays typing speed.
- **Fire / Ocean Wave / Meteor**: Environmental effects.
- **Pomodoro**: Productivity timer.

## Usage
### Command Line Interface
```sh
# Getting help
legion-kb-rgb --help

# Setting red color
legion-kb-rgb set -e Static -c 255,0,0,255,0,0,255,0,0,255,0,0

# Smooth Wave to the left
legion-kb-rgb set -e SmoothWave -s 4 -b 2 -d Left
```

## Compatibility
Tested on:
- Legion 5 (Pro) 2020-2024
- Ideapad Gaming 3 2021-2024
- LOQ 2023-2025

**Note:** Legion 7(i) and all models with white-only backlights are currently not supported.

## Building from Source
Requires Rust and Git.

1. Clone the repository:
   ```sh
   git clone https://github.com/KnightroParth/legion-keyboard-custom.git
   ```

2. Build using cargo:
   ```sh
   cargo build --release
   ```
