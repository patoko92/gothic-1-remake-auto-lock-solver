use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

pub const MIN_POS: i32 = 1;
pub const MAX_POS: i32 = 7;
pub const GOAL_VALUE: i32 = 4;

/// A dependency: when the "owner" layer is moved, this layer's pin moves by `dir` (+1=equal, -1=opposite, 0=none)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    pub bar: usize,
    pub dir: i32, // +1, -1, or 0
}

/// Full set of rules: for each layer, list of dependencies.
/// The self-move (itself, +1) should always be included for the layer to move itself.
pub type Rules = HashMap<usize, Vec<Dependency>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleConfig {
    pub num_bars: usize,
    pub start: Vec<i32>,
    pub rules: Rules,
}

/// Direction of a move: Left (+1) or Right (-1)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    #[serde(rename = "L")]
    Left,
    #[serde(rename = "R")]
    Right,
}

impl Direction {
    pub fn delta(self) -> i32 {
        match self {
            Direction::Left => 1,
            Direction::Right => -1,
        }
    }
}

/// A single move: layer index (0-based) and direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Move {
    pub bar: usize,
    pub direction: Direction,
}

/// Apply a move to a state. Returns None if the move is invalid (out of bounds).
pub fn apply_move(state: &[i32], rules: &Rules, mv: Move) -> Option<Vec<i32>> {
    let delta = mv.direction.delta();

    let deps = rules.get(&mv.bar)?;
    let mut new_state = state.to_vec();

    // First pass: validate all positions stay in bounds
    for dep in deps {
        let new_val = new_state[dep.bar] + delta * dep.dir;
        if new_val < MIN_POS || new_val > MAX_POS {
            return None;
        }
    }

    // Second pass: apply all changes
    for dep in deps {
        new_state[dep.bar] += delta * dep.dir;
    }

    Some(new_state)
}

/// BFS solver. Goal is always all-4s (length = num_bars).
/// Returns the sequence of moves to reach the goal, or None if impossible.
pub fn solve(config: &PuzzleConfig) -> Option<Vec<Move>> {
    let start: Vec<i32> = config.start.clone();
    let goal: Vec<i32> = vec![GOAL_VALUE; config.num_bars];

    let mut queue = VecDeque::new();
    let mut seen = HashSet::new();

    queue.push_back((start.clone(), Vec::new()));
    seen.insert(start);

    while let Some((state, path)) = queue.pop_front() {
        if state == goal {
            return Some(path);
        }

        for bar in 0..config.num_bars {
            for &dir in &[Direction::Left, Direction::Right] {
                if let Some(new_state) = apply_move(&state, &config.rules, Move { bar, direction: dir })
                {
                    if seen.insert(new_state.clone()) {
                        let mut new_path = path.clone();
                        new_path.push(Move { bar, direction: dir });
                        queue.push_back((new_state, new_path));
                    }
                }
            }
        }
    }

    None
}

/// Return the default "hard puzzle" config from old.py (6 layers)
pub fn default_hard_config() -> PuzzleConfig {
    PuzzleConfig {
        num_bars: 6,
        start: vec![5, 6, 2, 2, 1, 1],
        rules: default_hard_rules(),
    }
}

pub fn default_hard_rules() -> Rules {
    HashMap::from([
        (
            0,
            vec![Dependency { bar: 0, dir: 1 }],
        ),
        (
            1,
            vec![
                Dependency { bar: 1, dir: 1 },
                Dependency { bar: 2, dir: -1 },
                Dependency { bar: 4, dir: 1 },
            ],
        ),
        (
            2,
            vec![
                Dependency { bar: 2, dir: 1 },
                Dependency { bar: 3, dir: 1 },
            ],
        ),
        (
            3,
            vec![Dependency { bar: 3, dir: 1 }],
        ),
        (
            4,
            vec![
                Dependency { bar: 4, dir: 1 },
                Dependency { bar: 0, dir: -1 },
                Dependency { bar: 1, dir: 1 },
                Dependency { bar: 2, dir: -1 },
                Dependency { bar: 5, dir: -1 },
            ],
        ),
        (
            5,
            vec![
                Dependency { bar: 5, dir: 1 },
                Dependency { bar: 0, dir: -1 },
                Dependency { bar: 1, dir: 1 },
            ],
        ),
    ])
}

