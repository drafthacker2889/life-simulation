# Life Simulation

A neural life simulation built in Rust and compiled to WebAssembly. Hundreds of AI agents with recurrent neural brains live, learn, mate, build shelters, and evolve across a dynamic ecosystem with biomes, seasons, predators, and social systems — all running in real-time in your browser.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?style=flat&logo=webassembly&logoColor=white)
![Canvas 2D](https://img.shields.io/badge/Canvas_2D-E34F26?style=flat&logo=html5&logoColor=white)

---

## Features

- **600 neural agents** with GRU-based recurrent brains that learn in real-time
- **Gender-based reproduction** — only opposite-gender pairs can mate; no random spawning
- **Living predators** with their own lifecycle (aging, energy, mating, death)
- **4 biomes** (Forest, Desert, Swamp, Plains) affecting movement speed and food growth
- **4 seasons** cycling through Spring → Summer → Autumn → Winter
- **Shelter building** near rocks for protection from predators and winter
- **Social systems** — cooperation, betrayal, deception, reputation tracking
- **Curiosity-driven exploration** via intrinsic motivation and world-model prediction error
- **Neurochemistry** — dopamine, cortisol, oxytocin, serotonin modulate behavior
- **Episodic memory** — agents remember significant life events
- **Species divergence** — behavioral clustering identifies emerging species
- **Full save/load** — export and import entire simulation state as JSON
- **Interactive inspector** — click any agent to view its brain activity, memories, and social stats

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- Python 3 (or any local HTTP server)
- A modern browser with WebAssembly support

### Install the WASM target

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### Build

```bash
wasm-pack build --target web --release
```

This generates the `pkg/` directory with `life_simulation.js` and `life_simulation_bg.wasm`.

### Run

```bash
python -m http.server 8080
```

Open [http://localhost:8080](http://localhost:8080) in your browser.

---

## Architecture

### Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Simulation Engine | Rust | Agent physics, neural computation, game logic |
| WebAssembly | wasm-bindgen | Compile Rust to WASM for browser execution |
| Rendering | Canvas 2D (web-sys) | Real-time visualization |
| Serialization | serde + serde-wasm-bindgen | Save/load simulation state as JSON |
| Frontend | HTML5 + Vanilla JS | UI panels, controls, dashboard charts |

### Project Structure

```
life_simulation/
├── Cargo.toml           # Dependencies and build config
├── index.html           # Frontend: canvas, controls, inspector, dashboard
├── pkg/                 # Generated WASM output (after build)
└── src/
    ├── lib.rs           # Main simulation engine (~1500 lines)
    ├── brain.rs         # Neural architecture & cognitive subsystems (~770 lines)
    ├── constants.rs     # All tunable parameters (~75 lines)
    └── spatial_grid.rs  # Grid-based spatial indexing (~65 lines)
```

---

## Neural Architecture

Each agent has a recurrent neural network with the following pipeline:

```
22 inputs → Encoder (12) → GRU Memory (16) → Core (24) → Policy (7 actions)
                                                        → Value (1 estimate)
                                                        → World Model (12 predictions)
```

### Inputs (22 sensors)

| # | Input | Description |
|---|-------|-------------|
| 0 | food_dist | Distance to nearest food (normalized) |
| 1-2 | food_angle | sin/cos of angle to nearest food |
| 3 | pred_dist | Distance to nearest predator |
| 4-5 | pred_angle | sin/cos of angle to nearest predator |
| 6 | energy | Current energy (normalized) |
| 7 | friend_dist | Distance to nearest tribe mate |
| 8-10 | wall sensors | Left, center, right whisker distances |
| 11 | hearing | Social signal volume from nearby agents |
| 12 | env_signal | Biome and season information |
| 13-14 | long-term memory | Food and danger quadrant traces |
| 15 | avg_signal | Average signal type from neighbors |
| 16 | avg_reputation | Trust level of nearby agents |
| 17 | season | Current season (0-3) |
| 18-21 | neurochemistry | Dopamine, cortisol, oxytocin, serotonin |

### Outputs (7 actions)

| # | Action | Range | Description |
|---|--------|-------|-------------|
| 0 | Turn | [-1, 1] | Steering force |
| 1 | Speed | [-1, 1] | Movement velocity |
| 2 | Voice | [-1, 1] | Social signal volume |
| 3 | Build | [-1, 1] | Shelter construction intent |
| 4 | Flee | [-1, 1] | Panic / escape intent |
| 5 | Mate | [-1, 1] | Reproduction desire |
| 6 | Explore | [-1, 1] | Curiosity exploration |

---

## Cognitive Subsystems

### Neurochemistry

Four neuromodulators influence behavior and learning:

- **Dopamine** — Reward signal; modulates action noise for exploration vs exploitation
- **Cortisol** — Stress/threat response; increases with predator proximity
- **Oxytocin** — Social bonding; increases near tribe mates
- **Serotonin** — Wellbeing; inversely correlated with cortisol

### Goal Arbitration (5 Drives)

Each agent has genetically-encoded base drive weights, dynamically modulated:

| Drive | Urgency Formula |
|-------|----------------|
| Hunger | `1 - energy/200` |
| Fear | `threat_level` |
| Reproduction | `(energy/200) * (1 - cortisol)` |
| Social | `oxytocin` |
| Curiosity | `dopamine * (1 - cortisol)` |

### Memory Systems

- **GRU short-term memory** — 16-dimensional hidden state for temporal context
- **Long-term memory** — 4 quadrant traces for food and danger (exponential smoothing)
- **Episodic memory** — Top 8 most significant life events (food, poison, predator, social encounters)

### Curiosity Module

- **World model** predicts next encoded observation from current state + actions
- **Prediction error** drives intrinsic reward (novelty-seeking)
- **Visitation grid** provides exploration bonus for unvisited areas

### Social System

- **Reputation** (0-1): Earned through cooperation, lost through betrayal
- **Deception tendency**: Genetically encoded; high deception agents may steal instead of share
- **Signal types**: Neutral, "Food here", "Danger!", "Mate call"
- **Food sharing**: High-energy agents share with low-energy tribe mates

---

## Simulation Systems

### Gender & Reproduction

- Agents are randomly assigned male or female at birth
- Only opposite-gender pairs within the same tribe can mate
- Both parents must exceed the energy threshold and express mating intent
- Offspring inherit crossover of parent neural weights + random mutation
- Dead agents are not respawned — population grows only through mating

### Predator Lifecycle

Predators are living entities with their own lifecycle:

- **Age & die** — max lifespan of 20,000 ticks; slow down when old
- **Hunt & eat** — gain energy from kills; lose energy each tick
- **Gender & mating** — opposite-gender predators reproduce when energy is high
- **Population cap** — maximum 20 predators; minimum 1 guaranteed

### Biomes

| Biome | Speed | Food Growth | Special |
|-------|-------|-------------|---------|
| Plains | 1.0x | 1.0x | Default terrain |
| Forest | 0.7x | 2.0x | Rich but slow |
| Desert | 1.3x | 0.15x | Fast but barren |
| Swamp | 0.4x | 0.6x | Poison damage without shelter |

### Seasons (1200 ticks each)

| Season | Food Multiplier | Special |
|--------|----------------|---------|
| Spring | 1.0x | Normal conditions |
| Summer | 1.5x | Abundant food |
| Autumn | 1.0x | Normal conditions |
| Winter | 0.3x | 1.5x energy drain outside shelters |

### Food Growth

Food does not spawn randomly. Instead:

- Existing food has a small chance to spread within 80 units each tick
- Growth rate depends on biome (Forest 2x, Desert 0.15x) and season
- Food cannot grow on rocks
- Capped at 400 total food items

### Shelters

- Agents can build shelters near rocks (costs 25 energy)
- Shelters protect from predators and reduce energy drain
- Shelters decay over 3000 ticks
- Maximum 30 shelters on the map

### Combat

- Agents with energy > 150 become warriors (white glow)
- Warriors can kill predators at the cost of 50 energy
- Predators that kill agents gain 80 energy

---

## UI Controls

### Stats Panel (Left)

- Season, tick counter, curriculum stage
- Average energy, births, deaths, max generation
- Alive agents, predator count, male/female breakdown
- Species count, shelter count
- Tribe population bars (4 tribes)

### Sliders

- **Simulation speed** (1-20x)
- **Mutation rate** (0.01-1.0)
- **Food count** (10-300)
- **Predator speed** (0-5.0)
- **Reproduction threshold** (10-150)

### Inspector (Click any agent)

- Gender, age, generation, lineage
- Neurochemistry bars (4 neuromodulators)
- Active drive highlight
- Long-term memory quadrant grid
- Episodic memory event list
- Social stats (reputation, cooperations, betrayals, deception)
- Brain neuron visualization (encoded → memory → hidden → outputs)
- Curiosity metrics (prediction error, novelty, intrinsic reward)
- Behavior vector (speed, turn, food efficiency, social score)

### Dashboard (Mini Charts)

- Average energy over time
- Population over time
- Cumulative cooperation events
- Tribe diversity (entropy)

### Save / Load

- **Save State** — exports the full simulation as a JSON file
- **Load State** — imports a previously saved JSON to restore
- **Benchmark** — runs 1000 ticks and reports performance metrics

---

## Key Constants

| Parameter | Value | Description |
|-----------|-------|-------------|
| `AGENT_COUNT` | 600 | Initial agent population |
| `FOOD_COUNT` | 120 | Initial food items |
| `FOOD_MAX` | 400 | Maximum food on map |
| `PREDATOR_COUNT` | 5 | Initial predators |
| `PREDATOR_MAX_COUNT` | 20 | Max predators alive |
| `PREDATOR_MAX_AGE` | 20,000 | Predator lifespan in ticks |
| `ENERGY_CAP` | 200 | Max agent energy |
| `STARTING_ENERGY` | 100 | Energy at birth |
| `MATING_ENERGY_COST` | 25 | Energy cost per parent to mate |
| `MATING_COOLDOWN` | 300 | Ticks between matings |
| `SHELTER_MAX` | 30 | Max shelters on map |
| `SEASON_LENGTH` | 1200 | Ticks per season |
| `BIOME_CELL_SIZE` | 200 | Biome grid cell size (px) |
| `BASE_MUTATION_RATE` | 0.1 | Default mutation rate |

---

## Curriculum System

The simulation uses progressive difficulty staging:

| Stage | Ticks | Changes |
|-------|-------|---------|
| Easy | 0 – 5,000 | Poison damage at 0.2x |
| Medium | 5,000 – 15,000 | Poison damage at 0.6x |
| Hard | 15,000+ | Full poison damage, full challenge |

---

## License

This project is provided as-is for educational and research purposes.
