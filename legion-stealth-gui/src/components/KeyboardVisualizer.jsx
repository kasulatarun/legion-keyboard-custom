import { useMemo } from "react";

// LOQ 15 keyboard layout split into 4 zones
// Zone 0: Left   (Esc, Tab, Caps, Shift, Ctrl — columns 0-3)
// Zone 1: Center-Left  (columns 4-7)
// Zone 2: Center-Right (columns 8-11)
// Zone 3: Right  (columns 12+, numpad area, arrows)

const KEY_ROWS = [
  // row 0 - function row
  [
    { label: "Esc",  w: 1,   zone: 0 },
    { label: "F1",   w: 1,   zone: 0 },
    { label: "F2",   w: 1,   zone: 0 },
    { label: "F3",   w: 1,   zone: 0 },
    { label: "F4",   w: 1,   zone: 1 },
    { label: "F5",   w: 1,   zone: 1 },
    { label: "F6",   w: 1,   zone: 1 },
    { label: "F7",   w: 1,   zone: 2 },
    { label: "F8",   w: 1,   zone: 2 },
    { label: "F9",   w: 1,   zone: 2 },
    { label: "F10",  w: 1,   zone: 3 },
    { label: "F11",  w: 1,   zone: 3 },
    { label: "F12",  w: 1,   zone: 3 },
    { label: "Del",  w: 1,   zone: 3 },
  ],
  // row 1 - number row
  [
    { label: "`",    w: 1,   zone: 0 },
    { label: "1",    w: 1,   zone: 0 },
    { label: "2",    w: 1,   zone: 0 },
    { label: "3",    w: 1,   zone: 0 },
    { label: "4",    w: 1,   zone: 1 },
    { label: "5",    w: 1,   zone: 1 },
    { label: "6",    w: 1,   zone: 1 },
    { label: "7",    w: 1,   zone: 2 },
    { label: "8",    w: 1,   zone: 2 },
    { label: "9",    w: 1,   zone: 2 },
    { label: "0",    w: 1,   zone: 3 },
    { label: "-",    w: 1,   zone: 3 },
    { label: "=",    w: 1,   zone: 3 },
    { label: "⌫",   w: 1.5, zone: 3 },
  ],
  // row 2 - QWERTY
  [
    { label: "Tab",  w: 1.5, zone: 0 },
    { label: "Q",    w: 1,   zone: 0 },
    { label: "W",    w: 1,   zone: 0 },
    { label: "E",    w: 1,   zone: 1 },
    { label: "R",    w: 1,   zone: 1 },
    { label: "T",    w: 1,   zone: 1 },
    { label: "Y",    w: 1,   zone: 1 },
    { label: "U",    w: 1,   zone: 2 },
    { label: "I",    w: 1,   zone: 2 },
    { label: "O",    w: 1,   zone: 2 },
    { label: "P",    w: 1,   zone: 3 },
    { label: "[",    w: 1,   zone: 3 },
    { label: "]",    w: 1,   zone: 3 },
    { label: "\\",   w: 1,   zone: 3 },
  ],
  // row 3 - home row
  [
    { label: "Caps", w: 1.75, zone: 0 },
    { label: "A",    w: 1,    zone: 0 },
    { label: "S",    w: 1,    zone: 0 },
    { label: "D",    w: 1,    zone: 1 },
    { label: "F",    w: 1,    zone: 1 },
    { label: "G",    w: 1,    zone: 1 },
    { label: "H",    w: 1,    zone: 2 },
    { label: "J",    w: 1,    zone: 2 },
    { label: "K",    w: 1,    zone: 2 },
    { label: "L",    w: 1,    zone: 3 },
    { label: ";",    w: 1,    zone: 3 },
    { label: "'",    w: 1,    zone: 3 },
    { label: "Enter",w: 2.25, zone: 3 },
  ],
  // row 4 - shift row
  [
    { label: "Shift", w: 2.25, zone: 0 },
    { label: "Z",     w: 1,    zone: 0 },
    { label: "X",     w: 1,    zone: 1 },
    { label: "C",     w: 1,    zone: 1 },
    { label: "V",     w: 1,    zone: 1 },
    { label: "B",     w: 1,    zone: 2 },
    { label: "N",     w: 1,    zone: 2 },
    { label: "M",     w: 1,    zone: 2 },
    { label: ",",     w: 1,    zone: 3 },
    { label: ".",     w: 1,    zone: 3 },
    { label: "/",     w: 1,    zone: 3 },
    { label: "Shift", w: 2.75, zone: 3 },
  ],
  // row 5 - bottom row
  [
    { label: "Ctrl", w: 1.25, zone: 0 },
    { label: "Win",  w: 1,    zone: 0 },
    { label: "Alt",  w: 1,    zone: 0 },
    { label: "",     w: 6.25, zone: 1, isSpace: true },
    { label: "Alt",  w: 1,    zone: 2 },
    { label: "Fn",   w: 1,    zone: 2 },
    { label: "◄",   w: 1,    zone: 3 },
    { label: "▼",   w: 1,    zone: 3 },
    { label: "►",   w: 1,    zone: 3 },
  ],
];

function hexToRgba(hex, alpha = 1) {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}

export default function KeyboardVisualizer({ zones, activeZone, setActiveZone }) {
  const zoneColors = useMemo(
    () => zones.reduce((acc, z) => ({ ...acc, [z.id]: z.color }), {}),
    [zones]
  );

  return (
    <div className="kb-wrapper">
      {/* Zone labels */}
      <div className="zone-labels">
        {zones.map((z) => (
          <button
            key={z.id}
            className={`zone-label-btn ${activeZone === z.id ? "active" : ""}`}
            style={{
              "--zc": z.color,
              borderColor: activeZone === z.id ? z.color : "transparent",
              color: activeZone === z.id ? z.color : undefined,
            }}
            onClick={() => setActiveZone(z.id)}
          >
            <span
              className="zone-dot"
              style={{ background: z.color, boxShadow: `0 0 6px ${z.color}` }}
            />
            {z.name}
          </button>
        ))}
      </div>

      {/* Keyboard body */}
      <div className="keyboard">
        {KEY_ROWS.map((row, ri) => (
          <div key={ri} className="key-row">
            {row.map((key, ki) => {
              const color = zoneColors[key.zone] ?? "#333";
              const isActive = key.zone === activeZone;
              return (
                <button
                  key={ki}
                  className={`key ${isActive ? "key-active" : ""} ${key.isSpace ? "key-space" : ""}`}
                  style={{
                    width: `${key.w * 38}px`,
                    "--kc": color,
                    background: isActive
                      ? `linear-gradient(135deg, ${hexToRgba(color, 0.35)}, ${hexToRgba(color, 0.15)})`
                      : `linear-gradient(135deg, rgba(255,255,255,0.04), rgba(255,255,255,0.01))`,
                    borderColor: isActive ? hexToRgba(color, 0.8) : "rgba(255,255,255,0.07)",
                    boxShadow: isActive
                      ? `0 0 12px ${hexToRgba(color, 0.4)}, inset 0 1px 0 ${hexToRgba(color, 0.3)}`
                      : "none",
                    color: isActive ? color : "rgba(255,255,255,0.5)",
                    textShadow: isActive ? `0 0 8px ${hexToRgba(color, 0.8)}` : "none",
                  }}
                  onClick={() => setActiveZone(key.zone)}
                >
                  <span className="key-label">{key.label}</span>
                  {isActive && <span className="key-glow" style={{ background: hexToRgba(color, 0.15) }} />}
                </button>
              );
            })}
          </div>
        ))}

        {/* Zone divider lines */}
        <div className="zone-dividers">
          {zones.map((z) => (
            <div
              key={z.id}
              className="zone-indicator-bar"
              style={{ background: z.color, opacity: activeZone === z.id ? 0.9 : 0.3 }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
