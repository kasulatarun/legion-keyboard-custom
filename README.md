# 🌌 Legion RGB // CYBERNETIC LIGHT ENGINE v2.0

[![Rust](https://img.shields.io/badge/rust-1.80%2B-ffc107?style=for-the-badge&logo=rust)](https://rust-lang.org) [![eframe](https://img.shields.io/badge/egui_eframe-0.28-00d4aa?style=for-the-badge&logo=egui)](https://egui.rs) [![HID](https://img.shields.io/badge/HID-Driver-fb9300?style=for-the-badge&logo=usb)](https://usb.org)

**60FPS physics-driven RGB for Legion/LOQ/Ideapad.**

### 🚀 INSTANT START (EXE)

1. **Download** `legion-kb-rgb.exe` from Releases or build:
   ```
   git clone https://github.com/KnightroParth/legion-keyboard-custom
   cd legion-keyboard-custom
   cargo build --release
   .\target\release\legion-kb-rgb.exe  # Admin
   ```
2. GUI loads → Effect #1 active → Tray icon.
3. Right-click tray → Select effects (50+ total).
4. **EXE in**: `target/release/legion-kb-rgb.exe` or `dist/windows-release/`.

`[Neo] Rigid OEM → Quantum wavefronts.`

## 🔮 ALL EFFECTS LISTED HERE (50+ in app/src/manager/effects/)

**Core Effects (enums.rs - 21)**:
1. Lightning (RNG double-strike gaussian)
2. SmoothWave (rotation lerp Fill/Change)
3. Ripple (keypress shockwave propagation)
4. RippleLit (base + pressed primary)
5. AudioVisualizer (CPAL RMS beat staccato)
6. Fire (thermal + ember RNG)
7. OceanWave (sine + shimmer)
8. Meteor (bouncing x³ trail)
9. AmbientLight (GDI screen HSV, FPS/Vibrance)
10. Heartbeat (800ms LUB-DUB sin² center)
11. PrismShift (HSV fract rainbow)
12. LightLeak (amber sine i*1.5)
13. VHSRetro (chroma scan RNG15%)
14. NeonDream (pink/purple/cyan collision)
15. SummerRain (quadratic drop RNG5%)
16. AuroraBorealis (dual-sine Perlin)
17. Candlelight (flicker cubic interp)
18. CyberPulse (sin.abs cyan/magenta)
19. StarryNight (twinkle RNG2% bg)
20. SoftBloom (bloom²*0.6+0.4)
21. SunsetGlow (red-clamp multi-phase)

**Extended Modules (32)**: battery (charge% green-red pulse), biorhythms (24hr chrono palette), matrix (neon rain drops), pulse_static (sine breath), glitch (static/scan/jitter), pomodoro (green-red zone timer), system_monitor (CPU/RAM green-yellow-red bars), collider, fluid_flow (Navier lite), focus_lamp, frequency_audio, ghost_keys, gravity_well, health_trace, mental_flow, morse, network, sonar_ping, zones (keymap), ambient, apm_pulse, custom, default_ui.

*Source: app/src/manager/effects/*.rs. Select in GUI tray menu. Speed decoupled.*

## ⚙️ ARCH

```
60FPS @16ms ──> HID Static Bypass
         │
├── Physics (RNG/sin/exp/interp)
└── React (78key→4zone, audio RMS, GDI screen, WinAPI fg)
```

## 🌐 EVOLUTION

4JX base → 60FPS singularity. CRT/Perlin/SignalRGB insp.

## 📡 COMPAT

Legion5P/LOQ/IdeapadG3 [FULL]

## 🔗 CLONE/BUILD

```
git clone ...
cargo build --release
```

⭐ Fork.
