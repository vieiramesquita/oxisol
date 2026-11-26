// Oxisol Studio — Zero-dependency Interactive Scientific Frontend (EN / PT-BR)

const I18N = {
  en: {
    brand_sub: "Open-Source CENTURY-like Biogeochemical Engine in Rust (C · N · P · S)",
    tab_lab: "Scenario Lab",
    tab_grid: "Spatial Grid & HPC",
    tab_api: "REST API & Docs",
    sec_preset: "Ecosystem Preset",
    sec_climate: "Climate & Atmosphere",
    sec_soil: "Soil Properties",
    sec_mgmt: "Land Management",
    sec_solver: "Solver & Horizon",
    lbl_temp: "Mean Annual Temp",
    lbl_amp: "Seasonal Amplitude",
    lbl_precip: "Annual Precip",
    lbl_lat: "Latitude (+N / −S)",
    lbl_co2_start: "CO₂ Start (ppm)",
    lbl_co2_end: "CO₂ End (ppm)",
    lbl_sand: "Sand",
    lbl_clay: "Clay",
    lbl_ph: "Soil pH",
    lbl_bulkd: "Bulk Density",
    lbl_rock: "Rock Fraction",
    lbl_drain: "Drainage (Anaerobic → Well)",
    lbl_tillage: "Tillage & Harvest",
    opt_till_none: "Natural (No Harvest)",
    opt_till_notill: "No-Till + Harvest",
    opt_till_plow: "Moldboard Plow + Harvest",
    lbl_fert_n: "Fert N (g/m²/yr)",
    lbl_fert_p: "Fert P (g/m²/yr)",
    lbl_irri: "Irrigation",
    opt_irri_none: "Rainfed",
    opt_irri_auto: "Auto 75% AWHC",
    lbl_fire: "Fire Return (yr)",
    lbl_graz: "Grazing (0–0.6)",
    lbl_omad: "Compost C (g/m²)",
    lbl_years: "Simulation Years",
    lbl_elem: "Cycles",
    lbl_prec: "Float",
    lbl_prof: "Profile",
    btn_run: "Run Simulation",
    btn_running: "Simulating...",
    btn_pin: "Pin as Baseline (A)",
    btn_pinned: "Baseline A Pinned",
    btn_clear_pin: "Clear Baseline",
    btn_csv: "Export CSV",
    btn_json: "Export JSON",
    kpi_somsc: "Total Soil Carbon (SOMSC)",
    kpi_npp: "Annual NPP",
    kpi_aet: "Actual ET (AET)",
    kpi_minn: "Profile Mineral N",
    kpi_mb: "Mass Balance Residual",
    kpi_mb_sub: "Exact C·N·P·S Audit",
    kpi_speed: "Engine Speed",
    chart_som_title: "Soil Organic Carbon Pools (g C/m²)",
    chart_flux_title: "Ecosystem Fluxes: NPP, Respiration & Water",
    chart_bio_title: "Vegetation Carbon & Surface Mineral Nutrients",
    chart_diag_title: "Live Biogeochemical Pool Architecture",
    grid_cfg_title: "Spatial Grid Configuration",
    grid_source: "Grid Source",
    grid_opt_synth: "Built-in Continental Gradient Grid",
    grid_opt_attached: "Attached VEMAP / Spatial Dataset (--data-dir)",
    grid_rows: "Rows",
    grid_cols: "Cols",
    grid_years: "Years / Cell",
    grid_threads: "Worker Threads",
    grid_chunk: "Chunk Size",
    btn_run_grid: "Run Parallel Grid",
    grid_inspector: "Cell Inspector",
    grid_click_hint: "Click any active cell on the map to inspect its biogeochemical state.",
    gkpi_cells: "Active Cells × Years",
    gkpi_time: "Wall Time",
    gkpi_rate: "Measured Throughput",
    gkpi_hpc: "HPC 256T Server Est.",
    gkpi_mean_som: "Mean Continental SOMSC",
    map_title: "Interactive 2D Spatial Heatmap",
    api_overview_title: "Oxisol HTTP/JSON API",
    api_overview_desc: "Oxisol Studio embeds an Axum REST API inside the single compiled Rust binary. Automate scenarios from Python, R, Julia, or GIS pipelines on local machines or remote high-performance servers over SSH tunnels.",
    ep_health: "Liveness & version check",
    ep_sys: "Host CPU count, CellState L1d bytes & attached grid",
    ep_presets: "7 calibrated ecosystem presets (EN/PT-BR)",
    ep_sim: "Single-cell scenario trajectory + mass balance audit",
    ep_grid: "Lock-free multi-core spatial grid execution",
    api_example_title: "Quick Python Example",
    sys_card_title: "Architecture & Documentation",
    hpc_title: "Running Remotely on Modern High-Performance Servers",
    docs_pointer: "Full scientific documentation (mdBook) lives in docs/src/ and API reference builds via cargo doc --workspace --no-deps. Released under the MIT License."
  },
  pt: {
    brand_sub: "Engine Biogeoquímica CENTURY-like Open-Source em Rust (C · N · P · S)",
    tab_lab: "Laboratório de Cenários",
    tab_grid: "Grade Espacial & HPC",
    tab_api: "API REST & Docs",
    sec_preset: "Preset de Ecossistema",
    sec_climate: "Clima & Atmosfera",
    sec_soil: "Propriedades do Solo",
    sec_mgmt: "Manejo da Terra",
    sec_solver: "Solver & Horizonte",
    lbl_temp: "Temp. Média Anual",
    lbl_amp: "Amplitude Sazonal",
    lbl_precip: "Precipitação Anual",
    lbl_lat: "Latitude (+N / −S)",
    lbl_co2_start: "CO₂ Inicial (ppm)",
    lbl_co2_end: "CO₂ Final (ppm)",
    lbl_sand: "Areia",
    lbl_clay: "Argila",
    lbl_ph: "pH do Solo",
    lbl_bulkd: "Densidade (g/cm³)",
    lbl_rock: "Fração de Rocha",
    lbl_drain: "Drenagem (Anaeróbica → Boa)",
    lbl_tillage: "Preparo & Colheita",
    opt_till_none: "Natural (Sem Colheita)",
    opt_till_notill: "Plantio Direto + Colheita",
    opt_till_plow: "Aração Convencional + Colheita",
    lbl_fert_n: "Adubo N (g/m²/ano)",
    lbl_fert_p: "Adubo P (g/m²/ano)",
    lbl_irri: "Irrigação",
    opt_irri_none: "Sequeiro",
    opt_irri_auto: "Automática 75% CAD",
    lbl_fire: "Ciclo de Fogo (anos)",
    lbl_graz: "Pastejo (0–0,6)",
    lbl_omad: "Composto C (g/m²)",
    lbl_years: "Anos de Simulação",
    lbl_elem: "Ciclos",
    lbl_prec: "Precisão",
    lbl_prof: "Perfil",
    btn_run: "Rodar Simulação",
    btn_running: "Simulando...",
    btn_pin: "Fixar como Baseline (A)",
    btn_pinned: "Baseline A Fixado",
    btn_clear_pin: "Limpar Baseline",
    btn_csv: "Exportar CSV",
    btn_json: "Exportar JSON",
    kpi_somsc: "Carbono Total do Solo (SOMSC)",
    kpi_npp: "NPP Anual",
    kpi_aet: "Evapotranspiração Real (AET)",
    kpi_minn: "N Mineral no Perfil",
    kpi_mb: "Resíduo de Balanço de Massa",
    kpi_mb_sub: "Auditoria Exata C·N·P·S",
    kpi_speed: "Velocidade da Engine",
    chart_som_title: "Pools de Carbono Orgânico do Solo (g C/m²)",
    chart_flux_title: "Fluxos do Ecossistema: NPP, Respiração & Água",
    chart_bio_title: "Biomassa Vegetal & Nutrientes Minerais",
    chart_diag_title: "Arquitetura Viva de Pools Biogeoquímicos",
    grid_cfg_title: "Configuração da Grade Espacial",
    grid_source: "Fonte da Grade",
    grid_opt_synth: "Grade Continental Sintética Integrada",
    grid_opt_attached: "Dataset Espacial / VEMAP Anexado (--data-dir)",
    grid_rows: "Linhas",
    grid_cols: "Colunas",
    grid_years: "Anos / Célula",
    grid_threads: "Threads",
    grid_chunk: "Tamanho do Chunk",
    btn_run_grid: "Rodar Grade em Paralelo",
    grid_inspector: "Inspetor de Célula",
    grid_click_hint: "Clique em qualquer célula ativa no mapa para inspecionar o estado biogeoquímico.",
    gkpi_cells: "Células Ativas × Anos",
    gkpi_time: "Tempo de Parede",
    gkpi_rate: "Throughput Medido",
    gkpi_hpc: "Estimativa Servidor HPC 256T",
    gkpi_mean_som: "SOMSC Médio Continental",
    map_title: "Mapa de Calor Espacial 2D Interativo",
    api_overview_title: "API HTTP/JSON do Oxisol",
    api_overview_desc: "O Oxisol Studio embute uma API REST Axum dentro do binário único em Rust. Automatize cenários via Python, R, Julia ou pipelines GIS localmente ou em servidores modernos de alto desempenho via túnel SSH.",
    ep_health: "Checagem de saúde e versão",
    ep_sys: "CPUs do host, tamanho L1d do CellState e grade anexada",
    ep_presets: "7 presets calibrados de ecossistemas (EN/PT-BR)",
    ep_sim: "Trajetória de célula única + auditoria de balanço de massa",
    ep_grid: "Execução espacial multi-core lock-free via rayon",
    api_example_title: "Exemplo Rápido em Python",
    sys_card_title: "Arquitetura & Documentação",
    hpc_title: "Execução Remota em Servidores Modernos de Alto Desempenho",
    docs_pointer: "A documentação científica completa (mdBook) está em docs/src/ e a referência de API é gerada via cargo doc --workspace --no-deps. Licença MIT."
  }
};

const state = {
  lang: "en",
  system: null,
  presets: [],
  lastSim: null,
  baselineSim: null,
  lastGrid: null,
  gridCellMap: new Map()
};

const $ = (id) => document.getElementById(id);

function t(key) {
  return (I18N[state.lang] && I18N[state.lang][key]) || I18N.en[key] || key;
}

function applyLang(lang) {
  state.lang = lang;
  $("lang-en").classList.toggle("active", lang === "en");
  $("lang-pt").classList.toggle("active", lang === "pt");
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    if (I18N[lang][key]) el.textContent = I18N[lang][key];
  });
  populatePresetSelect(true);
  if (state.baselineSim) $("btn-pin").textContent = t("btn_pinned");
}

function fmt(num, digits = 1) {
  if (!Number.isFinite(num)) return "—";
  return num.toLocaleString("en-US", {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits
  });
}

function fmtSci(num) {
  if (!Number.isFinite(num)) return "—";
  if (num === 0) return "0.00e+0";
  return num.toExponential(2);
}

function updateTextureBar() {
  const sand = Number($("in-sand").value);
  let clay = Number($("in-clay").value);
  if (sand + clay > 96) {
    clay = Math.max(4, 96 - sand);
    $("in-clay").value = String(clay);
  }
  const silt = Math.max(2, 100 - sand - clay);
  $("val-sand").textContent = `${sand}%`;
  $("val-clay").textContent = `${clay}% (Silt ${silt}%)`;
  $("texture-bar").innerHTML = `
    <div style="width:${sand}%;background:#f59e0b" title="Sand ${sand}%"></div>
    <div style="width:${silt}%;background:#06b6d4" title="Silt ${silt}%"></div>
    <div style="width:${clay}%;background:#8b5cf6" title="Clay ${clay}%"></div>
  `;
}

function syncSliderLabels() {
  $("val-temp").textContent = `${Number($("in-temp").value).toFixed(1)} °C`;
  $("val-amp").textContent = `${Number($("in-amp").value).toFixed(1)} °C`;
  $("val-precip").textContent = `${$("in-precip").value} cm`;
  const lat = Number($("in-lat").value);
  $("val-lat").textContent = `${Math.abs(lat).toFixed(1)}°${lat >= 0 ? "N" : "S"}`;
  $("val-rock").textContent = `${$("in-rock").value}%`;
  $("val-drain").textContent = Number($("in-drain").value).toFixed(2);
  $("val-years").textContent = `${$("in-years").value} yr`;
  $("val-grid-years").textContent = `${$("grid-years").value} yr`;
  updateTextureBar();
}

function populatePresetSelect(keepSelection = false) {
  const sel = $("preset-select");
  const prev = keepSelection ? sel.value : null;
  sel.innerHTML = "";
  for (const p of state.presets) {
    const opt = document.createElement("option");
    opt.value = p.id;
    opt.textContent = state.lang === "pt" ? p.name_pt : p.name_en;
    sel.appendChild(opt);
  }
  if (prev && state.presets.some((p) => p.id === prev)) {
    sel.value = prev;
  }
  updatePresetDesc();
}

function updatePresetDesc() {
  const p = state.presets.find((x) => x.id === $("preset-select").value);
  if (!p) return;
  $("preset-desc").textContent = state.lang === "pt" ? p.desc_pt : p.desc_en;
}

function loadPresetIntoControls(presetId) {
  const p = state.presets.find((x) => x.id === presetId);
  if (!p) return;
  const d = p.defaults;
  $("in-temp").value = d.mean_temp_c;
  $("in-amp").value = d.temp_amp_c;
  $("in-precip").value = d.annual_precip_cm;
  $("in-lat").value = d.lat;
  $("in-co2-start").value = d.co2_start_ppm;
  $("in-co2-end").value = d.co2_end_ppm;
  $("in-sand").value = Math.round(d.sand * 100);
  $("in-clay").value = Math.round(d.clay * 100);
  $("in-ph").value = d.ph;
  $("in-bulkd").value = d.bulkd;
  $("in-rock").value = Math.round(d.rock_frac * 100);
  $("in-drain").value = d.drain;
  $("in-tillage").value = d.tillage;
  $("in-fert-n").value = d.fertilizer_n;
  $("in-fert-p").value = d.fertilizer_p;
  $("in-irri").value = d.irrigation;
  $("in-fire").value = d.fire_interval_yr;
  $("in-graz").value = d.grazing_intensity;
  $("in-omad").value = d.omad_c;
  $("in-years").value = d.years;
  $("in-elem").value = d.elements;
  $("in-prec").value = d.precision;
  $("in-prof").value = d.profile;
  updatePresetDesc();
  syncSliderLabels();
}

function collectSimRequest() {
  const sand = Number($("in-sand").value) / 100;
  const clay = Number($("in-clay").value) / 100;
  const silt = Math.max(0.02, 1.0 - sand - clay);
  return {
    preset_id: $("preset-select").value || "temperate_grassland",
    mean_temp_c: Number($("in-temp").value),
    temp_amp_c: Number($("in-amp").value),
    annual_precip_cm: Number($("in-precip").value),
    lat: Number($("in-lat").value),
    co2_start_ppm: Number($("in-co2-start").value),
    co2_end_ppm: Number($("in-co2-end").value),
    sand,
    silt,
    clay,
    bulkd: Number($("in-bulkd").value),
    ph: Number($("in-ph").value),
    rock_frac: Number($("in-rock").value) / 100,
    drain: Number($("in-drain").value),
    years: Number($("in-years").value),
    elements: $("in-elem").value,
    profile: $("in-prof").value,
    precision: $("in-prec").value,
    audit_mass_balance: true,
    tillage: $("in-tillage").value,
    fertilizer_n: Number($("in-fert-n").value),
    fertilizer_p: Number($("in-fert-p").value),
    irrigation: $("in-irri").value,
    fire_interval_yr: Number($("in-fire").value),
    grazing_intensity: Number($("in-graz").value),
    omad_c: Number($("in-omad").value)
  };
}

function renderDelta(elId, curr, base, unit) {
  const el = $(elId);
  if (base == null) {
    el.textContent = unit;
    return;
  }
  const diff = curr - base;
  const pct = Math.abs(base) > 1e-6 ? (diff / base) * 100 : 0;
  const cls = diff >= 0 ? "delta-pos" : "delta-neg";
  const sign = diff >= 0 ? "+" : "";
  el.innerHTML = `${unit} · <span class="${cls}">${sign}${fmt(diff, 1)} (${sign}${fmt(pct, 1)}%)</span>`;
}

// Reusable high-DPI SVG multi-series line chart with hover tooltip
function renderLineChart(hostId, legendId, seriesDefs, xVals, baselineDefs = []) {
  const host = $(hostId);
  const leg = $(legendId);
  const W = 640, H = 250;
  const pad = { l: 54, r: 16, t: 14, b: 30 };
  const plotW = W - pad.l - pad.r;
  const plotH = H - pad.t - pad.b;

  const allSeries = [...seriesDefs, ...baselineDefs];
  let maxY = 1e-6;
  for (const s of allSeries) {
    for (const v of s.data) {
      if (Number.isFinite(v) && v > maxY) maxY = v;
    }
  }
  maxY *= 1.1;
  const minX = xVals[0] ?? 1;
  const maxX = Math.max(minX + 1, xVals[xVals.length - 1] ?? 10);
  const sx = (x) => pad.l + ((x - minX) / (maxX - minX)) * plotW;
  const sy = (y) => pad.t + plotH - (Math.max(0, y) / maxY) * plotH;

  let gridSvg = "";
  for (let i = 0; i <= 4; i++) {
    const yv = (maxY * i) / 4;
    const yp = sy(yv);
    gridSvg += `<line x1="${pad.l}" y1="${yp}" x2="${W - pad.r}" y2="${yp}" stroke="#233044" stroke-dasharray="3,3"/>`;
    gridSvg += `<text x="${pad.l - 7}" y="${yp + 4}" text-anchor="end" fill="#8b9bb4" font-size="10" font-family="monospace">${yv >= 1000 ? (yv / 1000).toFixed(1) + "k" : yv.toFixed(0)}</text>`;
  }
  const xTicks = 5;
  for (let i = 0; i <= xTicks; i++) {
    const xv = minX + ((maxX - minX) * i) / xTicks;
    const xp = sx(xv);
    gridSvg += `<text x="${xp}" y="${H - 8}" text-anchor="middle" fill="#8b9bb4" font-size="10" font-family="monospace">Yr ${Math.round(xv)}</text>`;
  }

  let paths = "";
  for (const s of baselineDefs) {
    const d = s.data.map((y, idx) => `${idx === 0 ? "M" : "L"}${sx(xVals[idx]).toFixed(1)},${sy(y).toFixed(1)}`).join(" ");
    paths += `<path d="${d}" fill="none" stroke="${s.color}" stroke-width="1.6" stroke-dasharray="5,4" opacity="0.65"/>`;
  }
  for (const s of seriesDefs) {
    const d = s.data.map((y, idx) => `${idx === 0 ? "M" : "L"}${sx(xVals[idx]).toFixed(1)},${sy(y).toFixed(1)}`).join(" ");
    paths += `<path d="${d}" fill="none" stroke="${s.color}" stroke-width="${s.width || 2.2}"/>`;
  }

  host.innerHTML = `<svg viewBox="0 0 ${W} ${H}" preserveAspectRatio="none">
    ${gridSvg}
    ${paths}
    <line class="hover-line" x1="0" y1="${pad.t}" x2="0" y2="${pad.t + plotH}" stroke="#e6edf3" stroke-width="1" stroke-dasharray="3,3" style="display:none"/>
    <rect class="hover-zone" x="${pad.l}" y="${pad.t}" width="${plotW}" height="${plotH}" fill="transparent"/>
  </svg>`;

  leg.innerHTML = allSeries.map((s) =>
    `<span class="leg-item"><span class="leg-dot" style="background:${s.color};${s.dashed ? "opacity:0.6;border:1px dashed #fff;" : ""}"></span>${s.label}</span>`
  ).join("");

  const svg = host.querySelector("svg");
  const line = svg.querySelector(".hover-line");
  const zone = svg.querySelector(".hover-zone");
  const tip = $("tooltip");

  zone.addEventListener("mousemove", (ev) => {
    const rect = svg.getBoundingClientRect();
    const relX = ((ev.clientX - rect.left) / rect.width) * W;
    const frac = Math.min(1, Math.max(0, (relX - pad.l) / plotW));
    const idx = Math.min(xVals.length - 1, Math.max(0, Math.round(frac * (xVals.length - 1))));
    const xp = sx(xVals[idx]);
    line.setAttribute("x1", xp);
    line.setAttribute("x2", xp);
    line.style.display = "block";
    const rows = allSeries.map((s) => `<div style="color:${s.color}">${s.label}: <strong>${fmt(s.data[idx], 1)}</strong></div>`).join("");
    tip.innerHTML = `<div style="color:#8b9bb4;margin-bottom:3px;">Year ${xVals[idx]}</div>${rows}`;
    tip.style.display = "block";
    tip.style.left = `${Math.min(window.innerWidth - 220, ev.clientX + 14)}px`;
    tip.style.top = `${Math.max(12, ev.clientY - 24)}px`;
  });
  zone.addEventListener("mouseleave", () => {
    line.style.display = "none";
    tip.style.display = "none";
  });
}

function renderPoolDiagram(pools, co2End) {
  $("diag-co2-pill").textContent = `${fmt(co2End, 0)} ppm CO₂`;
  $("pool-diagram").innerHTML = `<svg viewBox="0 0 640 250">
    <defs>
      <marker id="arr" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
        <path d="M 0 1 L 10 5 L 0 9 z" fill="#4b6280"/>
      </marker>
    </defs>
    <!-- Arrows -->
    <line x1="125" y1="62" x2="125" y2="96" stroke="#10b981" stroke-width="2" marker-end="url(#arr)"/>
    <line x1="215" y1="128" x2="258" y2="128" stroke="#4b6280" stroke-width="2" marker-end="url(#arr)"/>
    <line x1="390" y1="108" x2="442" y2="54" stroke="#4b6280" stroke-width="2" marker-end="url(#arr)"/>
    <line x1="390" y1="128" x2="442" y2="128" stroke="#4b6280" stroke-width="2" marker-end="url(#arr)"/>
    <line x1="532" y1="78" x2="532" y2="100" stroke="#f59e0b" stroke-width="1.8" marker-end="url(#arr)"/>
    <line x1="532" y1="154" x2="532" y2="176" stroke="#8b5cf6" stroke-width="1.8" marker-end="url(#arr)"/>
    <line x1="322" y1="162" x2="322" y2="188" stroke="#06b6d4" stroke-width="1.8" marker-end="url(#arr)"/>

    <!-- Atmosphere -->
    <rect x="28" y="12" width="194" height="48" rx="9" fill="#17202d" stroke="#06b6d4" stroke-width="1.5"/>
    <text x="125" y="31" text-anchor="middle" fill="#06b6d4" font-size="11" font-weight="700">ATMOSPHERE CO₂</text>
    <text x="125" y="49" text-anchor="middle" fill="#e6edf3" font-size="11" font-family="monospace">NPP ${fmt(pools.final_npp, 0)} · Resp ${fmt(pools.final_resp, 0)}</text>

    <!-- Live Vegetation -->
    <rect x="28" y="100" width="186" height="58" rx="9" fill="#112620" stroke="#10b981" stroke-width="1.6"/>
    <text x="121" y="120" text-anchor="middle" fill="#10b981" font-size="11" font-weight="700">LIVE VEGETATION</text>
    <text x="121" y="143" text-anchor="middle" fill="#e6edf3" font-size="15" font-weight="700" font-family="monospace">${fmt(pools.vegetation_c, 0)} g C/m²</text>

    <!-- Litter & Dead Wood -->
    <rect x="262" y="100" width="128" height="58" rx="9" fill="#17202d" stroke="#94a3b8" stroke-width="1.4"/>
    <text x="326" y="120" text-anchor="middle" fill="#cbd5e1" font-size="10.5" font-weight="700">LITTER &amp; WOOD</text>
    <text x="326" y="143" text-anchor="middle" fill="#e6edf3" font-size="14" font-weight="700" font-family="monospace">${fmt(pools.litter_and_deadwood_c, 0)} g C</text>

    <!-- SOM1 Active -->
    <rect x="446" y="20" width="172" height="54" rx="9" fill="#102520" stroke="#10b981" stroke-width="1.5"/>
    <text x="532" y="39" text-anchor="middle" fill="#10b981" font-size="10.5" font-weight="700">SOM1 ACTIVE</text>
    <text x="532" y="61" text-anchor="middle" fill="#e6edf3" font-size="14" font-weight="700" font-family="monospace">${fmt(pools.som1_c, 1)} g C/m²</text>

    <!-- SOM2 Slow -->
    <rect x="446" y="102" width="172" height="52" rx="9" fill="#251d10" stroke="#f59e0b" stroke-width="1.5"/>
    <text x="532" y="120" text-anchor="middle" fill="#f59e0b" font-size="10.5" font-weight="700">SOM2 SLOW</text>
    <text x="532" y="142" text-anchor="middle" fill="#e6edf3" font-size="14" font-weight="700" font-family="monospace">${fmt(pools.som2_c, 0)} g C/m²</text>

    <!-- SOM3 Passive -->
    <rect x="446" y="180" width="172" height="54" rx="9" fill="#1d152c" stroke="#8b5cf6" stroke-width="1.5"/>
    <text x="532" y="199" text-anchor="middle" fill="#a78bfa" font-size="10.5" font-weight="700">SOM3 PASSIVE</text>
    <text x="532" y="221" text-anchor="middle" fill="#e6edf3" font-size="14" font-weight="700" font-family="monospace">${fmt(pools.som3_c, 0)} g C/m²</text>

    <!-- Mineral N / P / S -->
    <rect x="28" y="192" width="362" height="44" rx="9" fill="#101c28" stroke="#06b6d4" stroke-width="1.4"/>
    <text x="209" y="210" text-anchor="middle" fill="#06b6d4" font-size="10.5" font-weight="700">PROFILE MINERAL NUTRIENTS (g/m²)</text>
    <text x="209" y="228" text-anchor="middle" fill="#e6edf3" font-size="12" font-family="monospace">N: ${fmt(pools.profile_minerl_n, 2)}  ·  P: ${fmt(pools.profile_minerl_p, 2)}  ·  S: ${fmt(pools.profile_minerl_s, 2)}</text>
  </svg>`;
}

function renderSimulation(sim) {
  const fp = sim.final_pools;
  const bp = state.baselineSim ? state.baselineSim.final_pools : null;

  $("kpi-somsc").textContent = fmt(fp.somsc, 0);
  renderDelta("kpi-somsc-delta", fp.somsc, bp ? bp.somsc : null, "g C/m²");

  $("kpi-npp").textContent = fmt(fp.final_npp, 1);
  renderDelta("kpi-npp-delta", fp.final_npp, bp ? bp.final_npp : null, "g C/m²/yr");

  $("kpi-aet").textContent = fmt(fp.final_aet, 1);
  renderDelta("kpi-aet-delta", fp.final_aet, bp ? bp.final_aet : null, "cm/yr");

  $("kpi-minn").textContent = fmt(fp.profile_minerl_n, 2);
  renderDelta("kpi-minn-delta", fp.profile_minerl_n, bp ? bp.profile_minerl_n : null, "g N/m²");

  $("kpi-mb").textContent = fmtSci(fp.max_mass_balance_err);
  $("kpi-speed").textContent = `${fmt(sim.throughput_cell_yr_s, 0)}`;
  $("kpi-speed-sub").textContent = `${fmt(sim.elapsed_ms, 1)} ms · ${sim.precision}`;

  const x = sim.series.map((p) => p.year);
  const baseMatch = state.baselineSim && state.baselineSim.series.length === sim.series.length;

  renderLineChart(
    "chart-som",
    "legend-som",
    [
      { label: "Total SOMSC", color: "#10b981", width: 2.5, data: sim.series.map((p) => p.somsc) },
      { label: "SOM2 Slow", color: "#f59e0b", width: 1.9, data: sim.series.map((p) => p.som2c) },
      { label: "SOM3 Passive", color: "#8b5cf6", width: 1.9, data: sim.series.map((p) => p.som3c) },
      { label: "SOM1 Active", color: "#06b6d4", width: 1.5, data: sim.series.map((p) => p.som1c) }
    ],
    x,
    baseMatch
      ? [{ label: "Baseline SOMSC (A)", color: "#e6edf3", dashed: true, data: state.baselineSim.series.map((p) => p.somsc) }]
      : []
  );

  renderLineChart(
    "chart-flux",
    "legend-flux",
    [
      { label: "NPP (g C/m²/yr)", color: "#10b981", width: 2.2, data: sim.series.map((p) => p.npp) },
      { label: "Resp (g C/m²/yr)", color: "#f43f5e", width: 1.8, data: sim.series.map((p) => p.resp) },
      { label: "AET (cm/yr)", color: "#06b6d4", width: 1.8, data: sim.series.map((p) => p.aet) }
    ],
    x,
    baseMatch
      ? [{ label: "Baseline NPP (A)", color: "#e6edf3", dashed: true, data: state.baselineSim.series.map((p) => p.npp) }]
      : []
  );

  renderLineChart(
    "chart-bio",
    "legend-bio",
    [
      { label: "Large Wood C", color: "#f59e0b", width: 2.0, data: sim.series.map((p) => p.rlwodc) },
      { label: "Herbaceous C", color: "#10b981", width: 2.0, data: sim.series.map((p) => p.aglivc + p.bglivc) },
      { label: "Litter C", color: "#94a3b8", width: 1.6, data: sim.series.map((p) => p.litter_c) }
    ],
    x
  );

  const lastCo2 = sim.series.length ? sim.series[sim.series.length - 1].co2_ppm : 350;
  renderPoolDiagram(fp, lastCo2);
}

async function runSimulation() {
  const btn = $("btn-run");
  btn.disabled = true;
  btn.textContent = t("btn_running");
  try {
    const req = collectSimRequest();
    const res = await fetch("/api/simulate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req)
    });
    if (!res.ok) throw new Error(await res.text());
    const data = await res.json();
    state.lastSim = data;
    renderSimulation(data);
  } catch (err) {
    alert(`Simulation error: ${err.message}`);
  } finally {
    btn.disabled = false;
    btn.textContent = t("btn_run");
  }
}

// ==================== SPATIAL GRID & HEATMAP ====================
function viridisColor(tNorm) {
  const stops = [
    [27, 16, 53],
    [59, 45, 123],
    [29, 111, 138],
    [16, 185, 129],
    [163, 230, 53],
    [253, 224, 71]
  ];
  const clamped = Math.min(1, Math.max(0, tNorm));
  const scaled = clamped * (stops.length - 1);
  const lo = Math.floor(scaled);
  const hi = Math.min(stops.length - 1, lo + 1);
  const f = scaled - lo;
  const r = Math.round(stops[lo][0] + (stops[hi][0] - stops[lo][0]) * f);
  const g = Math.round(stops[lo][1] + (stops[hi][1] - stops[lo][1]) * f);
  const b = Math.round(stops[lo][2] + (stops[hi][2] - stops[lo][2]) * f);
  return `rgb(${r},${g},${b})`;
}

function drawGridHeatmap() {
  const grid = state.lastGrid;
  if (!grid) return;
  const layer = $("map-layer").value;
  const canvas = $("grid-canvas");
  const ctx = canvas.getContext("2d");
  const cw = canvas.width;
  const ch = canvas.height;
  ctx.fillStyle = "#070b10";
  ctx.fillRect(0, 0, cw, ch);

  let minV = Infinity, maxV = -Infinity;
  for (const c of grid.cells) {
    const v = Number(c[layer]);
    if (v < minV) minV = v;
    if (v > maxV) maxV = v;
  }
  if (!Number.isFinite(minV) || maxV <= minV) { minV = 0; maxV = 1; }
  $("cb-min").textContent = fmt(minV, layer === "biome_id" ? 0 : 1);
  $("cb-max").textContent = fmt(maxV, layer === "biome_id" ? 0 : 1);

  const cellW = cw / grid.cols;
  const cellH = ch / grid.rows;
  for (const c of grid.cells) {
    const v = Number(c[layer]);
    const norm = (v - minV) / (maxV - minV);
    ctx.fillStyle = viridisColor(norm);
    ctx.fillRect(c.col * cellW, c.row * cellH, Math.ceil(cellW), Math.ceil(cellH));
  }
}

function inspectCell(cell) {
  $("cell-inspector-hint").style.display = "none";
  $("cell-inspector-body").innerHTML = [
    ["Coordinates", `Row ${cell.row}, Col ${cell.col}`],
    ["Biome ID", `${cell.biome_id}`],
    ["Total SOMSC", `${fmt(cell.somsc, 1)} g C/m²`],
    ["Passive SOM3", `${fmt(cell.som3c, 1)} g C/m²`],
    ["Large Wood", `${fmt(cell.rlwodc, 1)} g C/m²`],
    ["Annual NPP", `${fmt(cell.npp, 1)} g C/m²/yr`],
    ["Annual AET", `${fmt(cell.aet, 1)} cm/yr`],
    ["Soil Water (L0)", `${fmt(cell.soil_water_cm, 2)} cm`],
    ["Mineral N (L0)", `${fmt(cell.minerl_n, 3)} g N/m²`]
  ].map(([k, v]) => `<div class="insp-row"><span>${k}</span><span>${v}</span></div>`).join("");
}

async function runSpatialGrid() {
  const btn = $("btn-run-grid");
  btn.disabled = true;
  btn.textContent = t("btn_running");
  try {
    const useAttached = $("grid-source").value === "attached";
    const req = {
      use_attached_grid: useAttached,
      rows: Number($("grid-rows").value),
      cols: Number($("grid-cols").value),
      years: Number($("grid-years").value),
      threads: Number($("grid-threads").value) || null,
      chunk_size: Number($("grid-chunk").value),
      profile: $("grid-prof").value,
      precision: $("grid-prec").value
    };
    const res = await fetch("/api/grid/run", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req)
    });
    if (!res.ok) throw new Error(await res.text());
    const data = await res.json();
    state.lastGrid = data;
    state.gridCellMap = new Map(data.cells.map((c) => [`${c.row}:${c.col}`, c]));

    $("gkpi-cells").textContent = `${data.ncells} × ${data.nyears}`;
    $("gkpi-source").textContent = data.source_label;
    $("gkpi-time").textContent = `${fmt(data.elapsed_secs, 3)} s`;
    $("gkpi-threads").textContent = `${data.threads_used} threads`;
    $("gkpi-rate").textContent = `${fmt(data.cell_years_per_sec, 0)}`;
    $("gkpi-mo-rate").textContent = `${fmt(data.cell_months_per_sec / 1e6, 2)}M cell-mo/s`;
    $("gkpi-hpc").textContent = `${fmt(data.hpc_256t_est_cell_yr_s / 1e6, 2)}M`;
    $("gkpi-som").textContent = `${fmt(data.mean_somsc, 0)} g C/m²`;
    $("gkpi-npp").textContent = `NPP: ${fmt(data.mean_npp, 1)} g C/m²/yr`;

    drawGridHeatmap();
    if (data.cells.length) inspectCell(data.cells[Math.floor(data.cells.length / 2)]);
  } catch (err) {
    alert(`Grid error: ${err.message}`);
  } finally {
    btn.disabled = false;
    btn.textContent = t("btn_run_grid");
  }
}

function downloadBlob(filename, content, mime) {
  const blob = new Blob([content], { type: mime });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
  URL.revokeObjectURL(a.href);
}

// ==================== INITIALIZATION ====================
async function init() {
  // Tab switching
  document.querySelectorAll(".nav-tab").forEach((btn) => {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".nav-tab").forEach((b) => b.classList.remove("active"));
      document.querySelectorAll(".panel").forEach((p) => p.classList.remove("active"));
      btn.classList.add("active");
      const target = btn.getAttribute("data-tab");
      $(`panel-${target}`).classList.add("active");
      if (target === "grid" && !state.lastGrid) runSpatialGrid();
    });
  });

  $("lang-en").addEventListener("click", () => applyLang("en"));
  $("lang-pt").addEventListener("click", () => applyLang("pt"));

  document.querySelectorAll('input[type="range"]').forEach((inp) => {
    inp.addEventListener("input", syncSliderLabels);
  });

  $("preset-select").addEventListener("change", (e) => {
    loadPresetIntoControls(e.target.value);
    runSimulation();
  });

  $("btn-run").addEventListener("click", runSimulation);
  $("btn-pin").addEventListener("click", () => {
    if (!state.lastSim) return;
    state.baselineSim = state.lastSim;
    $("btn-pin").textContent = t("btn_pinned");
    $("btn-clear-pin").style.display = "inline-block";
    renderSimulation(state.lastSim);
  });
  $("btn-clear-pin").addEventListener("click", () => {
    state.baselineSim = null;
    $("btn-pin").textContent = t("btn_pin");
    $("btn-clear-pin").style.display = "none";
    if (state.lastSim) renderSimulation(state.lastSim);
  });

  $("btn-export-csv").addEventListener("click", () => {
    if (!state.lastSim) return;
    const cols = Object.keys(state.lastSim.series[0]);
    const lines = [cols.join(","), ...state.lastSim.series.map((r) => cols.map((c) => r[c]).join(","))];
    downloadBlob("oxisol_scenario.csv", lines.join("\n"), "text/csv");
  });
  $("btn-export-json").addEventListener("click", () => {
    if (!state.lastSim) return;
    downloadBlob("oxisol_scenario.json", JSON.stringify(state.lastSim, null, 2), "application/json");
  });

  $("btn-run-grid").addEventListener("click", runSpatialGrid);
  $("map-layer").addEventListener("change", drawGridHeatmap);
  $("grid-source").addEventListener("change", () => {
    $("synth-dims").style.display = $("grid-source").value === "attached" ? "none" : "grid";
  });
  $("btn-grid-csv").addEventListener("click", () => {
    if (!state.lastGrid) return;
    const cols = Object.keys(state.lastGrid.cells[0]);
    const lines = [cols.join(","), ...state.lastGrid.cells.map((r) => cols.map((c) => r[c]).join(","))];
    downloadBlob("oxisol_grid.csv", lines.join("\n"), "text/csv");
  });

  $("grid-canvas").addEventListener("click", (ev) => {
    if (!state.lastGrid) return;
    const rect = ev.target.getBoundingClientRect();
    const col = Math.floor(((ev.clientX - rect.left) / rect.width) * state.lastGrid.cols);
    const row = Math.floor(((ev.clientY - rect.top) / rect.height) * state.lastGrid.rows);
    const cell = state.gridCellMap.get(`${row}:${col}`);
    if (cell) inspectCell(cell);
  });

  // Fetch system info & presets in parallel
  const [sysRes, preRes] = await Promise.all([fetch("/api/system"), fetch("/api/presets")]);
  state.system = await sysRes.json();
  state.presets = await preRes.json();

  $("hw-pill").textContent = `${state.system.logical_cpus}T · CellState ${state.system.cell_state_f64_bytes} B`;
  $("grid-threads").value = state.system.logical_cpus;
  $("grid-threads").max = state.system.logical_cpus;
  if (state.system.has_attached_grid) {
    $("opt-attached-grid").disabled = false;
    $("grid-source").value = "attached";
    $("synth-dims").style.display = "none";
  }

  $("sys-meta").innerHTML = [
    ["Engine", `${state.system.name} v${state.system.version}`],
    ["Logical CPU Threads", `${state.system.logical_cpus}`],
    ["CellState<f64> Footprint", `${state.system.cell_state_f64_bytes} bytes (64B L1d aligned)`],
    ["CellState<f32> Footprint", `${state.system.cell_state_f32_bytes} bytes (64B L1d aligned)`],
    ["Safety Guarantee", "#![forbid(unsafe_code)] · Pure Rust"],
    ["Attached Spatial Grid", state.system.attached_grid_label || "None (built-in synthetic grid active)"]
  ].map(([k, v]) => `<div class="insp-row"><span>${k}</span><span>${v}</span></div>`).join("");

  populatePresetSelect(false);
  loadPresetIntoControls(state.presets[0].id);
  runSimulation();
}

init();
