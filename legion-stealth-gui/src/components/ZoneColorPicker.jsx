import { useState, useRef } from "react";

const SWATCHES = [
  "#ff3b3b", "#ff6b35", "#ff8c00", "#ffcc00",
  "#00ff88", "#00d4ff", "#0080ff", "#7c3aed",
  "#e11d48", "#be185d", "#ffffff", "#000000",
];

export default function ZoneColorPicker({ zone, isActive, onClick, onChange }) {
  const [showPicker, setShowPicker] = useState(false);
  const pickerRef = useRef(null);

  const handleNativeChange = (e) => {
    onChange(e.target.value);
  };

  const handleSwatchClick = (color) => {
    onChange(color);
  };

  // Parse hex to hsl for display
  const hex = zone.color;
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const l = (max + min) / 2;

  return (
    <div
      className={`zone-card ${isActive ? "zone-card-active" : ""}`}
      style={{
        "--zc": zone.color,
        borderColor: isActive ? zone.color : "rgba(255,255,255,0.06)",
        boxShadow: isActive ? `0 0 24px ${zone.color}33, 0 0 1px ${zone.color}66` : "none",
      }}
      onClick={onClick}
    >
      {/* Active indicator */}
      {isActive && (
        <div className="zone-card-active-bar" style={{ background: zone.color }} />
      )}

      {/* Zone header */}
      <div className="zone-card-header">
        <div className="zone-name-row">
          <span
            className="zone-color-dot"
            style={{ background: zone.color, boxShadow: `0 0 8px ${zone.color}` }}
          />
          <div>
            <div className="zone-name">{zone.name}</div>
            <div className="zone-label">{zone.label}</div>
          </div>
        </div>
        <span className="zone-hex">{zone.color.toUpperCase()}</span>
      </div>

      {/* Color preview bar */}
      <div
        className="color-preview-bar"
        style={{
          background: `linear-gradient(90deg, ${zone.color}dd, ${zone.color}44)`,
          boxShadow: `0 0 12px ${zone.color}55`,
        }}
      />

      {/* Swatches */}
      <div className="swatch-grid">
        {SWATCHES.map((sw) => (
          <button
            key={sw}
            className={`swatch ${zone.color === sw ? "swatch-selected" : ""}`}
            style={{
              background: sw,
              boxShadow: zone.color === sw ? `0 0 8px ${sw}` : "none",
              outline: zone.color === sw ? `2px solid ${sw}` : "none",
            }}
            onClick={(e) => { e.stopPropagation(); handleSwatchClick(sw); }}
            title={sw}
          />
        ))}
      </div>

      {/* Custom color input */}
      <div className="custom-color-row" onClick={(e) => e.stopPropagation()}>
        <label className="custom-color-label">
          <span>CUSTOM</span>
          <div className="native-picker-wrapper" style={{ background: zone.color }}>
            <input
              ref={pickerRef}
              type="color"
              value={zone.color}
              className="native-color-input"
              onChange={handleNativeChange}
            />
            <span>⊕</span>
          </div>
        </label>
      </div>
    </div>
  );
}
