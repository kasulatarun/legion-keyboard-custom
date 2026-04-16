# Legion RGB Control: Premium Lighting Engine

A finely-tuned, highly-aesthetic RGB controller for Lenovo Legion, Lenovo LOQ, and Ideapad laptops. This software overrides the default hardware lighting presets to deliver ultra-smooth, 60-FPS software-driven lighting effects that run silently in your System Tray.

Based originally on 4JX's [L5P-Keyboard-RGB](https://github.com/4JX/L5P-Keyboard-RGB), this repository has been completely overhauled and engineered to focus on **Premium Aesthetics, Smooth Mathematical Animations, Realistic Physics, and Zero UI Clutter.**

---

## ⚡ Architecture & Custom Rendering Framework

This application abandons the laggy, predefined hardware states of standard controllers in favor of a specialized software engine. 

1. **Static hardware bypass**: The keyboard hardware is forced into a persistent "Static" mode.
2. **The 60-FPS Software Loop**: A dedicated background thread spins up an isolated `16ms` `thread::sleep()` loop. This guarantees we update the 4 distinct zones array (`[u8; 12]`) 60 times a second.
3. **Decoupled Speed from Framerate**: In legacy software solutions, slowing down an effect meant increasing the thread's sleep timer—resulting in horrible, choppy frames. Here, the update tick remains locked at 16ms, and "Speed" simply scales the time-delta variables within the sine/cosine/perlin math algorithms. 

---

## 🎨 The 21 Curated Effects & How They Work

We removed over 20 gimmicky, overlapping, and ugly effects (such as Battery Sync, Pomodoro, and System Monitor) from previous iterations to focus purely on high-fidelity visual fidelity. 

### 🌿 High-Fidelity Nature & Aesthetic Effects

#### 1. SummerRain
**The Visual**: Organic, random blue and cyan pulses that fade at an irregular curve, mimicking the appearance of raindrops sliding down the keyboard.
**The Algorithm**: We maintain an array of `drops[4]`. A weighted RNG decides when a "drop" hits a zone (spiking its float to `1.0`). Instead of a linear fade, we process it into RGB utilizing a quadratic curve `(150.0 * val * val)`, creating an intense initial flash that softly tapers off. We also clamp a bright baseline so the keyboard never turns completely black between rain impacts.

#### 2. AuroraBorealis
**The Visual**: A slow, flowing wave combining deep greens and purples, mimicking the ethereal movement of the northern lights.
**The Algorithm**: We utilize two overlapping sine/cosine waveforms (`phase1` and `phase2`) running at different frequency offsets `(i * 1.25)` across the 4 keyboard zones. The interaction of `val1` and `val2` causes the green and purple thresholds to dynamically overlap and push each other around organically, generating a Perlin-noise style result without heavy computational cost.

#### 3. Candlelight
**The Visual**: A realistic, irregular warm flickering utilizing multiple micro-stages of amber and gold.
**The Algorithm**: Unlike simple sine-wave pulses, this effect uses localized *target interpolation*. We calculate random target brightnesses (`0.3` for heavy flickers, `0.9` for micro-twitches) and smoothly interpolate the current zone's value towards it by `0.3` per tick. Red drops linearly, Green drops quadratically, and Blue drops cubically—meaning as the "candle" dims, it naturally shifts from white-hot to deep orange, perfectly simulating physical thermodynamics.

#### 4. LightLeak
**The Visual**: Simulates cinematic, warm amber "light leaks" across a camera lens, moving smoothly across the zones.
**The Algorithm**: Sine shifts utilizing a very high phase-offset `(i * 1.5)`. The base output is clamped to a minimum warmth `r = 150.0`, preventing any dead zones.

#### 5. StarryNight
**The Visual**: Twinkling white dots over a solid, dark ambient blue background.
**The Algorithm**: Similar stochastic RNG logic to SummerRain, but with a drastically lower spawn chance. The fade out is extremely flat, and the background utilizes a hardcoded dark-blue floor `b = 80.0`.

#### 6. SoftBloom
**The Visual**: A soft, elegant breathing effect where colors seamlessly expand and contract their intensity together.
**The Algorithm**: Amplifies the profile's base RGB array across a smooth cubic curve. We use `bloom_val * bloom_val * 0.6 + 0.4` so the expansion feels "heavy" and slow at its peak rather than continuously rubber-banding. 

#### 7. SunsetGlow
**The Visual**: Slow, luxurious cross-zone shifts between deep reds, purples, and oranges.
**The Algorithm**: A multi-phase cross dissolve that deliberately suppresses Green values while utilizing `clamp(100.0, 255.0)` on Red to maintain a vibrantly lit core.

#### 8. NeonDream
**The Visual**: A vibrant, oscillating gradient flowing between hot Pink, Purple, and Cyan.
**The Algorithm**: Two oscillating phases, identical to the logic in `AuroraBorealis`, but mathematically plotted to cross the boundaries of maximum Blue (`255.0`), causing dynamic "magenta" collisions.

#### 9. CyberPulse
**The Visual**: Rhythmic fast-paced Cyan and Magenta sweeps built for aggressive action gaming and EDM.
**The Algorithm**: A sine wave clamped inside the absolute value function `(phase.sin().abs())` which creates sharp visual "bounces" between Cyan and Magenta without any soft black fades. 

#### 10. VHSRetro
**The Visual**: A specialized retro CRT screen glitch with chromatic aberration and tracking interference.
**The Algorithm**: Rapid, seeded ThreadRng injected straight into the RGB output with a roaming `scan_zone`. Every frame, a "scanline" multiplies its zone's brightness by `1.15`, while non-scan zones drop to `0.9`. Heavy random noise is added to `15%` of the frames to simulate video tracking errors.

### ⚡ Enhanced Classic & Interactive Effects

#### 11. Lightning
**The Upgrade**: Added a realistic "Double-Strike Thunderstorm" algorithm. 
**How It Works**: When an RNG threshold is met, the key zone strikes. We added a micro-pause `thread::sleep(rng(20..80))` and a `70%` probability that the same zone strikes a *second* time weakly before fully decaying.

#### 12. Fire
**The Upgrade**: Introduced localized "Embers".
**How It Works**: The baseline algorithm is a standard flame math offset, but we added a `0.05%` chance per frame for an "ember" to pop, instantly shooting that zone to maximum Bright Yellow/White for a split-second.

#### 13. OceanWave
**The Upgrade**: Deep blues and cyans enriched by a high-frequency "Shimmer" layer.
**How It Works**: An incredibly fast secondary sine wave `fast_sine` is appended to the top of the slow, rolling primary wave, visually simulating sunlight rapidly reflecting off undulating water.

#### 14. Meteor
**The Upgrade**: Exponential visual falloff trails.
**How It Works**: Instead of a linear sweep, the trail of the meteor falls off on an exponential curve `x^3`, meaning the head of the comet is blindingly bright, but the tail cleanly vanishes without washing out the rest of the board.

#### 15. Heartbeat
**The Concept**: An anatomically accurate double-pulse.
**How It Works**: A time-duration matrix mapped into `0-400ms: LUB (large sine burst)`, `400-500ms: Pause`, `500-800ms: DUB (small sine burst)`. Only operates on the center two LED zones for an optical illusion of a beating core.

#### 16. AudioVisualizer
Reacts to the Windows CoreAudio API. Fast fourier transforms monitor system volume and slam the intensity onto the keyboard.

#### 17. AmbientLight
Pure immersion—the desktop screen is screengrabbed via GDI at low-resolution every tick, and the dominant colors are extracted and pushed natively to the 4 keyboard zones.

#### 18. SmoothWave
A high-fidelity software alternative to the choppy hardware preset wave, operating on the synced 16ms loop.

#### 19. PrismShift
A continuously, endlessly flowing rainbow prism built on phase-shifted hue-saturation mappings (HSV to RGB).

#### 20. Ripple / 21. RippleLit
Listens to global device-level keyboard hooks. Upon any keypress event, it launches an instantaneous shockwave of color that originates at the pressed zone and mathematically decays outward.

---

## 🛠 Building & Deployment

Requires `rustc` and `cargo` installed.

1. Clone the repository:
   ```sh
   git clone https://github.com/KnightroParth/legion-keyboard-custom.git
   ```

2. Build using cargo (Building Release is critical to prevent math-framerate lag!):
   ```sh
   cd legion-keyboard-custom
   cargo build --release
   ```

3. **Running the Application**:
   Navigate to `target/release/` and run `legion-kb-rgb.exe`. It will request UAC privileges (Required to communicate with USB HID), launch the GUI, apply the first effect, and plant itself in your System Tray!

## Compatibility
Tested perfectly on:
- Legion 5 (Pro) 2020-2024
- Ideapad Gaming 3 2021-2024
- LOQ 2023-2025

*(Note: Legion 7(i) models with per-key RGB or true-white backlights are currently not supported.)*
