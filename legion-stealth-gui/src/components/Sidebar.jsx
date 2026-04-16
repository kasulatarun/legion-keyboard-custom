const NAV = [
  { id: "zones",   icon: "⬛", label: "Zones"   },
  { id: "effects", icon: "✦",  label: "Effects" },
  { id: "presets", icon: "◈",  label: "Presets" },
];

export default function Sidebar({ activeTab, setActiveTab, deviceConnected, tauriAvailable }) {
  return (
    <aside className="sidebar">
      {/* Logo */}
      <div className="sidebar-logo">
        <span className="logo-glyph">⬡</span>
      </div>

      {/* Nav */}
      <nav className="sidebar-nav">
        {NAV.map((item) => (
          <button
            key={item.id}
            className={`nav-item ${activeTab === item.id ? "nav-item-active" : ""}`}
            onClick={() => setActiveTab(item.id)}
            title={item.label}
          >
            <span className="nav-icon">{item.icon}</span>
            <span className="nav-label">{item.label}</span>
          </button>
        ))}
      </nav>

      {/* Device status */}
      <div className="sidebar-footer">
        <div className={`device-dot ${deviceConnected ? "connected" : "disconnected"}`} />
        <span className="device-text">
          {!tauriAvailable ? "Preview" : deviceConnected ? "Connected" : "No Device"}
        </span>
      </div>
    </aside>
  );
}
