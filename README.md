# 🔒 Gothic 1 Remake — Auto Lock Solver

A BFS-based puzzle solver for the lockpicking mini-game in **Gothic 1 Remake**, written in Rust with a beautiful web UI. It finds the optimal sequence of moves to solve any lock configuration, then can **play the solution automatically** via simulated keyboard input (WASD).

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.85%2B-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
  <img src="https://img.shields.io/badge/status-stable-brightgreen" alt="Status">
</p>

---

## 📖 How the Lock Puzzle Works

In Gothic 1 Remake, locks consist of **1–10 layers**, each containing a **pin** at a position from **1 to 7**. The goal is always to get **all pins to position 4**. You move the selected pin by pressing:

| Key | Action |
|-----|--------|
| `W` | Select next layer |
| `S` | Select previous layer |
| `A` | Move selected pin **left** (+1) |
| `D` | Move selected pin **right** (−1) |

The twist: **moving one layer's pin can also move other pins on other layers**. These dependencies (rules) are different for each lock in the game. For example, moving layer 1 might also move layer 2's pin in the opposite direction and layer 4's pin in the same direction.

This solver takes the pin positions (one per layer) and dependency rules, computes the shortest solution via **BFS** (breadth-first search), and can optionally replay it as real keyboard input into the game.

> 📋 See the **[How to Use](#-how-to-use--step-by-step)** section above for a detailed walkthrough.

---

## 🎮 How to Use — Step by Step

When you encounter a lock in the game, follow these steps:

### 1️⃣ Count the Layers

Look at the lock in the game. How many layers does it have? Most locks have 4–6, but some can have more. Set the **Layers** number in the top-left of the UI to match.

### 2️⃣ Set the Initial Pin Positions

Each layer has a pin that starts at a certain position (1–7). Look at the lock and set each pin's starting value in the **Layer State** grid. You can type the number directly or use the arrow buttons.

### 3️⃣ Discover the Dependencies (Rules)

This is the most important step. For **each layer**, you need to figure out which other pins move when you move it:

1. In the game, move a pin **left** or **right** by one step
2. Watch which other pins also moved
3. In the UI, click the layer's row in the **Layer Rules** panel to open its editor
4. For each affected pin, click the badge to cycle through:
   - `self` — the pin itself moves (always required)
   - `same dir` — this other pin moves in the **same** direction
   - `opposite` — this other pin moves in the **opposite** direction
   - `off` — no effect (click again to remove)

> 💡 **Tip:** Start with the layer that has the fewest dependencies — it's easier to isolate what's moving.

Repeat for every layer until all dependencies are mapped out.

### 4️⃣ Solve

Click **🔍 Solve Puzzle**. The BFS solver computes the shortest sequence of moves to get all pins to position 4. You'll see the full move list in the solution box.

### 5️⃣ Execute the Solution

You have two options:

**Manual (read and follow):** Read each move from the solution box and perform it in-game. The format is `{layer}{direction}` — for example, `1R` means "move layer 1 right", `2L` means "move layer 2 left". Layer numbers are 1-based (matching the in-game display).

**Automatic (WASD Playback):**
1. Make sure the lock in the game is at the **starting position** you configured
2. Click **▶ Play** — a 10-second countdown starts
3. Switch to the game window immediately and hover the lock
4. Watch as the program types the keystrokes for you:
   - `W`/`S` to select the correct layer
   - `A`/`D` to move the pin left or right
5. You can click **⏹ Stop** at any time to abort

> ⚠️ **Important:** The playback assumes you are already at layer 1 when it starts. Make sure the lock UI in-game has the first layer selected, and the starting positions match what you entered in the solver.

---

## ⚡ Features

- 🧠 **BFS Solver** — finds the shortest sequence of moves to reach all-4s
- 🎮 **WASD Playback** — auto-plays the solution with real keyboard simulation
- 🌐 **Web UI** — beautiful Gothic-themed interface, works on desktop & mobile
- ⚙️ **Configurable Layers** — support for 1–10 layers, adjustable on the fly
- 🔗 **Rule Editor** — click-to-toggle dependencies between layers (self/same/opposite/none)
- 🏰 **Preset: Old Camp Tower Lock** — the notoriously hard 6-pin puzzle from the game
- 📡 **LAN Access** — bind to `0.0.0.0`, use from any device on your network
- 🎨 **Gothic Fantasy Theme** — MedievalSharp font, gold accents, dark palette

---

## 🚀 Quick Start

### Prerequisites
- [Rust](https://rustup.rs) 1.85+
- Linux with X11 (for keyboard playback via `enigo`)

### Run

```bash
git clone https://github.com/patoko92/gothic-1-remake-auto-lock-solver.git
cd gothic-1-remake-auto-lock-solver
cargo run --release
```

Then open **http://localhost:3000** in your browser.

---

## 🖥️ Web UI

The UI has two columns:

### Left Column
- **Layer State** — shows each layer's pin position (editable) and the number of layers
- **Solve** — runs the BFS solver and displays the move sequence
- **Playback Controls** — click **Play** to start a 10-second countdown (switch to the game!), then WASD keys are simulated automatically. Click **Stop** to cancel.

### Right Column
- **Layer Rules** — each layer row shows its dependencies. Click a row to open the editor.
- **Rule Editor** — clickable badges that cycle: `self` → `same dir` → `opposite` → `off`
- **Preset buttons** — load predefined lock configurations

---

## 🔌 REST API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Web UI |
| `GET` | `/api/config` | Get current puzzle config |
| `POST` | `/api/config` | Update puzzle config |
| `POST` | `/api/solve` | Run the solver |
| `POST` | `/api/playback/play` | Start WASD playback |
| `POST` | `/api/playback/stop` | Stop playback |
| `GET` | `/api/playback/status` | Check if playback is running |
| `GET` | `/api/lan-ip` | Get the LAN IP address |

### Example: Solve via API

```bash
curl -X POST http://localhost:3000/api/solve
```

Response:
```json
{
  "success": true,
  "solution": ["1R", "1R", "2L", "3L", "4R", "5R", "6R"],
  "steps": 7,
  "message": "Solved in 7 steps! (6 layers, goal all-4s)"
}
```

### Example: Update config

```bash
curl -X POST http://localhost:3000/api/config \
  -H 'Content-Type: application/json' \
  -d '{
    "num_bars": 6,
    "start": [5,6,2,2,1,1],
    "rules": {
      "0": [{"bar":0,"dir":1}],
      "1": [{"bar":1,"dir":1},{"bar":2,"dir":-1},{"bar":4,"dir":1}],
      "2": [{"bar":2,"dir":1},{"bar":3,"dir":1}],
      "3": [{"bar":3,"dir":1}],
      "4": [{"bar":4,"dir":1},{"bar":0,"dir":-1},{"bar":1,"dir":1},{"bar":2,"dir":-1},{"bar":5,"dir":-1}],
      "5": [{"bar":5,"dir":1},{"bar":0,"dir":-1},{"bar":1,"dir":1}]
    }
  }'
```

---

## 🏰 Old Camp Tower Lock (Hard Puzzle)

This is the infamous 6-layer lock from the Old Camp tower. Start positions: `[5, 6, 2, 2, 1, 1]`, goal: `[4, 4, 4, 4, 4, 4]`.

**Dependency rules:**

| Layer | Self | Also moves… |
|-----|------|-------------|
| 1 | ✅ | — |
| 2 | ✅ | Layer 3 opposite, Layer 5 same direction |
| 3 | ✅ | Layer 4 same direction |
| 4 | ✅ | — |
| 5 | ✅ | Layer 1 opposite, Layer 2 same, Layer 3 opposite, Layer 6 opposite |
| 6 | ✅ | Layer 1 opposite, Layer 2 same |

**Solution:** 46 steps — loaded automatically by clicking the preset button.

---

## 📁 Project Structure

```
locksolver/
├── Cargo.toml          # Dependencies (axum, enigo, serde, tokio)
├── Cargo.lock
├── .gitignore
├── README.md
├── src/
│   ├── main.rs         # Axum HTTP server + API endpoints
│   ├── solver.rs       # BFS puzzle solver logic
│   └── playback.rs     # WASD keyboard simulation via enigo
└── static/
    └── index.html      # Gothic-themed web UI
```

---

## 🛠️ Tech Stack

- **[Rust](https://www.rust-lang.org/)** — systems language
- **[Axum](https://github.com/tokio-rs/axum)** — async web framework
- **[enigo](https://github.com/enigo-rs/enigo)** — cross-platform keyboard simulation
- **[Tokio](https://tokio.rs/)** — async runtime
- **[Serde](https://serde.rs/)** — serialization

---

## 📝 Algorithm

The solver uses **Breadth-First Search (BFS)** to guarantee the shortest solution:

1. Start from the initial pin configuration
2. For each state, try moving every pin in both directions (L/R)
3. Before applying a move, validate all affected pins stay within bounds [1, 7]
4. Apply all dependency changes simultaneously
5. Track visited states in a `HashSet` to avoid cycles
6. Return the path when all pins reach position 4

**Complexity:** The state space is at most 7^N (where N = number of pins). For 6 pins, that's ~117,000 possible states — easily handled by BFS.

---

## 🤝 Contributing

Found a lock that can't be solved? Have ideas for more presets? PRs welcome!

1. Fork the repo
2. Create a feature branch
3. Submit a pull request

---

## 📄 License

MIT — feel free to use, modify, and share.
