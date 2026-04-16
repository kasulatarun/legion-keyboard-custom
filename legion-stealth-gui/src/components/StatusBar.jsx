export default function StatusBar({ deviceConnected, tauriAvailable, activeEffect, zones }) {
  // Build compact 12-byte buffer display
  const bufPreview = zones
    .map((z) => {
      const h = z.color.replace("#", "");
      return `${h.slice(0, 2)} ${h.slice(2, 4)} ${h.slice(4, 6)}`;
    })
    .join("  |  ");

  return (
    <footer className="status-bar">
      <div className="status-left">
        <span className={`status-dot ${deviceConnected ? "s-green" : "s-red"}`} />
        <span className="status-text">
          {!tauriAvailable
            ? "Running in browser preview mode"
            : deviceConnected
            ? "LOQ device connected · HID ready"
            : "Waiting for LOQ device…"}
        </span>
      </div>
      <div className="status-mid">
        <span className="s-label">EFFECT</span>
        <span className="s-value">{activeEffect}</span>
      </div>
      <div className="status-right">
        <span className="s-label">12-BYTE BUFFER</span>
        <code className="s-buf">{bufPreview}</code>
      </div>
    </footer>
  );
}
