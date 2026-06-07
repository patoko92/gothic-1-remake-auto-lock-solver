mod solver;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use solver::{Direction, Move, PuzzleConfig, Rules};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    config: Arc<Mutex<PuzzleConfig>>,
    solution: Arc<Mutex<Option<Vec<Move>>>>,
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
            message: format!("Solved in {} steps! ({} bars, goal all-4s)", moves.len(), config.num_bars),
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

#[tokio::main]
async fn main() {
    let config = solver::default_hard_config();

    let state = AppState {
        config: Arc::new(Mutex::new(config)),
        solution: Arc::new(Mutex::new(None)),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/solve", post(solve_puzzle))
        .route("/api/apply", post(apply_single_move))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("LockSolver running at http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
