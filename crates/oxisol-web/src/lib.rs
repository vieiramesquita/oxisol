//! # `oxisol-web` — Oxisol Studio & REST JSON API
//!
//! Single-binary embedded web studio powered by [`axum`] and [`tokio`], designed
//! to run identically on a local workstation or a remote high-core-count
//! modern high-performance server over SSH port-forwarding with **zero
//! external Node.js/CDN runtime dependencies**.
//!
//! ## Endpoints
//! - `GET /` — Embedded bilingual (**EN / PT-BR**) **Oxisol Studio** SPA
//! - `GET /api/health` — Liveness probe
//! - `GET /api/system` — Host CPU threads, `CellState` L1d footprint, and attached grid metadata
//! - `GET /api/presets` — 6 scientifically calibrated ecosystem presets
//! - `POST /api/simulate` — Interactive single-cell scenario run with mass-balance audit
//! - `POST /api/grid/run` — Lock-free `rayon` multi-core spatial grid run + 2D map payload
//! - `GET /api/lulc/schemes` — Built-in LULC reclassification schemes (MapBiomas Brasil, ESA WorldCover)
//! - `POST /api/lulc/run` — Spatial LULC + modeled climate GeoTIFF/COG engine powered by `terrio`

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod engine;
pub mod lulc;
pub mod models;
pub mod presets;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use oxisol_core::CellState;

use crate::engine::{execute_grid_run, execute_single_cell};
use crate::lulc::{builtin_lulc_schemes, execute_lulc_run, LulcRunRequest};
use crate::models::{GridRunRequest, HealthResponse, SimulateRequest, SystemInfoResponse};
use crate::presets::builtin_presets;

const INDEX_HTML: &str = include_str!("../ui/index.html");
const STYLE_CSS: &str = include_str!("../ui/style.css");
const APP_JS: &str = include_str!("../ui/app.js");

/// Configuration for launching the embedded Oxisol Studio HTTP server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address (e.g. `"127.0.0.1"` or `"0.0.0.0"` on an HPC head node).
    pub host: String,
    /// TCP port (default `8080`).
    pub port: u16,
    /// Optional VEMAP / spatial dataset directory exposed to the Spatial Grid Explorer.
    pub data_dir: Option<PathBuf>,
    /// Optional rock fraction `.v2` raster path.
    pub rock: Option<PathBuf>,
    /// Optional mineral depth `.v2` raster path.
    pub zminrl: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            data_dir: None,
            rock: None,
            zminrl: None,
        }
    }
}

/// Shared read-only server state injected into Axum route handlers.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Optional attached spatial dataset directory.
    pub data_dir: Option<PathBuf>,
    /// Optional rock raster path.
    pub rock: Option<PathBuf>,
    /// Optional mineral soil depth raster path.
    pub zminrl: Option<PathBuf>,
}

/// Construct the complete Axum [`Router`] for Oxisol Studio and its REST API.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/assets/style.css", get(serve_css))
        .route("/assets/app.js", get(serve_js))
        .route("/api/health", get(api_health))
        .route("/api/system", get(api_system))
        .route("/api/presets", get(api_presets))
        .route("/api/simulate", post(api_simulate))
        .route("/api/grid/run", post(api_grid_run))
        .route("/api/lulc/schemes", get(api_lulc_schemes))
        .route("/api/lulc/run", post(api_lulc_run))
        .with_state(state)
}

async fn serve_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn serve_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        STYLE_CSS,
    )
}

async fn serve_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        APP_JS,
    )
}

async fn api_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "Oxisol".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn api_system(State(st): State<Arc<AppState>>) -> Json<SystemInfoResponse> {
    let logical_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let attached_grid_label = st.data_dir.as_ref().map(|p| p.display().to_string());
    Json(SystemInfoResponse {
        name: "Oxisol Studio".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        logical_cpus,
        cell_state_f64_bytes: std::mem::size_of::<CellState<f64>>(),
        cell_state_f32_bytes: std::mem::size_of::<CellState<f32>>(),
        has_attached_grid: st.data_dir.is_some(),
        attached_grid_label,
    })
}

async fn api_presets() -> impl IntoResponse {
    Json(builtin_presets())
}

async fn api_simulate(Json(req): Json<SimulateRequest>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || execute_single_cell(&req)).await;
    match res {
        Ok(payload) => (StatusCode::OK, Json(payload)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Simulation task failed: {e}") })),
        )
            .into_response(),
    }
}

async fn api_grid_run(
    State(st): State<Arc<AppState>>,
    Json(req): Json<GridRunRequest>,
) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        execute_grid_run(
            &req,
            st.data_dir.as_deref(),
            st.rock.as_deref(),
            st.zminrl.as_deref(),
        )
    })
    .await;

    match res {
        Ok(Ok(payload)) => (StatusCode::OK, Json(payload)).into_response(),
        Ok(Err(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Grid task failed: {e}") })),
        )
            .into_response(),
    }
}

async fn api_lulc_schemes() -> impl IntoResponse {
    Json(builtin_lulc_schemes())
}

async fn api_lulc_run(Json(req): Json<LulcRunRequest>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || execute_lulc_run(&req)).await;
    match res {
        Ok(Ok(payload)) => (StatusCode::OK, Json(payload)).into_response(),
        Ok(Err(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("LULC task failed: {e}") })),
        )
            .into_response(),
    }
}

/// Bind the HTTP server and serve Oxisol Studio until `Ctrl+C` is received.
pub async fn serve(config: ServerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let state = Arc::new(AppState {
        data_dir: config.data_dir,
        rock: config.rock,
        zminrl: config.zminrl,
    });
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!(
        "Oxisol Studio v{} listening at http://{}",
        env!("CARGO_PKG_VERSION"),
        addr
    );
    println!("Press Ctrl+C to stop.");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_app() -> Router {
        build_router(Arc::new(AppState {
            data_dir: None,
            rock: None,
            zminrl: None,
        }))
    }

    #[tokio::test]
    async fn health_system_and_presets_endpoints_succeed() {
        let app = test_app();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/system")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/presets")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let presets: Vec<crate::models::PresetSummary> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(presets.len(), 8);
        assert_eq!(presets[0].id, "tropical_pasture_urochloa");
    }

    #[tokio::test]
    async fn simulate_endpoint_conserves_mass_and_returns_series() {
        let app = test_app();
        let mut req_body = builtin_presets()
            .into_iter()
            .find(|p| p.id == "cerrado_savanna")
            .unwrap()
            .defaults;
        req_body.years = 25;

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/simulate")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap();
        let sim: crate::models::SimulateResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(sim.series.len(), 25);
        assert!(sim.final_pools.somsc > 0.0 && sim.final_pools.somsc.is_finite());
        assert!(
            sim.final_pools.max_mass_balance_err < 1.0e-9,
            "mass balance residual too high: {}",
            sim.final_pools.max_mass_balance_err
        );
    }

    #[tokio::test]
    async fn grid_endpoint_executes_parallel_synthetic_continent() {
        let app = test_app();
        let req_body = GridRunRequest {
            use_attached_grid: false,
            rows: 10,
            cols: 16,
            years: 20,
            threads: Some(2),
            chunk_size: 8,
            profile: "clean".into(),
            precision: "f64".into(),
        };

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/grid/run")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap();
        let out: crate::models::GridRunResponse = serde_json::from_slice(&bytes).unwrap();
        assert!(out.ncells > 50);
        assert!(out.mean_somsc > 100.0);
        assert!(out.cell_years_per_sec > 0.0);
    }

    #[tokio::test]
    async fn lulc_endpoint_runs_amazon_cerrado_demo_and_exports_cogs() {
        let app = test_app();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/lulc/schemes")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let schemes_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let schemes: Vec<crate::lulc::LulcScheme> = serde_json::from_slice(&schemes_bytes).unwrap();
        let mb = schemes.iter().find(|s| s.id == "mapbiomas_br").unwrap();
        assert_eq!(mb.name_en, "MapBiomas Brasil (Collection 11)");
        assert_eq!(mb.classes.len(), 41);
        for required_code in [1, 3, 4, 6, 7, 15, 20, 39, 62, 75, 77, 84, 91, 33] {
            assert!(
                mb.classes.iter().any(|c| c.code == required_code),
                "missing MapBiomas Collection 11 code {required_code}"
            );
        }

        let tmp_dir = std::env::temp_dir().join("oxisol_lulc_cog_test");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        let req_body = LulcRunRequest {
            mode: "demo".into(),
            scheme_id: "mapbiomas_br".into(),
            class_mappings: vec![],
            lulc_paths: vec![],
            precip_path: None,
            tmin_path: None,
            tmax_path: None,
            precip_unit: crate::lulc::PrecipUnit::Mm,
            temp_unit: crate::lulc::TempUnit::Celsius,
            sand_path: None,
            clay_path: None,
            bulkd_path: None,
            ph_path: None,
            rock_path: None,
            soilgrids_units: true,
            spinup_years: 15,
            transient_years: 12,
            co2_start_ppm: 354.0,
            co2_end_ppm: 422.0,
            temp_delta_c: 1.5,
            precip_scale: 0.92,
            cropland_tillage: crate::models::TillageMode::MoldboardPlow,
            threads: Some(2),
            export_cog_dir: Some(tmp_dir.display().to_string()),
        };

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/lulc/run")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 16 * 1024 * 1024)
            .await
            .unwrap();
        let out: crate::lulc::LulcRunResponse = serde_json::from_slice(&bytes).unwrap();
        assert!(out.n_active > 1000);
        assert!(out.n_converted > 100);
        assert_eq!(out.exported_cogs.len(), 6);
        // Verify exported COG reads back cleanly via terrio
        let som_cog = oxisol_io::raster_geo::GeoRaster::open(&out.exported_cogs[0]).unwrap();
        assert_eq!(som_cog.rows(), 36);
        assert_eq!(som_cog.cols(), 56);
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
