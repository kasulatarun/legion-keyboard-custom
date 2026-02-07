# Legion RGB Control

**⚠️ Use at your own risk, the developer is not responsible for any damages that may arise as a result of using this program.**

Legion RGB Control is a tool for controlling the RGB backlight on Lenovo Legion and Ideapad laptops.

## Download
Builds are available on the [releases tab](https://github.com/KnightroParth/legion-keyboard-custom/releases).

## Available Effects
- **Static**: Selected colors stay constant.
- **Breath**: Pulses with selected colors.
- **Smooth**: Smooth transitions between colors.
- **Wave (Left/Right)**: Classic wave effect.
- **Lightning**: Random flashes of light.
- **AmbientLight**: Reacts to screen content.
- **Disco**: Party mode!
- **Christmas**: Festive colors.
- **Fade**: Dims lights after inactivity.
- **Temperature**: Based on CPU temperature (Linux only).

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

**Note:** Legion 7(i) and models with white-only backlights are currently not supported.

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

---
Special thanks to legendk95 for the initial reverse engineering work.
