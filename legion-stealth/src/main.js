import './style.css';
const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

let currentPreset = null;
let currentBrightness = 1.0;

async function init() {
  document.querySelector('#app').innerHTML = `
    <div class="container">
      <header>
        <h1>Legion GUI</h1>
        <div class="status-bar">
          <span id="conn-status" class="badge">Connecting...</span>
        </div>
      </header>

      <main>
        <section class="panel">
          <h2>Lighting Effects</h2>
          <select id="preset-selector"></select>
          
          <div class="controls">
             <label>Brightness: <span id="bright-val">100%</span></label>
             <input type="range" id="brightness-slider" min="0" max="100" value="100" />
          </div>
          
          <div id="params-container"></div>
          
          <button id="apply-btn" class="primary-btn">Apply Lighting</button>
        </section>

        <section class="panel">
           <h2>Visualizer</h2>
           <div class="keyboard" id="keyboard-zones">
              <div class="zone" id="z1"></div>
              <div class="zone" id="z2"></div>
              <div class="zone" id="z3"></div>
              <div class="zone" id="z4"></div>
           </div>
        </section>
      </main>
    </div>
  `;

  // Bind Events
  document.getElementById('brightness-slider').addEventListener('input', handleBrightness);
  document.getElementById('preset-selector').addEventListener('change', handlePresetChange);
  document.getElementById('apply-btn').addEventListener('click', applySettings);

  // Status Check Timer
  setInterval(checkStatus, 2000);
  checkStatus();

  // Load Presets
  try {
    const presets = await invoke('get_preset_metadata');
    const select = document.getElementById('preset-selector');
    presets.forEach(p => {
      const opt = document.createElement('option');
      opt.value = opt.textContent = p.name;
      opt.dataset.meta = JSON.stringify(p);
      select.appendChild(opt);
    });
    if (presets.length > 0) {
      currentPreset = presets[0];
      renderParams(currentPreset);
    }
  } catch(e) { console.error("Failed to load presets", e); }

  // Listen to frame events
  listen('new-colors', (event) => {
    const frame = event.payload;
    if (frame && frame.length >= 24) {
      // average back to 4 physical zones for simple display
      for (let i = 0; i < 4; i++) {
        let r=0,g=0,b=0;
        for (let j=0; j<6; j++){
           let col = frame[i*6 + j];
           r += col.r; g += col.g; b += col.b;
        }
        document.getElementById(`z${i+1}`).style.backgroundColor = `rgb(${r/6},${g/6},${b/6})`;
      }
    }
  });
}

function renderParams(preset) {
  const container = document.getElementById('params-container');
  container.innerHTML = '';
  preset.parameters.forEach(p => {
     const isColor = p.param_type && p.param_type.type === 'Color'; // Tauri serializes enum
     const div = document.createElement('div');
     div.className = 'param-row';
     const label = document.createElement('label');
     label.textContent = p.label;
     
     if (isColor || typeof p.param_type === 'object' && 'Color' in p.param_type) {
        const inp = document.createElement('input');
        inp.type = 'color';
        inp.id = `param-${p.name}`;
        // default value extraction depending on rust format could vary.
        inp.value = "#00e5ff"; 
        div.appendChild(label);
        div.appendChild(inp);
     } else {
        const inp = document.createElement('input');
        inp.type = 'range';
        inp.id = `param-${p.name}`;
        inp.min = Math.floor(p.min * 100);
        inp.max = Math.floor(p.max * 100);
        inp.value = Math.floor(p.default * 100);
        div.appendChild(label);
        div.appendChild(inp);
     }
     container.appendChild(div);
  });
}

async function handlePresetChange(e) {
  const metaStr = e.target.selectedOptions[0].dataset.meta;
  if (metaStr) {
    currentPreset = JSON.parse(metaStr);
    renderParams(currentPreset);
  }
}

async function handleBrightness(e) {
  const val = e.target.value;
  document.getElementById('bright-val').textContent = val + '%';
  try {
     await invoke('set_brightness', { brightness: val / 100.0 });
  } catch(err) { console.error(err); }
}

async function applySettings() {
  if(!currentPreset) return;
  let parameters = {};
  currentPreset.parameters.forEach(p => {
     let el = document.getElementById(`param-${p.name}`);
     const isColor = p.param_type && p.param_type.type === 'Color' || typeof p.param_type === 'object' && 'Color' in p.param_type;
     if (isColor) {
         let hex = el.value.replace('#', '');
         parameters[p.name] = { 
            Color: { 
              r: parseInt(hex.substring(0,2), 16),
              g: parseInt(hex.substring(2,4), 16),
              b: parseInt(hex.substring(4,6), 16)
            }
         };
     } else {
         parameters[p.name] = { Float: parseFloat(el.value) / 100.0 };
     }
  });

  try {
    const btn = document.getElementById('apply-btn');
    btn.textContent = "Applying...";
    await invoke('set_preset', { presetName: currentPreset.name, parameters });
    setTimeout(() => { btn.textContent = "Apply Lighting"; }, 500);
  } catch(e) {
    console.error(e);
  }
}

async function checkStatus() {
  try {
     let connected = await invoke('get_connection_status');
     let badge = document.getElementById('conn-status');
     badge.textContent = connected ? "Connected" : "Disconnected";
     badge.className = connected ? "badge badge-green" : "badge badge-red";
  } catch(e) {}
}

window.addEventListener('DOMContentLoaded', init);
