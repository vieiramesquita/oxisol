// Oxisol Studio — Zero-dependency Interactive Scientific Frontend (EN / PT-BR)

const I18N = {
  en: {
    brand_sub: "Open-Source CENTURY-like Biogeochemical Engine in Rust (C · N · P · S)",
    tab_lab: "Scenario Lab",
    tab_grid: "Spatial Grid & HPC",
    tab_lulc: "LULC & Raster (GeoTIFF)",
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
    climo_lbl: "12-Month Climatology Preview",
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
    kpi_somsc: "Total Soil Carbon (SOMSC · 0–20 cm)",
    kpi_npp: "Annual NPP",
    kpi_aet: "Actual ET (AET)",
    kpi_minn: "Profile Mineral N",
    kpi_mb: "Mass Balance Residual",
    kpi_mb_sub: "Exact C·N·P·S Audit",
    kpi_speed: "Engine Speed",
    chart_som_title: "Soil Organic Carbon Pools",
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
    ep_presets: "8 calibrated ecosystem presets (EN/PT-BR)",
    ep_sim: "Single-cell scenario trajectory + mass balance audit",
    ep_grid: "Lock-free multi-core spatial grid execution",
    ep_lulc_s: "MapBiomas & ESA WorldCover LULC schemes",
    ep_lulc_r: "Spatial LULC + modeled climate GeoTIFF/COG engine (terrio)",
    lulc_src_title: "LULC & Modeled Climate Source",
    lulc_mode: "Execution Mode",
    lulc_mode_demo: "Amazon–Cerrado Frontier LUC Demo (1990–2024)",
    lulc_mode_tif: "Real GeoTIFF / COG Rasters (terrio pure-Rust)",
    lulc_scheme: "Classification Scheme",
    lulc_paths_lbl: "LULC GeoTIFF Path(s) (1 per line for multi-year LUC)",
    lulc_precip_path: "Precipitation Raster (12 or 12×N bands)",
    lulc_punit: "Precip Unit",
    lulc_tunit: "Temp Unit",
    lulc_sg: "Auto-scale SoilGrids units (g/kg, cg/cm³, pH×10)",
    lulc_scen_title: "Climate, LUC & COG Export",
    lulc_spinup: "Spinup (yr)",
    lulc_trans: "Transient (yr)",
    lulc_dt: "Warming Shift (ΔT)",
    lulc_pscale: "Rainfall Factor",
    lulc_res_lbl: "Modeling Resolution",
    lulc_res_native: "Native (Raster)",
    lulc_hw_lbl: "Compute Profile",
    lulc_hw_max: "Max Performance (All Cores)",
    lulc_hw_lap: "Laptop Quiet (50% Cores)",
    lulc_filter_lbl: "Filter Classes (opt. e.g. 15 = Pasture, or 3,4,15)",
    lulc_till: "Cropland Tillage Practice",
    lulc_export_dir: "Export Cloud-Optimized GeoTIFFs to Dir (optional)",
    btn_run_lulc: "Run Spatial LULC Simulation",
    lulc_legend_title: "LULC Class Legend",
    lulc_insp_title: "Pixel Trajectory Inspector",
    lkpi_pixels: "Active Pixels",
    lkpi_som: "Mean Final SOMSC",
    lkpi_delta: "Net Carbon Change (ΔSOMSC)",
    lkpi_cog: "COG GeoTIFF Export",
    lulc_map_title: "Georeferenced LULC & Soil Carbon Map (EPSG:4326)",
    api_example_title: "Quick Python Example",
    sys_card_title: "Architecture & Documentation",
    hpc_title: "Running Remotely on Modern High-Performance Servers",
    docs_pointer: "Full scientific documentation (mdBook) lives in docs/src/ and API reference builds via cargo doc --workspace --no-deps. Released under the MIT License."
  },
  pt: {
    brand_sub: "Engine Biogeoquímica CENTURY-like Open-Source em Rust (C · N · P · S)",
    tab_lab: "Laboratório de Cenários",
    tab_grid: "Grade Espacial & HPC",
    tab_lulc: "Mapas LULC & Clima (GeoTIFF)",
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
    climo_lbl: "Climatologia Mensal (12 Meses)",
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
    kpi_somsc: "Carbono Total do Solo (SOMSC · 0–20 cm)",
    kpi_npp: "NPP Anual",
    kpi_aet: "Evapotranspiração Real (AET)",
    kpi_minn: "N Mineral no Perfil",
    kpi_mb: "Resíduo de Balanço de Massa",
    kpi_mb_sub: "Auditoria Exata C·N·P·S",
    kpi_speed: "Velocidade da Engine",
    chart_som_title: "Pools de Carbono Orgânico do Solo",
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
    ep_presets: "8 presets calibrados de ecossistemas (EN/PT-BR)",
    ep_sim: "Trajetória de célula única + auditoria de balanço de massa",
    ep_grid: "Execução espacial multi-core lock-free via rayon",
    ep_lulc_s: "Esquemas de classes MapBiomas & ESA WorldCover",
    ep_lulc_r: "Engine espacial LULC + clima modelado em GeoTIFF/COG (terrio)",
    lulc_src_title: "Fonte LULC & Clima Modelado",
    lulc_mode: "Modo de Execução",
    lulc_mode_demo: "Demo Fronteira Amazônia–Cerrado (1990–2024)",
    lulc_mode_tif: "Rasters GeoTIFF / COG Reais (terrio pure-Rust)",
    lulc_scheme: "Esquema de Classificação",
    lulc_paths_lbl: "Caminho(s) GeoTIFF LULC (1 por linha para série anual)",
    lulc_precip_path: "Raster de Precipitação (12 ou 12×N bandas)",
    lulc_punit: "Unidade Precip.",
    lulc_tunit: "Unidade Temp.",
    lulc_sg: "Auto-escalar SoilGrids (g/kg, cg/cm³, pH×10)",
    lulc_scen_title: "Clima, Transições & Exportação COG",
    lulc_spinup: "Spinup (anos)",
    lulc_trans: "Transiente (anos)",
    lulc_dt: "Aquecimento (ΔT)",
    lulc_pscale: "Fator de Chuva",
    lulc_res_lbl: "Resolução de Modelagem",
    lulc_res_native: "Nativa (do Raster)",
    lulc_hw_lbl: "Perfil de Hardware",
    lulc_hw_max: "Desempenho Máximo (Todos Núcleos)",
    lulc_hw_lap: "Notebook Silencioso (50% Núcleos)",
    lulc_filter_lbl: "Filtrar Classes (opc. ex.: 15 = Pastagem, ou 3,4,15)",
    lulc_till: "Preparo na Lavoura",
    lulc_export_dir: "Exportar GeoTIFFs COG na pasta (opcional)",
    btn_run_lulc: "Rodar Simulação Espacial LULC",
    lulc_legend_title: "Legenda de Classes LULC",
    lulc_insp_title: "Inspetor de Trajetória do Pixel",
    lkpi_pixels: "Pixels Ativos",
    lkpi_som: "SOMSC Médio Final",
    lkpi_delta: "Balanço de Carbono (ΔSOMSC)",
    lkpi_cog: "Exportação COG GeoTIFF",
    lulc_map_title: "Mapa Georreferenciado de LULC & Carbono do Solo (EPSG:4326)",
    api_example_title: "Exemplo Rápido em Python",
    sys_card_title: "Arquitetura & Documentação",
    hpc_title: "Execução Remota em Servidores Modernos de Alto Desempenho",
    docs_pointer: "A documentação científica completa (mdBook) está em docs/src/ e a referência de API é gerada via cargo doc --workspace --no-deps. Licença MIT."
  }
};

const state = {
  lang: "en",
  carbonUnit: "mgha", // "mgha" (Mg C/ha, default) or "gm2" (g C/m²)
  system: null,
  presets: [],
  lulcSchemes: [],
  lastSim: null,
  baselineSim: null,
  lastGrid: null,
  selectedGridCell: null,
  gridCellMap: new Map(),
  lastLulc: null,
  selectedLulcPixel: null,
  lulcPixelMap: new Map()
};

const CARBON_LAYERS = new Set([
  "somsc",
  "som3c",
  "rlwodc",
  "npp",
  "somsc_final",
  "somsc_0_30",
  "somsc_0_100",
  "delta_somsc",
  "delta_somsc_0_30",
  "biomass_c"
]);
const CO2E_FACTOR = 44.0 / 12.0;

function cScale(val) {
  return state.carbonUnit === "mgha" ? val / 100.0 : val;
}
function cUnit() {
  return state.carbonUnit === "mgha" ? "Mg C/ha" : "g C/m²";
}
function cFluxUnit() {
  const yr = state.lang === "pt" ? "ano" : "yr";
  return state.carbonUnit === "mgha" ? `Mg C/ha/${yr}` : `g C/m²/${yr}`;
}
function cDigits(isFlux = false) {
  if (state.carbonUnit === "mgha") return isFlux ? 2 : 1;
  return isFlux ? 1 : 0;
}
function isCarbonLayer(layer) {
  return CARBON_LAYERS.has(layer);
}

function syncUnitLabels() {
  const u = cUnit();
  const fu = cFluxUnit();
  const somHead = $("chart-som-head");
  if (somHead) somHead.textContent = `${t("chart_som_title")} (${u})`;
  const lkpiSomUnit = $("lkpi-som-unit");
  if (lkpiSomUnit) lkpiSomUnit.textContent = u;

  const mapSel = $("map-layer");
  if (mapSel) {
    const labels = {
      somsc: `Total SOMSC (${u})`,
      som3c: `Passive SOM3 (${u})`,
      rlwodc: `Forest Large Wood (${u})`,
      npp: `Annual NPP (${fu})`
    };
    for (const opt of mapSel.options) {
      if (labels[opt.value]) opt.textContent = labels[opt.value];
    }
  }
  const lulcSel = $("lulc-layer");
  if (lulcSel) {
    const labels = {
      delta_somsc: `ΔSOMSC 0–20 cm (${u})`,
      delta_somsc_0_30: `ΔSOMSC 0–30 cm IPCC (${u})`,
      somsc_final: `Final SOMSC 0–20 cm (${u})`,
      somsc_0_30: `Final SOMSC 0–30 cm IPCC (${u})`,
      somsc_0_100: `Final SOMSC 0–100 cm Deep (${u})`,
      biomass_c: `Live Biomass C (${u})`,
      npp: `Annual NPP (${fu})`
    };
    for (const opt of lulcSel.options) {
      if (labels[opt.value]) opt.textContent = labels[opt.value];
    }
  }
}

function applyCarbonUnit(unit) {
  state.carbonUnit = unit;
  $("unit-mgha").classList.toggle("active", unit === "mgha");
  $("unit-gm2").classList.toggle("active", unit === "gm2");
  syncUnitLabels();
  if (state.lastSim) renderSimulation(state.lastSim);
  if (state.lastGrid) {
    renderGridSummary(state.lastGrid);
    drawGridHeatmap();
    if (state.selectedGridCell) inspectCell(state.selectedGridCell);
  }
  if (state.lastLulc) {
    renderLulcSummary(state.lastLulc);
    drawLulcMap();
    if (state.selectedLulcPixel) inspectLulcPixel(state.selectedLulcPixel);
  }
}

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
  populateLulcSchemeSelect(true);
  syncUnitLabels();
  syncSliderLabels();
  if (state.lastSim) renderSimulation(state.lastSim);
  if (state.lastGrid) {
    renderGridSummary(state.lastGrid);
    if (state.selectedGridCell) inspectCell(state.selectedGridCell);
  }
  if (state.lastLulc) {
    renderLulcSummary(state.lastLulc);
    if (state.selectedLulcPixel) inspectLulcPixel(state.selectedLulcPixel);
  }
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

function lerpRamp(u, stops) {
  const c = Math.max(0, Math.min(1, u));
  for (let i = 0; i < stops.length - 1; i++) {
    const [p0, c0] = stops[i];
    const [p1, c1] = stops[i + 1];
    if (c <= p1) {
      const t = (c - p0) / Math.max(p1 - p0, 1e-6);
      const r = Math.round(c0[0] + (c1[0] - c0[0]) * t);
      const g = Math.round(c0[1] + (c1[1] - c0[1]) * t);
      const b = Math.round(c0[2] + (c1[2] - c0[2]) * t);
      return `rgb(${r}, ${g}, ${b})`;
    }
  }
  const last = stops[stops.length - 1][1];
  return `rgb(${last[0]}, ${last[1]}, ${last[2]})`;
}

const TEMP_STOPS = [
  [0.0, [56, 189, 248]],
  [0.22, [34, 211, 238]],
  [0.42, [16, 185, 129]],
  [0.66, [245, 158, 11]],
  [0.83, [249, 115, 22]],
  [1.0, [239, 68, 68]]
];
const AMP_STOPS = [
  [0.0, [20, 184, 166]],
  [0.35, [56, 189, 248]],
  [0.7, [245, 158, 11]],
  [1.0, [225, 29, 72]]
];
const PRECIP_STOPS = [
  [0.0, [203, 213, 225]],
  [0.18, [148, 163, 184]],
  [0.42, [125, 211, 252]],
  [0.72, [2, 132, 199]],
  [1.0, [30, 58, 138]]
];
const LAT_STOPS = [
  [0.0, [96, 165, 250]],
  [0.28, [52, 211, 153]],
  [0.46, [245, 158, 11]],
  [0.66, [52, 211, 153]],
  [1.0, [96, 165, 250]]
];

function styleSliderValue(inputEl, pillEl, color, textOverride) {
  if (inputEl) inputEl.style.setProperty("--thumb-color", color);
  if (pillEl) {
    pillEl.style.color = textOverride || color;
    pillEl.style.borderColor = color;
  }
}

function renderLatGlobe(lat) {
  const host = $("lat-globe");
  if (!host) return;
  const rad = (lat * Math.PI) / 180;
  const cx = 33, cy = 33, R = 27;
  const y = cy - R * Math.sin(rad);
  const rx = Math.max(2.5, R * Math.cos(rad));
  const ry = Math.max(1.2, rx * 0.22);
  const absLat = Math.abs(lat);
  const latNorm = (lat + 65) / 140;
  const col = lerpRamp(latNorm, LAT_STOPS);

  const eqY = cy;
  const trN = cy - R * Math.sin((23.5 * Math.PI) / 180);
  const trS = cy + R * Math.sin((23.5 * Math.PI) / 180);
  const dotX = cx - rx * 0.28; // Brazil / Americas longitude focus

  host.innerHTML = `<svg viewBox="0 0 66 66" role="img" aria-label="Earth latitude ${lat.toFixed(1)}">
    <defs>
      <radialGradient id="oxGlobeBg" cx="35%" cy="30%" r="72%">
        <stop offset="0%" stop-color="#1d4ed8" />
        <stop offset="55%" stop-color="#0f294a" />
        <stop offset="100%" stop-color="#071324" />
      </radialGradient>
      <clipPath id="oxGlobeClip"><circle cx="${cx}" cy="${cy}" r="${R}" /></clipPath>
    </defs>
    <circle cx="${cx}" cy="${cy}" r="${R + 1.5}" fill="none" stroke="rgba(56,189,248,0.22)" stroke-width="1.2" />
    <circle cx="${cx}" cy="${cy}" r="${R}" fill="url(#oxGlobeBg)" stroke="#2c4464" stroke-width="1" />
    <g clip-path="url(#oxGlobeClip)">
      <!-- Stylized Americas + Africa/Europe silhouettes centered near Brazil/Atlantic -->
      <path d="M12,11 L21,9 L26,15 L23,23 L17,26 L13,22 Z" fill="#1e5c45" opacity="0.88" />
      <path d="M19,29 L27,30 L32,37 L29,48 L24,56 L21,53 L20,41 L17,34 Z" fill="#237a57" opacity="0.92" />
      <path d="M41,11 L53,13 L56,22 L47,25 L41,20 Z" fill="#1e5c45" opacity="0.72" />
      <path d="M40,26 L53,27 L57,38 L51,51 L44,48 L41,37 Z" fill="#1e5c45" opacity="0.78" />
      <!-- Reference latitudes: Tropics + Equator -->
      <line x1="${cx - R}" y1="${trN.toFixed(1)}" x2="${cx + R}" y2="${trN.toFixed(1)}" stroke="rgba(251,191,36,0.28)" stroke-width="0.7" stroke-dasharray="2,2" />
      <line x1="${cx - R}" y1="${trS.toFixed(1)}" x2="${cx + R}" y2="${trS.toFixed(1)}" stroke="rgba(251,191,36,0.28)" stroke-width="0.7" stroke-dasharray="2,2" />
      <line x1="${cx - R}" y1="${eqY}" x2="${cx + R}" y2="${eqY}" stroke="rgba(255,255,255,0.4)" stroke-width="0.85" stroke-dasharray="3,2" />
      <!-- Active Latitude Parallel & Glowing Marker -->
      <ellipse cx="${cx}" cy="${y.toFixed(1)}" rx="${rx.toFixed(1)}" ry="${ry.toFixed(1)}" fill="none" stroke="${col}" stroke-width="1.6" />
      <circle cx="${dotX.toFixed(1)}" cy="${y.toFixed(1)}" r="4.6" fill="${col}" opacity="0.28" />
      <circle cx="${dotX.toFixed(1)}" cy="${y.toFixed(1)}" r="2.5" fill="${col}" stroke="#071324" stroke-width="0.9" />
    </g>
    <circle cx="${cx}" cy="${cy}" r="${R}" fill="none" stroke="rgba(255,255,255,0.16)" stroke-width="0.9" />
  </svg>`;

  const pt = state.lang === "pt";
  let zone = pt ? "Boreal / Polar" : "Boreal / Subpolar";
  if (absLat < 7) zone = pt ? "Equatorial" : "Equatorial";
  else if (absLat < 23.5) zone = pt ? "Tropical" : "Tropical";
  else if (absLat < 35) zone = pt ? "Subtropical" : "Subtropical";
  else if (absLat < 55) zone = pt ? "Temperado" : "Temperate";

  const zBadge = $("lat-zone-badge");
  const hBadge = $("lat-hemi-badge");
  if (zBadge) {
    zBadge.textContent = zone;
    zBadge.style.color = col;
    zBadge.style.borderColor = col;
  }
  if (hBadge) {
    hBadge.textContent = lat < -0.2
      ? (pt ? "Sul · Austral" : "SH · Austral")
      : lat > 0.2
        ? (pt ? "Norte · Boreal" : "NH · Boreal")
        : (pt ? "Equador 0°" : "Equator 0°");
  }
}

function renderMiniClimograph(meanT, ampT, precipCm, lat) {
  const host = $("mini-climograph");
  if (!host) return;
  const peakMonth = lat >= 0 ? 6 : 0;
  const seas = Math.abs(lat) < 25 ? 0.55 : 0.28;
  const weights = Array.from({ length: 12 }, (_, m) => {
    const ph = (2 * Math.PI * (m - peakMonth)) / 12;
    return Math.max(0.15, 1 + seas * Math.cos(ph));
  });
  const wSum = weights.reduce((a, b) => a + b, 0);
  const precips = weights.map((w) => (Math.max(1, precipCm) * w) / wSum);
  const temps = Array.from({ length: 12 }, (_, m) => {
    const ph = (2 * Math.PI * (m - peakMonth)) / 12;
    return meanT + ampT * Math.cos(ph);
  });

  const W = 280, H = 64, padL = 6, padR = 6, padT = 8, padB = 14;
  const plotW = W - padL - padR;
  const plotH = H - padT - padB;
  const maxP = Math.max(5, ...precips) * 1.15;
  const minT = Math.min(-10, ...temps) - 2;
  const maxT = Math.max(35, ...temps) + 2;
  const barW = plotW / 12 - 3;
  const pCol = lerpRamp((precipCm - 5) / 595, PRECIP_STOPS);
  const tCol = lerpRamp((meanT + 15) / 53, TEMP_STOPS);

  const months = state.lang === "pt"
    ? ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"]
    : ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"];

  const bars = precips.map((p, m) => {
    const x = padL + (m + 0.5) * (plotW / 12) - barW / 2;
    const h = Math.max(2, (p / maxP) * plotH);
    const y = padT + plotH - h;
    return `<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${barW.toFixed(1)}" height="${h.toFixed(1)}" rx="1.5" fill="${pCol}" opacity="0.65" />`;
  }).join("");

  const pts = temps.map((tv, m) => {
    const x = padL + (m + 0.5) * (plotW / 12);
    const y = padT + plotH - ((tv - minT) / Math.max(maxT - minT, 1)) * plotH;
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  }).join(" ");

  const labels = months.map((lbl, m) => {
    const x = padL + (m + 0.5) * (plotW / 12);
    return `<text x="${x.toFixed(1)}" y="${H - 2}" text-anchor="middle" fill="#64748b" font-size="8" font-family="var(--mono)">${lbl}</text>`;
  }).join("");

  host.innerHTML = `<svg viewBox="0 0 ${W} ${H}">${bars}<polyline points="${pts}" fill="none" stroke="${tCol}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />${labels}</svg>`;

  const pBadge = $("precip-regime-badge");
  if (pBadge) {
    const pt = state.lang === "pt";
    let reg = pt ? "Hiperúmido" : "Hyper-humid";
    if (precipCm < 25) reg = pt ? "Árido" : "Arid";
    else if (precipCm < 55) reg = pt ? "Semiárido" : "Semi-arid";
    else if (precipCm < 110) reg = pt ? "Subúmido" : "Sub-humid";
    else if (precipCm < 220) reg = pt ? "Úmido" : "Humid";
    pBadge.textContent = reg;
    pBadge.style.color = precipCm > 380 ? "#93c5fd" : pCol;
    pBadge.style.borderColor = pCol;
  }
}

function updatePhLitmus() {
  const phEl = $("in-ph");
  const needle = $("ph-needle");
  const badge = $("ph-badge");
  if (!phEl || !needle) return;
  const ph = Math.max(3.8, Math.min(9.2, Number(phEl.value) || 5.5));
  const u = (ph - 3.8) / (9.2 - 3.8);
  needle.style.left = `${(u * 100).toFixed(1)}%`;
  if (badge) {
    const pt = state.lang === "pt";
    let lbl = pt ? "Alcalino" : "Alkaline";
    let col = "#a78bfa";
    if (ph < 5.0) { lbl = pt ? "Muito Ácido" : "Strong Acid"; col = "#f87171"; }
    else if (ph < 6.2) { lbl = pt ? "Ácido" : "Acidic"; col = "#fb923c"; }
    else if (ph <= 7.4) { lbl = pt ? "Neutro" : "Neutral"; col = "#34d399"; }
    badge.textContent = lbl;
    badge.style.color = col;
  }
}

function syncSliderLabels() {
  const temp = Number($("in-temp").value);
  const amp = Number($("in-amp").value);
  const precip = Number($("in-precip").value);
  const lat = Number($("in-lat").value);

  $("val-temp").textContent = `${temp.toFixed(1)} °C`;
  $("val-amp").textContent = `${amp.toFixed(1)} °C`;
  $("val-precip").textContent = `${precip} cm`;
  $("val-lat").textContent = `${Math.abs(lat).toFixed(1)}°${lat >= 0 ? "N" : "S"}`;
  $("val-rock").textContent = `${$("in-rock").value}%`;
  $("val-drain").textContent = Number($("in-drain").value).toFixed(2);
  $("val-years").textContent = `${$("in-years").value} yr`;
  $("val-grid-years").textContent = `${$("grid-years").value} yr`;

  const ldt = Number($("lulc-dt").value);
  const lps = Number($("lulc-pscale").value);
  $("val-lulc-dt").textContent = `${ldt >= 0 ? "+" : ""}${ldt.toFixed(2)} °C`;
  $("val-lulc-pscale").textContent = `${lps}%`;

  // Dynamic semantic slider & badge coloring
  styleSliderValue($("in-temp"), $("val-temp"), lerpRamp((temp + 15) / 53, TEMP_STOPS));
  styleSliderValue($("in-amp"), $("val-amp"), lerpRamp((amp - 0.5) / 29.5, AMP_STOPS));
  const precipCol = lerpRamp((precip - 5) / 595, PRECIP_STOPS);
  styleSliderValue($("in-precip"), $("val-precip"), precipCol, precip > 380 ? "#93c5fd" : precipCol);
  styleSliderValue($("in-lat"), $("val-lat"), lerpRamp((lat + 65) / 140, LAT_STOPS));
  styleSliderValue($("lulc-dt"), $("val-lulc-dt"), lerpRamp((ldt + 2) / 8, TEMP_STOPS));
  const lpsCol = lerpRamp((lps - 60) / 80, PRECIP_STOPS);
  styleSliderValue($("lulc-pscale"), $("val-lulc-pscale"), lpsCol, lps > 120 ? "#93c5fd" : lpsCol);

  renderLatGlobe(lat);
  renderMiniClimograph(temp, amp, precip, lat);
  updatePhLitmus();
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
    const yLabel = yv >= 1000 ? (yv / 1000).toFixed(1) + "k" : maxY < 25 ? yv.toFixed(1) : yv.toFixed(0);
    gridSvg += `<text x="${pad.l - 7}" y="${yp + 4}" text-anchor="end" fill="#8b9bb4" font-size="10" font-family="monospace">${yLabel}</text>`;
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
  const u = cUnit();
  const dStock = cDigits(false);
  const dFlux = cDigits(true);
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
    <text x="125" y="49" text-anchor="middle" fill="#e6edf3" font-size="11" font-family="monospace">NPP ${fmt(cScale(pools.final_npp), dFlux)} · Resp ${fmt(cScale(pools.final_resp), dFlux)}</text>

    <!-- Live Vegetation -->
    <rect x="28" y="100" width="186" height="58" rx="9" fill="#112620" stroke="#10b981" stroke-width="1.6"/>
    <text x="121" y="120" text-anchor="middle" fill="#10b981" font-size="11" font-weight="700">LIVE VEGETATION</text>
    <text x="121" y="143" text-anchor="middle" fill="#e6edf3" font-size="14.5" font-weight="700" font-family="monospace">${fmt(cScale(pools.vegetation_c), dStock)} ${u}</text>

    <!-- Litter & Dead Wood -->
    <rect x="262" y="100" width="128" height="58" rx="9" fill="#17202d" stroke="#94a3b8" stroke-width="1.4"/>
    <text x="326" y="120" text-anchor="middle" fill="#cbd5e1" font-size="10.5" font-weight="700">LITTER &amp; WOOD</text>
    <text x="326" y="143" text-anchor="middle" fill="#e6edf3" font-size="13.5" font-weight="700" font-family="monospace">${fmt(cScale(pools.litter_and_deadwood_c), dStock)} ${state.carbonUnit === "mgha" ? "Mg" : "g"}</text>

    <!-- SOM1 Active -->
    <rect x="446" y="20" width="172" height="54" rx="9" fill="#102520" stroke="#10b981" stroke-width="1.5"/>
    <text x="532" y="39" text-anchor="middle" fill="#10b981" font-size="10.5" font-weight="700">SOM1 ACTIVE</text>
    <text x="532" y="61" text-anchor="middle" fill="#e6edf3" font-size="14" font-weight="700" font-family="monospace">${fmt(cScale(pools.som1_c), state.carbonUnit === "mgha" ? 2 : 1)} ${u}</text>

    <!-- SOM2 Slow -->
    <rect x="446" y="102" width="172" height="52" rx="9" fill="#251d10" stroke="#f59e0b" stroke-width="1.5"/>
    <text x="532" y="120" text-anchor="middle" fill="#f59e0b" font-size="10.5" font-weight="700">SOM2 SLOW</text>
    <text x="532" y="142" text-anchor="middle" fill="#e6edf3" font-size="14" font-weight="700" font-family="monospace">${fmt(cScale(pools.som2_c), dStock)} ${u}</text>

    <!-- SOM3 Passive -->
    <rect x="446" y="180" width="172" height="54" rx="9" fill="#1d152c" stroke="#8b5cf6" stroke-width="1.5"/>
    <text x="532" y="199" text-anchor="middle" fill="#a78bfa" font-size="10.5" font-weight="700">SOM3 PASSIVE</text>
    <text x="532" y="221" text-anchor="middle" fill="#e6edf3" font-size="14" font-weight="700" font-family="monospace">${fmt(cScale(pools.som3_c), dStock)} ${u}</text>

    <!-- Multi-Depth Profile (0-20 / 0-30 IPCC / 0-100 Deep) & Mineral N/P/S -->
    <rect x="28" y="182" width="362" height="58" rx="9" fill="#101c28" stroke="#06b6d4" stroke-width="1.4"/>
    <text x="209" y="199" text-anchor="middle" fill="#06b6d4" font-size="10" font-weight="700">MULTI-DEPTH PROFILE (${u}) · MINERAL N/P/S</text>
    <text x="209" y="216" text-anchor="middle" fill="#10b981" font-size="11.5" font-weight="700" font-family="monospace">0–20cm: ${fmt(cScale(pools.somsc), dStock)} · IPCC 0–30: ${fmt(cScale(pools.depth_profile ? pools.depth_profile.somsc_0_30 : pools.somsc), dStock)} · Deep 0–100: ${fmt(cScale(pools.depth_profile ? pools.depth_profile.somsc_0_100 : pools.somsc), dStock)}</text>
    <text x="209" y="232" text-anchor="middle" fill="#94a3b8" font-size="10.5" font-family="monospace">Mineral (g/m²) — N: ${fmt(pools.profile_minerl_n, 2)} · P: ${fmt(pools.profile_minerl_p, 2)} · S: ${fmt(pools.profile_minerl_s, 2)}</text>
  </svg>`;
}

function renderSimulation(sim) {
  const fp = sim.final_pools;
  const bp = state.baselineSim ? state.baselineSim.final_pools : null;
  const u = cUnit();
  const fu = cFluxUnit();
  const dStock = cDigits(false);
  const dFlux = cDigits(true);

  $("kpi-somsc").textContent = fmt(cScale(fp.somsc), dStock);
  renderDelta("kpi-somsc-delta", cScale(fp.somsc), bp ? cScale(bp.somsc) : null, u);

  $("kpi-npp").textContent = fmt(cScale(fp.final_npp), dFlux);
  renderDelta("kpi-npp-delta", cScale(fp.final_npp), bp ? cScale(bp.final_npp) : null, fu);

  $("kpi-aet").textContent = fmt(fp.final_aet, 1);
  renderDelta("kpi-aet-delta", fp.final_aet, bp ? bp.final_aet : null, "cm/yr");

  $("kpi-minn").textContent = fmt(fp.profile_minerl_n, 2);
  renderDelta("kpi-minn-delta", fp.profile_minerl_n, bp ? bp.profile_minerl_n : null, "g N/m²");

  $("kpi-mb").textContent = fmtSci(fp.max_mass_balance_err);
  $("kpi-speed").textContent = `${fmt(sim.throughput_cell_yr_s, 0)}`;
  $("kpi-speed-sub").textContent = `${fmt(sim.elapsed_ms, 1)} ms · ${sim.precision}`;

  const x = sim.series.map((p) => p.year);
  const baseMatch = state.baselineSim && state.baselineSim.series.length === sim.series.length;

  const depthMode = ($("som-depth-mode") && $("som-depth-mode").value) || "deep";
  const somSeries = depthMode === "classic"
    ? [
        { label: "Total SOMSC (0–20)", color: "#10b981", width: 2.5, data: sim.series.map((p) => cScale(p.somsc)) },
        { label: "SOM2 Slow", color: "#f59e0b", width: 1.9, data: sim.series.map((p) => cScale(p.som2c)) },
        { label: "SOM3 Passive", color: "#8b5cf6", width: 1.9, data: sim.series.map((p) => cScale(p.som3c)) },
        { label: "SOM1 Active", color: "#06b6d4", width: 1.5, data: sim.series.map((p) => cScale(p.som1c)) }
      ]
    : [
        { label: "Deep 0–100 cm", color: "#8b5cf6", width: 2.5, data: sim.series.map((p) => cScale(p.somsc_0_100 || p.somsc)) },
        { label: "IPCC 0–30 cm", color: "#06b6d4", width: 2.2, data: sim.series.map((p) => cScale(p.somsc_0_30 || p.somsc)) },
        { label: "Classic 0–20 cm", color: "#10b981", width: 2.0, data: sim.series.map((p) => cScale(p.somsc)) },
        { label: "SOM2 Slow (0–20)", color: "#f59e0b", width: 1.5, data: sim.series.map((p) => cScale(p.som2c)) }
      ];

  renderLineChart(
    "chart-som",
    "legend-som",
    somSeries,
    x,
    baseMatch
      ? [{ label: "Baseline SOMSC (A)", color: "#e6edf3", dashed: true, data: state.baselineSim.series.map((p) => cScale(depthMode === "deep" ? (p.somsc_0_30 || p.somsc) : p.somsc)) }]
      : []
  );

  renderLineChart(
    "chart-flux",
    "legend-flux",
    [
      { label: `NPP (${fu})`, color: "#10b981", width: 2.2, data: sim.series.map((p) => cScale(p.npp)) },
      { label: `Resp (${fu})`, color: "#f43f5e", width: 1.8, data: sim.series.map((p) => cScale(p.resp)) },
      { label: "AET (cm/yr)", color: "#06b6d4", width: 1.8, data: sim.series.map((p) => p.aet) }
    ],
    x,
    baseMatch
      ? [{ label: "Baseline NPP (A)", color: "#e6edf3", dashed: true, data: state.baselineSim.series.map((p) => cScale(p.npp)) }]
      : []
  );

  renderLineChart(
    "chart-bio",
    "legend-bio",
    [
      { label: `Large Wood C (${u})`, color: "#f59e0b", width: 2.0, data: sim.series.map((p) => cScale(p.rlwodc)) },
      { label: `Herbaceous C (${u})`, color: "#10b981", width: 2.0, data: sim.series.map((p) => cScale(p.aglivc + p.bglivc)) },
      { label: `Litter C (${u})`, color: "#94a3b8", width: 1.6, data: sim.series.map((p) => cScale(p.litter_c)) }
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

  const scale = (v) => (isCarbonLayer(layer) ? cScale(v) : v);
  let minV = Infinity, maxV = -Infinity;
  for (const c of grid.cells) {
    const v = scale(Number(c[layer]));
    if (v < minV) minV = v;
    if (v > maxV) maxV = v;
  }
  if (!Number.isFinite(minV) || maxV <= minV) { minV = 0; maxV = 1; }
  const suffix = isCarbonLayer(layer) ? ` ${layer === "npp" ? cFluxUnit() : cUnit()}` : "";
  $("cb-min").textContent = `${fmt(minV, layer === "biome_id" ? 0 : 1)}${suffix}`;
  $("cb-max").textContent = `${fmt(maxV, layer === "biome_id" ? 0 : 1)}${suffix}`;

  const cellW = cw / grid.cols;
  const cellH = ch / grid.rows;
  for (const c of grid.cells) {
    const v = scale(Number(c[layer]));
    const norm = (v - minV) / (maxV - minV);
    ctx.fillStyle = viridisColor(norm);
    ctx.fillRect(c.col * cellW, c.row * cellH, Math.ceil(cellW), Math.ceil(cellH));
  }
}

function inspectCell(cell) {
  state.selectedGridCell = cell;
  const u = cUnit();
  const fu = cFluxUnit();
  const dStock = state.carbonUnit === "mgha" ? 2 : 1;
  $("cell-inspector-hint").style.display = "none";
  $("cell-inspector-body").innerHTML = [
    ["Coordinates", `Row ${cell.row}, Col ${cell.col}`],
    ["Biome ID", `${cell.biome_id}`],
    ["Total SOMSC", `${fmt(cScale(cell.somsc), dStock)} ${u}`],
    ["Passive SOM3", `${fmt(cScale(cell.som3c), dStock)} ${u}`],
    ["Large Wood", `${fmt(cScale(cell.rlwodc), dStock)} ${u}`],
    ["Annual NPP", `${fmt(cScale(cell.npp), dStock)} ${fu}`],
    ["Annual AET", `${fmt(cell.aet, 1)} cm/yr`],
    ["Soil Water (L0)", `${fmt(cell.soil_water_cm, 2)} cm`],
    ["Mineral N (L0)", `${fmt(cell.minerl_n, 3)} g N/m²`]
  ].map(([k, v]) => `<div class="insp-row"><span>${k}</span><span>${v}</span></div>`).join("");
}

function renderGridSummary(data) {
  $("gkpi-cells").textContent = `${data.ncells} × ${data.nyears}`;
  $("gkpi-source").textContent = data.source_label;
  $("gkpi-time").textContent = `${fmt(data.elapsed_secs, 3)} s`;
  $("gkpi-threads").textContent = `${data.threads_used} threads`;
  $("gkpi-rate").textContent = `${fmt(data.cell_years_per_sec, 0)}`;
  $("gkpi-mo-rate").textContent = `${fmt(data.cell_months_per_sec / 1e6, 2)}M cell-mo/s`;
  $("gkpi-hpc").textContent = `${fmt(data.hpc_256t_est_cell_yr_s / 1e6, 2)}M`;
  $("gkpi-som").textContent = `${fmt(cScale(data.mean_somsc), cDigits(false))} ${cUnit()}`;
  $("gkpi-npp").textContent = `NPP: ${fmt(cScale(data.mean_npp), cDigits(true))} ${cFluxUnit()}`;
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

    renderGridSummary(data);

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

// ==================== LULC & RASTER ENGINE (TAB 3) ====================
function populateLulcSchemeSelect(keep = false) {
  const sel = $("lulc-scheme");
  if (!sel || !state.lulcSchemes.length) return;
  const prev = keep ? sel.value : null;
  sel.innerHTML = "";
  for (const s of state.lulcSchemes) {
    const opt = document.createElement("option");
    opt.value = s.id;
    opt.textContent = state.lang === "pt" ? s.name_pt : s.name_en;
    sel.appendChild(opt);
  }
  if (prev && state.lulcSchemes.some((s) => s.id === prev)) sel.value = prev;
  renderLulcLegend();
}

function activeLulcClasses() {
  const sid = $("lulc-scheme").value;
  const scheme = state.lulcSchemes.find((s) => s.id === sid) || state.lulcSchemes[0];
  return scheme ? scheme.classes : [];
}

function renderLulcLegend() {
  const classes = activeLulcClasses();
  $("lulc-legend-list").innerHTML = classes.map((c) => `
    <div class="lulc-leg-row">
      <span class="lulc-leg-left">
        <span class="lulc-swatch" style="background:${c.color}"></span>
        <span>${state.lang === "pt" ? c.label_pt : c.label_en}</span>
      </span>
      <span class="lulc-code">${c.code} · ${c.preset_id === "masked" ? "mask" : c.preset_id.split("_")[0]}</span>
    </div>
  `).join("");
}

function divergingColor(val, maxAbs) {
  const norm = Math.max(-1, Math.min(1, val / Math.max(maxAbs, 1e-6)));
  if (norm < 0) {
    const u = -norm;
    const r = Math.round(30 + u * 214);
    const g = Math.round(41 + u * 22);
    const b = Math.round(59 + u * 35);
    return `rgb(${r},${g},${b})`;
  } else {
    const u = norm;
    const r = Math.round(30 - u * 14);
    const g = Math.round(41 + u * 144);
    const b = Math.round(59 + u * 70);
    return `rgb(${r},${g},${b})`;
  }
}

function drawLulcMap() {
  const res = state.lastLulc;
  if (!res) return;
  const layer = $("lulc-layer").value;
  const canvas = $("lulc-canvas");
  const ctx = canvas.getContext("2d");
  const cw = canvas.width;
  const ch = canvas.height;
  ctx.fillStyle = "#0b192c"; // Water/river background
  ctx.fillRect(0, 0, cw, ch);

  const cellW = cw / res.cols;
  const cellH = ch / res.rows;
  const isCategorical = layer === "lulc_initial" || layer === "lulc_final";
  const isDiverging = layer === "delta_somsc" || layer === "delta_somsc_0_30";

  const colorMap = new Map(res.legend.map((c) => [c.code, c.color]));
  const cbWrap = $("lulc-cb-wrap");
  const cbBar = $("lulc-colorbar");

  if (isCategorical) {
    cbWrap.style.display = "none";
    for (const p of res.pixels) {
      ctx.fillStyle = colorMap.get(p[layer]) || "#94a3b8";
      ctx.fillRect(p.col * cellW, p.row * cellH, Math.ceil(cellW), Math.ceil(cellH));
    }
    return;
  }

  cbWrap.style.display = "flex";
  if (isDiverging) {
    let maxAbs = 10;
    for (const p of res.pixels) maxAbs = Math.max(maxAbs, Math.abs(Number(p[layer])));
    cbBar.style.background = "linear-gradient(90deg, #f43f5e, #1e293b, #10b981)";
    const dDig = state.carbonUnit === "mgha" ? 1 : 0;
    $("lcb-min").textContent = `-${fmt(cScale(maxAbs), dDig)} ${cUnit()}`;
    $("lcb-max").textContent = `+${fmt(cScale(maxAbs), dDig)} ${cUnit()}`;
    for (const p of res.pixels) {
      ctx.fillStyle = divergingColor(Number(p[layer]), maxAbs);
      ctx.fillRect(p.col * cellW, p.row * cellH, Math.ceil(cellW), Math.ceil(cellH));
    }
  } else {
    const scale = (v) => (isCarbonLayer(layer) ? cScale(v) : v);
    let minV = Infinity, maxV = -Infinity;
    for (const p of res.pixels) {
      const v = scale(Number(p[layer]));
      if (v < minV) minV = v;
      if (v > maxV) maxV = v;
    }
    if (!Number.isFinite(minV) || maxV <= minV) { minV = 0; maxV = 1; }
    const suffix = isCarbonLayer(layer) ? ` ${layer === "npp" ? cFluxUnit() : cUnit()}` : "";
    cbBar.style.background = "linear-gradient(90deg, #1b1035, #3b2d7b, #1d6f8a, #10b981, #a3e635, #fde047)";
    $("lcb-min").textContent = `${fmt(minV, 1)}${suffix}`;
    $("lcb-max").textContent = `${fmt(maxV, 1)}${suffix}`;
    for (const p of res.pixels) {
      ctx.fillStyle = viridisColor((scale(Number(p[layer])) - minV) / (maxV - minV));
      ctx.fillRect(p.col * cellW, p.row * cellH, Math.ceil(cellW), Math.ceil(cellH));
    }
  }
}

function inspectLulcPixel(p) {
  state.selectedLulcPixel = p;
  const res = state.lastLulc;
  const labelOf = (code) => {
    const m = res && res.legend.find((c) => c.code === code);
    return m ? `${state.lang === "pt" ? m.label_pt : m.label_en} [${code}]` : `[${code}]`;
  };
  const u = cUnit();
  const fu = cFluxUnit();
  const dStock = state.carbonUnit === "mgha" ? 2 : 1;
  const dSign = p.delta_somsc >= 0 ? "+" : "";
  const co2e = (p.delta_somsc / 100.0) * CO2E_FACTOR;
  $("lulc-insp-hint").style.display = "none";
  $("lulc-insp-body").innerHTML = [
    ["Coordinates", `${p.lat.toFixed(2)}°, ${p.lon.toFixed(2)}°`],
    ["LULC Initial", labelOf(p.lulc_initial)],
    ["LULC Final", labelOf(p.lulc_final)],
    ["SOMSC Initial", `${fmt(cScale(p.somsc_initial), dStock)} ${u}`],
    ["SOMSC 0–20 cm (Final)", `${fmt(cScale(p.somsc_final), dStock)} ${u}`],
    ["SOMSC 0–30 cm (IPCC)", `${fmt(cScale(p.somsc_0_30 || p.somsc_final), dStock)} ${u}`],
    ["SOMSC 0–100 cm (Deep)", `${fmt(cScale(p.somsc_0_100 || p.somsc_final), dStock)} ${u}`],
    ["ΔSOMSC 0–20 cm", `${dSign}${fmt(cScale(p.delta_somsc), dStock)} ${u} (${dSign}${fmt(co2e, 2)} t CO₂e/ha)`],
    ["Live Biomass C", `${fmt(cScale(p.biomass_c), dStock)} ${u}`],
    ["Annual NPP", `${fmt(cScale(p.npp), dStock)} ${fu}`],
    ["Annual AET", `${fmt(p.aet, 1)} cm/yr`],
    ["Mineral N (L0)", `${fmt(p.minerl_n, 3)} g N/m²`]
  ].map(([k, v]) => `<div class="insp-row"><span>${k}</span><span>${v}</span></div>`).join("");
}

function renderLulcSummary(data) {
  const pctConv = data.n_active ? (data.n_converted / data.n_active) * 100 : 0;
  $("lkpi-pixels").textContent = `${data.n_active}`;
  $("lkpi-converted").textContent = `${data.n_converted} converted (${fmt(pctConv, 1)}%)`;
  $("lkpi-time").textContent = `${fmt(data.elapsed_secs, 2)} s`;
  $("lkpi-rate").textContent = `${fmt(data.pixel_years_per_sec, 0)} pixel-yr/s (${data.threads_used}T)`;
  $("lkpi-som").textContent = fmt(cScale(data.mean_somsc), cDigits(false));
  const ipcc30 = data.mean_somsc_0_30 ? fmt(cScale(data.mean_somsc_0_30), cDigits(false)) : "—";
  const deep100 = data.mean_somsc_0_100 ? fmt(cScale(data.mean_somsc_0_100), cDigits(false)) : "—";
  $("lkpi-som-unit").textContent = `${cUnit()} · 30cm: ${ipcc30} · 1m: ${deep100}`;
  const dSign = data.mean_delta_somsc >= 0 ? "+" : "";
  const dEl = $("lkpi-delta");
  dEl.textContent = `${dSign}${fmt(cScale(data.mean_delta_somsc), state.carbonUnit === "mgha" ? 2 : 1)}`;
  dEl.style.color = data.mean_delta_somsc >= 0 ? "var(--emerald)" : "var(--rose)";
  const co2e = data.mean_delta_t_c_ha * CO2E_FACTOR;
  $("lkpi-delta-ha").textContent = state.carbonUnit === "mgha"
    ? `Mg C/ha · ${dSign}${fmt(co2e, 2)} t CO₂e/ha`
    : `g C/m² · ${dSign}${fmt(data.mean_delta_t_c_ha, 2)} Mg C/ha`;
  $("lkpi-cog").textContent = data.exported_cogs.length ? `${data.exported_cogs.length} COGs` : "terrio";
  $("lkpi-cog-sub").textContent = data.exported_cogs.length
    ? data.exported_cogs[0]
    : "Set dir to write GeoTIFFs";
}

async function runLulcSimulation() {
  const btn = $("btn-run-lulc");
  btn.disabled = true;
  btn.textContent = t("btn_running");
  try {
    const mode = $("lulc-mode").value;
    const rawPaths = ($("lulc-paths").value || "")
      .split(/\r?\n|,/)
      .map((s) => s.trim())
      .filter(Boolean);
    const optStr = (id) => {
      const v = ($(id).value || "").trim();
      return v.length ? v : null;
    };
    const resVal = Number($("lulc-res").value || 0);
    const onlyClasses = ($("lulc-only-classes").value || "")
      .split(/[\s,;]+/)
      .map((s) => Number(s.trim()))
      .filter((n) => Number.isInteger(n) && n > 0);
    const hwChoice = $("lulc-threads").value || "auto";
    const maxCpus = state.system ? state.system.logical_cpus : 4;
    const threads = hwChoice === "auto"
      ? maxCpus
      : hwChoice === "laptop"
        ? Math.max(1, Math.floor(maxCpus / 2))
        : Number(hwChoice);

    const req = {
      mode,
      scheme_id: $("lulc-scheme").value || "mapbiomas_br",
      class_mappings: activeLulcClasses(),
      lulc_paths: rawPaths,
      precip_path: optStr("lulc-precip-path"),
      tmin_path: optStr("lulc-tmin-path"),
      tmax_path: optStr("lulc-tmax-path"),
      precip_unit: $("lulc-punit").value,
      temp_unit: $("lulc-tunit").value,
      sand_path: optStr("lulc-sand-path"),
      clay_path: optStr("lulc-clay-path"),
      bulkd_path: optStr("lulc-bulkd-path"),
      ph_path: optStr("lulc-ph-path"),
      rock_path: null,
      soilgrids_units: $("lulc-soilgrids").checked,
      spinup_years: Number($("lulc-spinup").value),
      transient_years: Number($("lulc-trans").value),
      co2_start_ppm: Number($("lulc-co2-start").value),
      co2_end_ppm: Number($("lulc-co2-end").value),
      temp_delta_c: Number($("lulc-dt").value),
      precip_scale: Number($("lulc-pscale").value) / 100.0,
      cropland_tillage: $("lulc-till").value,
      target_resolution_m: resVal > 0 ? resVal : null,
      only_classes: onlyClasses,
      output_unit: state.carbonUnit,
      tile_rows: 64,
      max_preview_pixels: 40000,
      threads,
      export_cog_dir: optStr("lulc-export-dir")
    };
    const resp = await fetch("/api/lulc/run", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req)
    });
    if (!resp.ok) throw new Error(await resp.text());
    const data = await resp.json();
    state.lastLulc = data;
    state.lulcPixelMap = new Map(data.pixels.map((p) => [`${p.row}:${p.col}`, p]));

    renderLulcSummary(data);

    drawLulcMap();
    const convertedSample = data.pixels.find((p) => p.converted) || data.pixels[0];
    if (convertedSample) inspectLulcPixel(convertedSample);
  } catch (err) {
    alert(`LULC raster error: ${err.message}`);
  } finally {
    btn.disabled = false;
    btn.textContent = t("btn_run_lulc");
  }
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
      if (target === "lulc" && !state.lastLulc) runLulcSimulation();
    });
  });

  $("lang-en").addEventListener("click", () => applyLang("en"));
  $("lang-pt").addEventListener("click", () => applyLang("pt"));
  $("unit-mgha").addEventListener("click", () => applyCarbonUnit("mgha"));
  $("unit-gm2").addEventListener("click", () => applyCarbonUnit("gm2"));
  syncUnitLabels();

  document.querySelectorAll('input[type="range"]').forEach((inp) => {
    inp.addEventListener("input", syncSliderLabels);
  });
  $("in-ph").addEventListener("input", updatePhLitmus);

  $("preset-select").addEventListener("change", (e) => {
    loadPresetIntoControls(e.target.value);
    runSimulation();
  });
  if ($("som-depth-mode")) {
    $("som-depth-mode").addEventListener("change", () => {
      if (state.lastSim) renderSimulation(state.lastSim);
    });
  }

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

  $("lulc-mode").addEventListener("change", () => {
    $("lulc-real-paths").style.display = $("lulc-mode").value === "geotiff" ? "block" : "none";
  });
  $("lulc-scheme").addEventListener("change", renderLulcLegend);
  $("lulc-layer").addEventListener("change", drawLulcMap);
  $("btn-run-lulc").addEventListener("click", runLulcSimulation);
  $("btn-lulc-csv").addEventListener("click", () => {
    if (!state.lastLulc) return;
    const cols = Object.keys(state.lastLulc.pixels[0]);
    const lines = [cols.join(","), ...state.lastLulc.pixels.map((r) => cols.map((c) => r[c]).join(","))];
    downloadBlob("oxisol_lulc_spatial.csv", lines.join("\n"), "text/csv");
  });
  $("lulc-canvas").addEventListener("click", (ev) => {
    if (!state.lastLulc) return;
    const rect = ev.target.getBoundingClientRect();
    const col = Math.floor(((ev.clientX - rect.left) / rect.width) * state.lastLulc.cols);
    const row = Math.floor(((ev.clientY - rect.top) / rect.height) * state.lastLulc.rows);
    const pix = state.lulcPixelMap.get(`${row}:${col}`);
    if (pix) inspectLulcPixel(pix);
  });

  // Fetch system info, presets & LULC schemes in parallel
  const [sysRes, preRes, lulcRes] = await Promise.all([
    fetch("/api/system"),
    fetch("/api/presets"),
    fetch("/api/lulc/schemes")
  ]);
  state.system = await sysRes.json();
  state.presets = await preRes.json();
  state.lulcSchemes = await lulcRes.json();
  populateLulcSchemeSelect(false);

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
