export default function EffectPanel({ effects, activeEffect, onEffectChange, accentColor }) {
  const hardware = effects.filter((e) => e.category === "hardware");
  const software = effects.filter((e) => e.category === "software");

  const EffectGroup = ({ title, items }) => (
    <div className="effect-group">
      <div className="effect-group-title">
        <span className="eg-bar" style={{ background: accentColor }} />
        {title}
      </div>
      <div className="effects-grid">
        {items.map((effect) => {
          const isActive = activeEffect === effect.id;
          return (
            <button
              key={effect.id}
              className={`effect-card ${isActive ? "effect-card-active" : ""}`}
              style={{
                "--ac": accentColor,
                borderColor: isActive ? accentColor : "rgba(255,255,255,0.06)",
                boxShadow: isActive ? `0 0 20px ${accentColor}44` : "none",
              }}
              onClick={() => onEffectChange(effect.id)}
            >
              <span
                className="effect-icon"
                style={{ color: isActive ? accentColor : "rgba(255,255,255,0.4)" }}
              >
                {effect.icon}
              </span>
              <span className="effect-label">{effect.label}</span>
              {isActive && (
                <span
                  className="effect-active-pip"
                  style={{ background: accentColor, boxShadow: `0 0 6px ${accentColor}` }}
                />
              )}
            </button>
          );
        })}
      </div>
    </div>
  );

  return (
    <div className="effect-panel">
      <EffectGroup title="HARDWARE EFFECTS" items={hardware} />
      <EffectGroup title="SOFTWARE EFFECTS" items={software} />
      <p className="effect-hint">
        Hardware effects run on-device. Software effects are rendered by the Rust engine at 30 FPS.
      </p>
    </div>
  );
}
