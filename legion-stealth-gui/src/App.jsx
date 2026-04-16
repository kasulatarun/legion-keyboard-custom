import { useState, useEffect, useCallback } from "react";
import KeyboardVisualizer from "./components/KeyboardVisualizer";
import ZoneColorPicker from "./components/ZoneColorPicker";
import EffectPanel from "./components/EffectPanel";
import Sidebar from "./components/Sidebar";
import StatusBar from "./components/StatusBar";
import { useTauri } from "./hooks/useTauri";

// 4 zones for LOQ keyboard
export const DEFAULT_ZONES = [
  { id: 0, name: "Zone 1", label: "Left",         color: "#ff3b3b" },
  { id: 1, name: "Zone 2", label: "Center-Left",  color: "#ff8c00" },
  { id: 2, name: "Zone 3", label: "Center-Right", color: "#00d4ff" },
  { id: 3, name: "Zone 4", label: "Right",        color: "#7c3aed" },
];

export const EFFECTS = [
  { id: "Static",        label: "Static",         icon: "■", category: "hardware" },
  { id: "Breath",        label: "Breathing",       icon: "◉", category: "hardware" },
  { id: "Wave",          label: "Wave",            icon: "〜", category: "hardware" },
  { id: "Lightning",     label: "Lightning",       icon: "⚡", category: "software" },
  { id: "Ripple",        label: "Ripple",          icon: "◎", category: "software" },
  { id: "SystemMonitor", label: "System Monitor",  icon: "◈", category: "software" },
  { id: "Starfall",      label: "Starfall",        icon: "✦", category: "software" },
  { id: "Neon",          label: "Neon Pulse",      icon: "◆", category: "software" },
];

export default function App() {
  const [zones, setZones] = useState(DEFAULT_ZONES);
  const [activeZone, setActiveZone] = useState(0);
  const [activeEffect, setActiveEffect] = useState("Static");
  const [activeTab, setActiveTab] = useState("zones"); // "zones" | "effects" | "presets"
  const [brightness, setBrightness] = useState(100);
  const [speed, setSpeed] = useState(50);
  const [deviceConnected, setDeviceConnected] = useState(false);
  const [applying, setApplying] = useState(false);

  const { invoke, isAvailable: tauriAvailable } = useTauri();

  // Build 12-byte buffer: 3 bytes (R,G,B) × 4 zones
  const buildColorBuffer = useCallback((zoneList) => {
    const buf = [];
    for (const z of zoneList) {
      const hex = z.color.replace("#", "");
      buf.push(parseInt(hex.slice(0, 2), 16)); // R
      buf.push(parseInt(hex.slice(2, 4), 16)); // G
      buf.push(parseInt(hex.slice(4, 6), 16)); // B
    }
    return buf; // 12 bytes total
  }, []);

  const applyProfile = useCallback(async () => {
    if (!tauriAvailable) return;
    setApplying(true);
    try {
      const colorBuffer = buildColorBuffer(zones);
      await invoke("set_profile", {
        colorBuffer,
        effect: activeEffect,
        brightness,
        speed,
      });
    } catch (e) {
      console.error("set_profile failed:", e);
    } finally {
      setApplying(false);
    }
  }, [tauriAvailable, zones, activeEffect, brightness, speed, buildColorBuffer, invoke]);

  const applyEffect = useCallback(async (effectId) => {
    if (!tauriAvailable) return;
    try {
      const colorBuffer = buildColorBuffer(zones);
      await invoke("apply_custom_effect", {
        effect: effectId,
        colorBuffer,
        speed,
        brightness,
      });
    } catch (e) {
      console.error("apply_custom_effect failed:", e);
    }
  }, [tauriAvailable, zones, speed, brightness, buildColorBuffer, invoke]);

  useEffect(() => {
    const checkDevice = async () => {
      if (!tauriAvailable) return;
      try {
        const state = await invoke("get_state");
        setDeviceConnected(state?.connected ?? false);
      } catch {
        setDeviceConnected(false);
      }
    };
    checkDevice();
    const interval = setInterval(checkDevice, 3000);
    return () => clearInterval(interval);
  }, [tauriAvailable, invoke]);

  const updateZoneColor = (zoneId, color) => {
    setZones((prev) =>
      prev.map((z) => (z.id === zoneId ? { ...z, color } : z))
    );
  };

  const handleEffectChange = (effectId) => {
    setActiveEffect(effectId);
    applyEffect(effectId);
  };

  return (
    <div className="app-root">
      {/* Ambient glow based on active zone color */}
      <div
        className="ambient-glow"
        style={{ "--zone-color": zones[activeZone]?.color ?? "#7c3aed" }}
      />

      <div className="app-layout">
        <Sidebar
          activeTab={activeTab}
          setActiveTab={setActiveTab}
          deviceConnected={deviceConnected}
          tauriAvailable={tauriAvailable}
        />

        <main className="main-content">
          {/* Header */}
          <header className="app-header">
            <div className="header-title">
              <span className="header-brand">LEGION</span>
              <span className="header-sub">4-Zone RGB Controller</span>
            </div>
            <button
              className={`apply-btn ${applying ? "applying" : ""}`}
              onClick={applyProfile}
              disabled={applying || !deviceConnected}
            >
              {applying ? (
                <><span className="spinner" /> Applying…</>
              ) : (
                <><span className="apply-icon">▶</span> Apply</>
              )}
            </button>
          </header>

          {/* Keyboard Visualizer */}
          <KeyboardVisualizer
            zones={zones}
            activeZone={activeZone}
            setActiveZone={setActiveZone}
          />

          {/* Tab Content */}
          <div className="tab-content">
            {activeTab === "zones" && (
              <div className="zones-panel">
                <div className="zones-grid">
                  {zones.map((zone) => (
                    <ZoneColorPicker
                      key={zone.id}
                      zone={zone}
                      isActive={activeZone === zone.id}
                      onClick={() => setActiveZone(zone.id)}
                      onChange={(color) => updateZoneColor(zone.id, color)}
                    />
                  ))}
                </div>

                {/* Global controls */}
                <div className="global-controls">
                  <div className="control-row">
                    <label className="ctrl-label">
                      <span>BRIGHTNESS</span>
                      <span className="ctrl-value">{brightness}%</span>
                    </label>
                    <input
                      type="range" min="0" max="100"
                      value={brightness}
                      className="ctrl-slider"
                      style={{ "--thumb-color": zones[activeZone]?.color }}
                      onChange={(e) => setBrightness(+e.target.value)}
                    />
                  </div>
                  <div className="control-row">
                    <label className="ctrl-label">
                      <span>SPEED</span>
                      <span className="ctrl-value">{speed}%</span>
                    </label>
                    <input
                      type="range" min="1" max="100"
                      value={speed}
                      className="ctrl-slider"
                      style={{ "--thumb-color": zones[activeZone]?.color }}
                      onChange={(e) => setSpeed(+e.target.value)}
                    />
                  </div>
                </div>
              </div>
            )}

            {activeTab === "effects" && (
              <EffectPanel
                effects={EFFECTS}
                activeEffect={activeEffect}
                onEffectChange={handleEffectChange}
                accentColor={zones[activeZone]?.color}
              />
            )}

            {activeTab === "presets" && (
              <PresetsPanel zones={zones} setZones={setZones} />
            )}
          </div>
        </main>
      </div>

      <StatusBar
        deviceConnected={deviceConnected}
        tauriAvailable={tauriAvailable}
        activeEffect={activeEffect}
        zones={zones}
      />
    </div>
  );
}

// Quick presets panel
function PresetsPanel({ zones, setZones }) {
  const PRESETS = [
    { name: "Inferno",   colors: ["#ff1a00", "#ff4400", "#ff7700", "#ffaa00"] },
    { name: "Glacier",   colors: ["#00bfff", "#0080ff", "#0040ff", "#4000ff"] },
    { name: "Phantom",   colors: ["#7c3aed", "#9d174d", "#be185d", "#e11d48"] },
    { name: "Matrix",    colors: ["#00ff41", "#00cc33", "#009926", "#006619"] },
    { name: "Spectral",  colors: ["#ff0080", "#ff8000", "#00ff80", "#0080ff"] },
    { name: "Mono Red",  colors: ["#ff3b3b", "#ff3b3b", "#ff3b3b", "#ff3b3b"] },
    { name: "Mono Blue", colors: ["#00d4ff", "#00d4ff", "#00d4ff", "#00d4ff"] },
    { name: "Mono White",colors: ["#ffffff", "#ffffff", "#ffffff", "#ffffff"] },
    { name: "Off",       colors: ["#000000", "#000000", "#000000", "#000000"] },
  ];

  return (
    <div className="presets-panel">
      <p className="presets-hint">Click a preset to load its 4-zone color scheme.</p>
      <div className="presets-grid">
        {PRESETS.map((p) => (
          <button
            key={p.name}
            className="preset-card"
            onClick={() =>
              setZones((prev) =>
                prev.map((z, i) => ({ ...z, color: p.colors[i] }))
              )
            }
          >
            <div className="preset-swatches">
              {p.colors.map((c, i) => (
                <div key={i} className="preset-swatch" style={{ background: c }} />
              ))}
            </div>
            <span className="preset-name">{p.name}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
