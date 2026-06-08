mod solver;
mod playback;

use axum::{
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use solver::{Direction, Move, PuzzleConfig, Rules};
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

#[derive(Clone)]
struct AppState {
    config: Arc<Mutex<PuzzleConfig>>,
    solution: Arc<Mutex<Option<Vec<Move>>>>,
    playing: Arc<AtomicBool>,
    lan_mode: Arc<AtomicBool>,
}

#[derive(Serialize)]
struct SolveResponse {
    success: bool,
    solution: Option<Vec<String>>,
    steps: usize,
    message: String,
}

#[derive(Deserialize)]
struct UpdateConfigRequest {
    start: Option<Vec<i32>>,
    num_bars: Option<usize>,
    rules: Option<Rules>,
}

fn format_solution(moves: &[Move]) -> Vec<String> {
    moves
        .iter()
        .map(|m| {
            let dir = match m.direction {
                Direction::Left => "L",
                Direction::Right => "R",
            };
            format!("{}{}", m.bar + 1, dir) // 1-based display
        })
        .collect()
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn get_config(State(state): State<AppState>) -> Json<PuzzleConfig> {
    let config = state.config.lock().unwrap();
    Json(config.clone())
}

async fn update_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<PuzzleConfig>, StatusCode> {
    let mut config = state.config.lock().unwrap();

    if let Some(num_bars) = req.num_bars {
        if num_bars < 1 || num_bars > 10 {
            return Err(StatusCode::BAD_REQUEST);
        }
        config.num_bars = num_bars;
        // Resize start if needed
        config.start.resize(num_bars, 4);
    }
    if let Some(start) = req.start {
        if start.len() != config.num_bars || start.iter().any(|&x| x < 1 || x > 7) {
            return Err(StatusCode::BAD_REQUEST);
        }
        config.start = start;
    }
    if let Some(rules) = req.rules {
        // Validate all dependency bars are within bounds
        for (_, deps) in &rules {
            if deps.iter().any(|d| d.bar >= config.num_bars) {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        config.rules = rules;
    }

    // Clear solution when config changes
    *state.solution.lock().unwrap() = None;

    Ok(Json(config.clone()))
}

async fn solve_puzzle(State(state): State<AppState>) -> Json<SolveResponse> {
    let config = state.config.lock().unwrap().clone();

    let solution = solver::solve(&config);

    let response = match &solution {
        Some(moves) => SolveResponse {
            success: true,
            solution: Some(format_solution(moves)),
            steps: moves.len(),
            message: format!("Solved in {} steps! ({} layers, goal all-4s)", moves.len(), config.num_bars),
        },
        None => SolveResponse {
            success: false,
            solution: None,
            steps: 0,
            message: "No solution found!".to_string(),
        },
    };

    *state.solution.lock().unwrap() = solution;

    Json(response)
}

/// Apply a single move to current state and return new state
#[derive(Deserialize)]
struct ApplyMoveRequest {
    state: Vec<i32>,
    bar: usize,
    direction: String,
}

#[derive(Serialize)]
struct ApplyMoveResponse {
    success: bool,
    new_state: Option<Vec<i32>>,
    error: Option<String>,
}

async fn apply_single_move(
    State(state): State<AppState>,
    Json(req): Json<ApplyMoveRequest>,
) -> Json<ApplyMoveResponse> {
    let config = state.config.lock().unwrap();
    let dir = match req.direction.as_str() {
        "L" => Direction::Left,
        "R" => Direction::Right,
        _ => {
            return Json(ApplyMoveResponse {
                success: false,
                new_state: None,
                error: Some("Invalid direction".to_string()),
            })
        }
    };

    match solver::apply_move(&req.state, &config.rules, Move { bar: req.bar, direction: dir }) {
        Some(new_state) => Json(ApplyMoveResponse {
            success: true,
            new_state: Some(new_state),
            error: None,
        }),
        None => Json(ApplyMoveResponse {
            success: false,
            new_state: None,
            error: Some("Move out of bounds".to_string()),
        }),
    }
}

#[derive(Serialize)]
struct PlaybackStatusResponse {
    playing: bool,
}

async fn playback_status(State(state): State<AppState>) -> Json<PlaybackStatusResponse> {
    Json(PlaybackStatusResponse {
        playing: state.playing.load(Ordering::SeqCst),
    })
}

async fn play_solution(State(state): State<AppState>) -> Json<serde_json::Value> {
    if state.playing.swap(true, Ordering::SeqCst) {
        return Json(serde_json::json!({"status": "already_playing"}));
    }

    let solution = state.solution.lock().unwrap().clone();
    let playing = state.playing.clone();

    if let Some(moves) = solution {
        let steps = moves.len();
        tokio::task::spawn_blocking(move || {
            playback::play_moves(&moves, &playing);
            playing.store(false, Ordering::SeqCst);
        });
        Json(serde_json::json!({"status": "playing", "steps": steps}))
    } else {
        state.playing.store(false, Ordering::SeqCst);
        Json(serde_json::json!({"status": "no_solution"}))
    }
}

async fn stop_playback(State(state): State<AppState>) -> Json<serde_json::Value> {
    state.playing.store(false, Ordering::SeqCst);
    Json(serde_json::json!({"status": "stopped"}))
}

async fn lan_ip() -> Json<serde_json::Value> {
    let ip = std::process::Command::new("sh")
        .arg("-c")
        .arg("ip -4 -br addr show 2>/dev/null | grep -vE '^lo|tailscale|docker|veth|br-|virbr' | awk '{print $3}' | cut -d/ -f1 | head -1")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    Json(serde_json::json!({"ip": ip}))
}

/// Middleware: when lan_mode is false, block non-localhost requests.
async fn enforce_local_only(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let is_lan = state.lan_mode.load(Ordering::SeqCst);
    if !is_lan && !addr.ip().is_loopback() {
        return (StatusCode::FORBIDDEN, "Access restricted to localhost").into_response();
    }
    next.run(req).await
}

async fn get_lan_mode(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"lan_mode": state.lan_mode.load(Ordering::SeqCst)}))
}

#[derive(Deserialize)]
struct SetLanModeRequest {
    lan_mode: bool,
}

async fn set_lan_mode(
    State(state): State<AppState>,
    Json(req): Json<SetLanModeRequest>,
) -> Json<serde_json::Value> {
    state.lan_mode.store(req.lan_mode, Ordering::SeqCst);
    Json(serde_json::json!({"lan_mode": req.lan_mode}))
}

#[tokio::main]
async fn main() {
    let config = solver::default_hard_config();

    let state = AppState {
        config: Arc::new(Mutex::new(config)),
        solution: Arc::new(Mutex::new(None)),
        playing: Arc::new(AtomicBool::new(false)),
        lan_mode: Arc::new(AtomicBool::new(true)),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/solve", post(solve_puzzle))
        .route("/api/apply", post(apply_single_move))
        .route("/api/playback/play", post(play_solution))
        .route("/api/playback/stop", post(stop_playback))
        .route("/api/playback/status", get(playback_status))
        .route("/api/lan-ip", get(lan_ip))
        .route("/api/lan-mode", get(get_lan_mode).post(set_lan_mode))
        .layer(middleware::from_fn_with_state(state.clone(), enforce_local_only))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("LockSolver running at http://localhost:3000");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
