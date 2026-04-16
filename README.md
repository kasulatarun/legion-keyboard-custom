# Legion RGB Control

A finely-tuned, highly-aesthetic RGB controller for Lenovo Legion, Lenovo LOQ, and Ideapad laptops. This controller overrides the default hardware lighting to deliver ultra-smooth, 60-FPS software-driven lighting effects that run silently in your System Tray.

Based originally on 4JX's [L5P-Keyboard-RGB](https://github.com/4JX/L5P-Keyboard-RGB) but completely overhauled to focus on **Premium Aesthetics, Smooth 60FPS Animations, and Zero UI Clutter.**

---

## 🎨 The 21 Curated Effects

We removed over 20 gimmicky, overlapping, and ugly effects from previous iterations. The remaining 21 effects have been painstakingly optimized for proper physics algorithms, high baseline brightness (no more "dead black" flickering), and butter-smooth performance.

### Premium Aesthetic & Nature Effects (Brand New)
- **NeonDream**: A vibrant, oscillating gradient flowing between Pink, Purple, and Cyan.
- **LightLeak**: Simulates organic, cinematic amber light leaks moving smoothly across the keys.
- **SummerRain**: Soft, random blue/cyan pulses that fade organically, like raindrops sliding down glass.
- **AuroraBorealis**: A Perlin-noise style ethereal wave combining deep greens and purples.
- **Candlelight**: A realistic, irregular warm flickering utilizing multiple micro-stages of amber and gold.
- **StarryNight**: Twinkling white dots over a solid, dark ambient background.
- **SoftBloom**: A soft breathing effect where colors elegantly expand and contract their intensity.
- **CyberPulse**: Rhythmic fast-paced Cyan and Magenta sweeps built for action framing.
- **VHSRetro**: A specialized retro screen-glitch with color separation (chromatic aberration) and scanlines.
- **SunsetGlow**: Slow, luxurious shifts between deep reds, purples, and oranges.

### Enhanced Classic & Interactive Effects
- **Lightning**: Features a modernized "double-strike" thunderstorm mechanic where zones flash sequentially before fading out realistically.
- **SmoothWave**: A high-fidelity, ultra-smooth RGB wave replacing the older, stuttery preset.
- **Ripple**: Highly responsive! Pressed zones organically light up and fade outward.
- **RippleLit**: A core base color stays dominant, while key presses overlay a fast secondary color ripple.
- **AudioVisualizer**: Dynamically reacts to the overall volume of your system audio.
- **Fire**: Completely overhauled with simulated "embers" (high-intensity sparks) stacked over a standard flame algorithm.
- **OceanWave**: Deep blues and cyans enriched by a high-frequency "shimmer" layer to visually simulate sunlight reflecting off water.
- **Meteor**: Uses an exponential depth-fade so the comet trail falls off realistically without turning the rest of your board dark.
- **AmbientLight**: Pure immersion—matches your keyboard zones natively to what is rendering on your screen.
- **Heartbeat**: The Pulse! Employs an anatomically realistic "lub-dub" double-beat rhythm from the center zones outward.
- **PrismShift**: A continuously flowing rainbow prism with flawless zone blending.

---

## ⚙️ How It Works (System Tray & Background Threading)

The application functions via a unified frontend/backend split designed to run seamlessly in the background.

**The GUI & System Tray (`tray.rs`)**  
When executed, the program asks for elevated privileges (needed to write directly to USB HID devices) and sits quietly in the Windows System Tray (Taskbar). This allows you to open settings on the fly. We use `egui` to render a minimalist and highly responsive graphical interface. If the window is minimized or closed, the program gracefully hides itself, only showing up as a taskbar icon.

**The Lighting Engine Loop (`manager/mod.rs`)**  
The core of this application isn't the hardware—it's a custom multi-threaded software manager. When you select an effect:
1. The manager puts the keyboard hardware into "Static" configuration mode.
2. An isolated background thread runs a specialized `16ms` ticks loop (aiming for buttery `60 FPS`).
3. Mathematical formulas (from `manager/effects/*.rs`) rapidly calculate colors (clamped manually so it never turns ugly!) and stream them natively to the 4 RGB zones.

---

## 🚀 Recent Optimizations (Why It Feels So Fast)

If you've used previous versions, this update feels drastically better. Here is what we changed:
1. **The Great De-cluttering**: Eradicated 20+ low-value effects (like SystemMonitor, BioRhythms, MorseCode, BatterySync) that severely bloated the dropdown.
2. **True 60 FPS**: Standardized thread sleeping dynamically inside effects. Previously, slowing the speed slider just increased the `thread::sleep()` time, making the keyboard look laggy. Now the math slows down, but the output frames stay consistently smooth.
3. **No "Missing" Colors**: We heavily clamped scaling multipliers (e.g. `val.max(0.4)`) so dim effects like `Candlelight` and `SoftBloom` leave beautiful baseline gradients rather than shutting zones entirely off.

---

## 🛠 Building & Deployment

Requires `rustc` and `cargo` installed.

1. Clone the repository:
   ```sh
   git clone https://github.com/KnightroParth/legion-keyboard-custom.git
   ```

2. Build using cargo (Building Release is critical for GUI and math performance!):
   ```sh
   cd legion-keyboard-custom
   cargo build --release
   ```

3. **Running the Application**:
   Navigate to `target/release/` and run `legion-kb-rgb.exe`. It will launch the GUI, apply the first effect, and plant itself in your System Tray!

## Compatibility
Tested perfectly on:
- Legion 5 (Pro) 2020-2024
- Ideapad Gaming 3 2021-2024
- LOQ 2023-2025

*(Note: Legion 7(i) models with per-key RGB or true-white backlights are currently not supported.)*
