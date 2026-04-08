use wasm_bindgen::prelude::*;
use js_sys::Math;
use serde::{Serialize, Deserialize};

// MODULES
mod constants;
mod brain;
mod spatial_grid;

use brain::{Brain, MemorySystem};
use constants::*;
use spatial_grid::SpatialGrid;

// ============================================================
//  PREDATOR — living entity with age, energy, gender
// ============================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct Predator {
    pub x: f64,
    pub y: f64,
    pub energy: f64,
    pub age: u64,
    pub female: bool,
    pub last_mate_tick: u64,
    pub angle: f64,
}

impl Predator {
    pub fn new(x: f64, y: f64) -> Self {
        Predator {
            x, y,
            energy: PREDATOR_ENERGY_START,
            age: 0,
            female: Math::random() > 0.5,
            last_mate_tick: 0,
            angle: Math::random() * 6.28,
        }
    }
}

// ============================================================
//  SAVE STATE — full simulation snapshot for save/load
// ============================================================
#[derive(Serialize, Deserialize)]
struct SaveState {
    positions: Vec<(f64, f64)>,
    angles: Vec<f64>,
    energies: Vec<f64>,
    brains: Vec<Brain>,
    colors: Vec<String>,
    voices: Vec<f64>,
    genders: Vec<bool>,
    food: Vec<(f64, f64)>,
    poison: Vec<(f64, f64)>,
    predators: Vec<Predator>,
    rocks: Vec<(f64, f64, f64)>,
    mud: Vec<(f64, f64, f64)>,
    shelters: Vec<(f64, f64, u32)>,
    biome_grid: Vec<u8>,
    biome_cols: usize,
    biome_rows: usize,
    width: f64,
    height: f64,
    tick: u64,
    season: u8,
    season_food_mult: f64,
    mutation_rate: f64,
    predator_speed: f64,
    reproduction_threshold: f64,
    total_births: u64,
    total_deaths: u64,
    max_generation: u32,
    curriculum_stage: u8,
    metrics_energy: Vec<f64>,
    metrics_population: Vec<f64>,
    metrics_cooperation: Vec<f64>,
    metrics_diversity: Vec<f64>,
}

#[wasm_bindgen]
pub struct Simulation {
    positions: Vec<(f64, f64)>,
    angles: Vec<f64>,
    energies: Vec<f64>,
    brains: Vec<Brain>,
    colors: Vec<String>,
    voices: Vec<f64>,
    genders: Vec<bool>,

    food: Vec<(f64, f64)>,
    poison: Vec<(f64, f64)>,
    predators: Vec<Predator>,

    rocks: Vec<(f64, f64, f64)>,
    mud: Vec<(f64, f64, f64)>,
    shelters: Vec<(f64, f64, u32)>,

    grid: SpatialGrid,

    log_buffer: Vec<String>,

    width: f64,
    height: f64,

    mutation_rate: f64,
    predator_speed: f64,
    reproduction_threshold: f64,
    view_x: f64, view_y: f64, zoom: f64,

    // Season system
    tick: u64,
    season: u8,
    season_food_mult: f64,

    // Stats
    total_births: u64,
    total_deaths: u64,
    max_generation: u32,

    // Biomes
    biome_grid: Vec<u8>,
    biome_cols: usize,
    biome_rows: usize,

    // Curriculum
    curriculum_stage: u8,

    // Metrics time-series
    metrics_energy: Vec<f64>,
    metrics_population: Vec<f64>,
    metrics_cooperation: Vec<f64>,
    metrics_diversity: Vec<f64>,
}

#[wasm_bindgen]
impl Simulation {
    pub fn new(width: f64, height: f64) -> Simulation {
        let mut positions = Vec::new();
        let mut angles = Vec::new();
        let mut energies = Vec::new();
        let mut brains = Vec::new();
        let mut colors = Vec::new();
        let mut voices = Vec::new();
        let mut genders = Vec::new();
        let mut food = Vec::new();
        let mut poison = Vec::new();
        let mut predators = Vec::new();
        let mut rocks = Vec::new();
        let mut mud = Vec::new();

        let color_palette = ["#ff00cc", "#ccff00", "#00ccff", "#ffcc00"];

        for _ in 0..AGENT_COUNT {
            positions.push((Math::random() * width, Math::random() * height));
            angles.push(Math::random() * 6.28);
            energies.push(STARTING_ENERGY);
            brains.push(Brain::new());
            let color_idx = (Math::random() * 4.0) as usize;
            colors.push(color_palette[color_idx].to_string());
            voices.push(0.0);
            genders.push(Math::random() > 0.5); // true = female
        }

        for _ in 0..FOOD_COUNT { food.push((Math::random() * width, Math::random() * height)); }
        for _ in 0..POISON_COUNT { poison.push((Math::random() * width, Math::random() * height)); }
        for _ in 0..PREDATOR_COUNT { predators.push(Predator::new(Math::random() * width, Math::random() * height)); }
        for _ in 0..15 { rocks.push((Math::random() * width, Math::random() * height, 20.0 + Math::random() * 30.0)); }
        for _ in 0..10 { mud.push((Math::random() * width, Math::random() * height, 40.0 + Math::random() * 60.0)); }

        let grid = SpatialGrid::new(width, height, 100.0);

        // Generate biome grid
        let biome_cols = (width / BIOME_CELL_SIZE).ceil().max(1.0) as usize;
        let biome_rows = (height / BIOME_CELL_SIZE).ceil().max(1.0) as usize;
        let mut biome_grid = vec![BIOME_PLAINS; biome_cols * biome_rows];
        // Seed clusters
        let num_seeds = 8;
        for _ in 0..num_seeds {
            let sc = (Math::random() * biome_cols as f64) as usize;
            let sr = (Math::random() * biome_rows as f64) as usize;
            let biome_type = (Math::random() * 4.0) as u8;
            let spread = 2 + (Math::random() * 3.0) as i32;
            for dr in -spread..=spread {
                for dc in -spread..=spread {
                    let r = sr as i32 + dr;
                    let c = sc as i32 + dc;
                    if r >= 0 && r < biome_rows as i32 && c >= 0 && c < biome_cols as i32 {
                        if (dr * dr + dc * dc) <= (spread * spread) {
                            biome_grid[r as usize * biome_cols + c as usize] = biome_type;
                        }
                    }
                }
            }
        }

        Simulation {
            positions, angles, energies, brains, colors, voices, genders,
            food, poison, predators, rocks, mud,
            shelters: Vec::new(),
            grid,
            log_buffer: Vec::new(),
            width, height,
            mutation_rate: BASE_MUTATION_RATE,
            predator_speed: 2.2,
            reproduction_threshold: 60.0,
            view_x: 0.0, view_y: 0.0, zoom: 1.0,
            tick: 0, season: 0, season_food_mult: 1.0,
            total_births: 0, total_deaths: 0, max_generation: 0,
            biome_grid, biome_cols, biome_rows,
            curriculum_stage: 0,
            metrics_energy: Vec::new(),
            metrics_population: Vec::new(),
            metrics_cooperation: Vec::new(),
            metrics_diversity: Vec::new(),
        }
    }

    // ---- Biome helpers ----
    fn biome_at(&self, x: f64, y: f64) -> u8 {
        let c = ((x / BIOME_CELL_SIZE) as usize).min(self.biome_cols.saturating_sub(1));
        let r = ((y / BIOME_CELL_SIZE) as usize).min(self.biome_rows.saturating_sub(1));
        let idx = r * self.biome_cols + c;
        if idx < self.biome_grid.len() { self.biome_grid[idx] } else { BIOME_PLAINS }
    }

    fn biome_speed_mult(biome: u8) -> f64 {
        match biome { BIOME_FOREST => 0.7, BIOME_DESERT => 1.3, BIOME_SWAMP => 0.4, _ => 1.0 }
    }

    fn biome_food_mult(biome: u8) -> f64 {
        match biome { BIOME_FOREST => 1.5, BIOME_DESERT => 0.4, BIOME_SWAMP => 0.6, _ => 1.0 }
    }

    // ---- Curriculum ----
    fn update_curriculum(&mut self) {
        let old = self.curriculum_stage;
        if self.tick < CURRICULUM_EASY_END {
            self.curriculum_stage = 0;
        } else if self.tick < CURRICULUM_MEDIUM_END {
            self.curriculum_stage = 1;
        } else {
            self.curriculum_stage = 2;
        }
        if self.curriculum_stage != old {
            let name = match self.curriculum_stage { 0 => "Easy", 1 => "Medium", 2 => "Hard", _ => "?" };
            self.log_buffer.push(format!("Curriculum: {} mode", name));
        }
    }

    fn curriculum_poison_mult(&self) -> f64 {
        match self.curriculum_stage { 0 => 0.3, 1 => 1.0, 2 => 1.5, _ => 1.0 }
    }

    fn curriculum_predator_speed_mult(&self) -> f64 {
        match self.curriculum_stage { 0 => 0.5, 1 => 1.0, 2 => 1.3, _ => 1.0 }
    }

    // ---- Metrics sampling ----
    fn sample_metrics(&mut self) {
        if self.tick % METRICS_INTERVAL != 0 { return; }

        let avg_e = if self.energies.is_empty() { 0.0 } else {
            self.energies.iter().filter(|&&e| e > 0.0).sum::<f64>() /
            self.energies.iter().filter(|&&e| e > 0.0).count().max(1) as f64
        };
        self.metrics_energy.push(avg_e);

        let alive = self.energies.iter().filter(|&&e| e > 0.0).count();
        self.metrics_population.push(alive as f64);

        let total_coop: u32 = self.brains.iter().map(|b| b.social.cooperation_count).sum();
        self.metrics_cooperation.push(total_coop as f64);

        // Shannon entropy for tribe diversity
        let mut counts = [0.0_f64; 4];
        for c in &self.colors {
            match c.as_str() {
                "#ff00cc" => counts[0] += 1.0,
                "#ccff00" => counts[1] += 1.0,
                "#00ccff" => counts[2] += 1.0,
                "#ffcc00" => counts[3] += 1.0,
                _ => {}
            }
        }
        let total = counts.iter().sum::<f64>();
        let mut entropy = 0.0;
        if total > 0.0 {
            for &c in &counts {
                if c > 0.0 {
                    let p = c / total;
                    entropy -= p * p.ln();
                }
            }
        }
        self.metrics_diversity.push(entropy);

        // Trim
        if self.metrics_energy.len() > METRICS_MAX_POINTS { self.metrics_energy.remove(0); }
        if self.metrics_population.len() > METRICS_MAX_POINTS { self.metrics_population.remove(0); }
        if self.metrics_cooperation.len() > METRICS_MAX_POINTS { self.metrics_cooperation.remove(0); }
        if self.metrics_diversity.len() > METRICS_MAX_POINTS { self.metrics_diversity.remove(0); }
    }

    // ---- Logging ----
    pub fn fetch_logs(&mut self) -> String {
        if self.log_buffer.is_empty() { return String::new(); }
        let output = self.log_buffer.join("\n");
        self.log_buffer.clear();
        output
    }

    // ---- Inspector ----
    pub fn get_agent_at(&self, x: f64, y: f64) -> i32 {
        let mut best_dist = 30.0;
        let mut best_idx = -1;
        for i in 0..self.positions.len() {
            let dist = (self.positions[i].0 - x).hypot(self.positions[i].1 - y);
            if dist < best_dist { best_dist = dist; best_idx = i as i32; }
        }
        best_idx
    }

    pub fn get_agent_brain(&self, index: usize) -> JsValue {
        if index < self.brains.len() {
            serde_wasm_bindgen::to_value(&self.brains[index]).unwrap()
        } else { JsValue::NULL }
    }

    pub fn get_tribe_stats(&self) -> Box<[i32]> {
        let mut stats = vec![0, 0, 0, 0];
        for (i, color) in self.colors.iter().enumerate() {
            if self.energies.get(i).map_or(true, |e| *e <= 0.0) { continue; }
            match color.as_str() {
                "#ff00cc" => stats[0] += 1,
                "#ccff00" => stats[1] += 1,
                "#00ccff" => stats[2] += 1,
                "#ffcc00" => stats[3] += 1,
                _ => {},
            }
        }
        stats.into_boxed_slice()
    }

    pub fn get_season(&self) -> u8 { self.season }
    pub fn get_tick(&self) -> u64 { self.tick }
    pub fn get_total_births(&self) -> u64 { self.total_births }
    pub fn get_total_deaths(&self) -> u64 { self.total_deaths }
    pub fn get_max_generation(&self) -> u32 { self.max_generation }
    pub fn get_curriculum_stage(&self) -> u8 { self.curriculum_stage }
    pub fn get_shelter_count(&self) -> usize { self.shelters.len() }
    pub fn get_alive_count(&self) -> usize {
        self.energies.iter().filter(|e| **e > 0.0).count()
    }
    pub fn get_alive_predators(&self) -> usize {
        self.predators.iter().filter(|p| p.energy > 0.0).count()
    }
    pub fn get_agent_gender(&self, index: usize) -> bool {
        if index < self.genders.len() { self.genders[index] } else { false }
    }
    pub fn get_male_count(&self) -> usize {
        self.genders.iter().enumerate()
            .filter(|(i, g)| !**g && self.energies.get(*i).map_or(false, |e| *e > 0.0))
            .count()
    }
    pub fn get_female_count(&self) -> usize {
        self.genders.iter().enumerate()
            .filter(|(i, g)| **g && self.energies.get(*i).map_or(false, |e| *e > 0.0))
            .count()
    }

    pub fn set_mutation_rate(&mut self, rate: f64) { self.mutation_rate = rate; }
    pub fn set_predator_speed(&mut self, speed: f64) { self.predator_speed = speed; }
    pub fn set_reproduction_threshold(&mut self, val: f64) { self.reproduction_threshold = val; }
    pub fn set_food_count(&mut self, count: usize) {
        let current = self.food.len();
        if count > current {
            for _ in 0..(count - current) { self.food.push((Math::random() * self.width, Math::random() * self.height)); }
        } else if count < current { self.food.truncate(count); }
    }
    pub fn resize(&mut self, width: f64, height: f64) { self.width = width; self.height = height; }
    pub fn pan(&mut self, dx: f64, dy: f64) { self.view_x += dx / self.zoom; self.view_y += dy / self.zoom; }
    pub fn zoom_at(&mut self, factor: f64) { self.zoom *= factor; }
    pub fn get_avg_energy(&self) -> f64 {
        if self.energies.is_empty() { return 0.0; }
        self.energies.iter().sum::<f64>() / self.energies.len() as f64
    }

    // ---- Metrics export ----
    pub fn get_metrics_energy(&self) -> Box<[f64]> { self.metrics_energy.clone().into_boxed_slice() }
    pub fn get_metrics_population(&self) -> Box<[f64]> { self.metrics_population.clone().into_boxed_slice() }
    pub fn get_metrics_cooperation(&self) -> Box<[f64]> { self.metrics_cooperation.clone().into_boxed_slice() }
    pub fn get_metrics_diversity(&self) -> Box<[f64]> { self.metrics_diversity.clone().into_boxed_slice() }

    // ---- Species divergence: count behavioral clusters ----
    pub fn get_species_count(&self) -> u32 {
        // Simple approach: cluster behavior vectors by distance
        let mut centroids: Vec<(f64, f64, f64)> = Vec::new(); // (avg_speed, avg_turn, avg_voice)
        let threshold = 0.15;

        for b in &self.brains {
            let bv = &b.behavior;
            if bv.samples < 50 { continue; }
            let mut found = false;
            for c in centroids.iter_mut() {
                let dist = ((bv.avg_speed - c.0).powi(2) + (bv.avg_turn - c.1).powi(2) + (bv.avg_voice - c.2).powi(2)).sqrt();
                if dist < threshold {
                    // Merge into centroid (running average)
                    c.0 = c.0 * 0.99 + bv.avg_speed * 0.01;
                    c.1 = c.1 * 0.99 + bv.avg_turn * 0.01;
                    c.2 = c.2 * 0.99 + bv.avg_voice * 0.01;
                    found = true;
                    break;
                }
            }
            if !found {
                centroids.push((bv.avg_speed, bv.avg_turn, bv.avg_voice));
            }
        }
        centroids.len().max(1) as u32
    }

    // ---- Benchmark: run N ticks and return score ----
    pub fn run_benchmark(&mut self, ticks: u32) -> JsValue {
        let start_births = self.total_births;
        let start_deaths = self.total_deaths;

        for _ in 0..ticks {
            self.step();
        }

        #[derive(Serialize)]
        struct BenchResult {
            ticks_run: u32,
            births: u64,
            deaths: u64,
            avg_energy: f64,
            max_generation: u32,
            species: u32,
            final_tick: u64,
        }

        let result = BenchResult {
            ticks_run: ticks,
            births: self.total_births - start_births,
            deaths: self.total_deaths - start_deaths,
            avg_energy: self.get_avg_energy(),
            max_generation: self.max_generation,
            species: self.get_species_count(),
            final_tick: self.tick,
        };
        serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
    }

    // ---- Save / Load ----
    pub fn export_state(&self) -> JsValue {
        let state = SaveState {
            positions: self.positions.clone(),
            angles: self.angles.clone(),
            energies: self.energies.clone(),
            brains: self.brains.clone(),
            colors: self.colors.clone(),
            voices: self.voices.clone(),
            genders: self.genders.clone(),
            food: self.food.clone(),
            poison: self.poison.clone(),
            predators: self.predators.clone(),
            rocks: self.rocks.clone(),
            mud: self.mud.clone(),
            shelters: self.shelters.clone(),
            biome_grid: self.biome_grid.clone(),
            biome_cols: self.biome_cols,
            biome_rows: self.biome_rows,
            width: self.width,
            height: self.height,
            tick: self.tick,
            season: self.season,
            season_food_mult: self.season_food_mult,
            mutation_rate: self.mutation_rate,
            predator_speed: self.predator_speed,
            reproduction_threshold: self.reproduction_threshold,
            total_births: self.total_births,
            total_deaths: self.total_deaths,
            max_generation: self.max_generation,
            curriculum_stage: self.curriculum_stage,
            metrics_energy: self.metrics_energy.clone(),
            metrics_population: self.metrics_population.clone(),
            metrics_cooperation: self.metrics_cooperation.clone(),
            metrics_diversity: self.metrics_diversity.clone(),
        };
        serde_wasm_bindgen::to_value(&state).unwrap_or(JsValue::NULL)
    }

    pub fn import_state(&mut self, val: JsValue) -> bool {
        let state: Result<SaveState, _> = serde_wasm_bindgen::from_value(val);
        match state {
            Ok(s) => {
                self.positions = s.positions;
                self.angles = s.angles;
                self.energies = s.energies;
                self.brains = s.brains;
                self.colors = s.colors;
                self.voices = s.voices;
                self.genders = s.genders;
                self.food = s.food;
                self.poison = s.poison;
                self.predators = s.predators;
                self.rocks = s.rocks;
                self.mud = s.mud;
                self.shelters = s.shelters;
                self.biome_grid = s.biome_grid;
                self.biome_cols = s.biome_cols;
                self.biome_rows = s.biome_rows;
                self.width = s.width;
                self.height = s.height;
                self.tick = s.tick;
                self.season = s.season;
                self.season_food_mult = s.season_food_mult;
                self.mutation_rate = s.mutation_rate;
                self.predator_speed = s.predator_speed;
                self.reproduction_threshold = s.reproduction_threshold;
                self.total_births = s.total_births;
                self.total_deaths = s.total_deaths;
                self.max_generation = s.max_generation;
                self.curriculum_stage = s.curriculum_stage;
                self.metrics_energy = s.metrics_energy;
                self.metrics_population = s.metrics_population;
                self.metrics_cooperation = s.metrics_cooperation;
                self.metrics_diversity = s.metrics_diversity;
                self.grid = SpatialGrid::new(self.width, self.height, 100.0);
                self.log_buffer.push("State loaded successfully".to_string());
                true
            }
            Err(_) => {
                self.log_buffer.push("Failed to load state".to_string());
                false
            }
        }
    }

    // ============================================================
    //  MAIN STEP
    // ============================================================
    pub fn step(&mut self) {
        self.tick += 1;

        // --- Curriculum ---
        self.update_curriculum();

        // --- Season Update ---
        let season_tick = self.tick % (SEASON_LENGTH * 4);
        self.season = (season_tick / SEASON_LENGTH) as u8;
        self.season_food_mult = match self.season {
            0 => 1.0,
            1 => SUMMER_FOOD_MULT,
            2 => 0.7,
            3 => WINTER_FOOD_MULT,
            _ => 1.0,
        };
        if self.tick % SEASON_LENGTH == 1 {
            let name = match self.season { 0 => "Spring", 1 => "Summer", 2 => "Autumn", 3 => "Winter", _ => "?" };
            self.log_buffer.push(format!("Season: {}", name));
        }

        let total_agents = self.positions.len();

        // 1. Refresh Spatial Grid
        self.grid.clear();
        for i in 0..total_agents {
            if self.energies[i] > 0.0 {
                self.grid.insert(self.positions[i].0, self.positions[i].1, i);
            }
        }

        // 1.5 Decay shelters
        self.shelters.retain_mut(|s| {
            if s.2 > 0 { s.2 -= 1; true } else { false }
        });

        // 2. Update Predators — living creatures with age, energy, mating
        let pred_speed_mult = self.curriculum_predator_speed_mult();
        let pred_count = self.predators.len();
        let tick_now_pred = self.tick;
        let w = self.width;
        let h = self.height;

        // Predator movement + aging + energy drain
        for i in 0..pred_count {
            self.predators[i].age += 1;

            let px = self.predators[i].x;
            let py = self.predators[i].y;
            let mut closest_agent_dist = 999999.0_f64;
            let mut target_x = px;
            let mut target_y = py;

            for j in 0..total_agents {
                if self.energies[j] <= 0.0 { continue; }
                let (ax, ay) = self.positions[j];
                let in_shelter = self.shelters.iter().any(|s| (ax - s.0).hypot(ay - s.1) < SHELTER_RADIUS);
                if in_shelter { continue; }
                let dist = (px - ax).hypot(py - ay);
                if dist < closest_agent_dist { closest_agent_dist = dist; target_x = ax; target_y = ay; }
            }

            // Old predators become slower
            let age_factor = if self.predators[i].age > PREDATOR_OLD_AGE_THRESHOLD {
                0.5 + 0.5 * (1.0 - (self.predators[i].age - PREDATOR_OLD_AGE_THRESHOLD) as f64 / (PREDATOR_MAX_AGE - PREDATOR_OLD_AGE_THRESHOLD) as f64).clamp(0.0, 1.0)
            } else { 1.0 };

            let speed = self.predator_speed * pred_speed_mult * age_factor;
            let mut dx = target_x - px;
            let mut dy = target_y - py;
            let dist = dx.hypot(dy);
            if dist > 0.0 { dx = (dx / dist) * speed; dy = (dy / dist) * speed; }

            // Repel from shelters
            for &(sx, sy, _) in &self.shelters {
                let sdist = (px - sx).hypot(py - sy);
                if sdist < SHELTER_PREDATOR_REPEL && sdist > 0.0 {
                    dx += (px - sx) / sdist * 2.0;
                    dy += (py - sy) / sdist * 2.0;
                }
            }

            // Separation from other predators
            for k in 0..pred_count {
                if i == k { continue; }
                let sep_dist = (px - self.predators[k].x).hypot(py - self.predators[k].y);
                if sep_dist < 30.0 && sep_dist > 0.0 {
                    dx += (px - self.predators[k].x) / sep_dist * 0.8;
                    dy += (py - self.predators[k].y) / sep_dist * 0.8;
                }
            }

            let mut new_px = px + dx;
            let mut new_py = py + dy;
            // Push predators out of rocks
            for (rx, ry, rr) in &self.rocks {
                let ddx = new_px - rx;
                let ddy = new_py - ry;
                let dist = ddx.hypot(ddy);
                if dist < rr + 4.0 {
                    if dist < 0.01 {
                        new_px = rx + rr + 5.0;
                        new_py = ry + rr + 5.0;
                    } else {
                        let push = (rr + 5.0 - dist) / dist;
                        new_px += ddx * push;
                        new_py += ddy * push;
                    }
                }
            }
            self.predators[i].x = new_px.clamp(5.0, w - 5.0);
            self.predators[i].y = new_py.clamp(5.0, h - 5.0);

            // Energy drain
            self.predators[i].energy -= PREDATOR_MOVE_COST;
            if self.predators[i].angle.is_nan() { self.predators[i].angle = 0.0; }
            self.predators[i].angle = dy.atan2(dx);
        }

        // Predator mating
        let mut new_predators: Vec<Predator> = Vec::new();
        for i in 0..pred_count {
            if self.predators[i].energy <= 0.0 { continue; }
            if self.predators[i].energy < PREDATOR_MATE_THRESHOLD { continue; }
            if tick_now_pred.saturating_sub(self.predators[i].last_mate_tick) < PREDATOR_MATE_COOLDOWN { continue; }
            if !self.predators[i].female { continue; }
            // Find a male partner nearby
            for j in 0..pred_count {
                if i == j || self.predators[j].energy <= 0.0 || self.predators[j].female { continue; }
                if self.predators[j].energy < PREDATOR_MATE_THRESHOLD { continue; }
                let dist = (self.predators[i].x - self.predators[j].x).hypot(self.predators[i].y - self.predators[j].y);
                if dist < PREDATOR_MATE_RADIUS && (self.predators.len() + new_predators.len()) < PREDATOR_MAX_COUNT {
                    self.predators[i].energy -= PREDATOR_MATE_COST;
                    self.predators[j].energy -= PREDATOR_MATE_COST;
                    self.predators[i].last_mate_tick = tick_now_pred;
                    self.predators[j].last_mate_tick = tick_now_pred;
                    let cx = (self.predators[i].x + self.predators[j].x) * 0.5 + (Math::random() - 0.5) * 20.0;
                    let cy = (self.predators[i].y + self.predators[j].y) * 0.5 + (Math::random() - 0.5) * 20.0;
                    new_predators.push(Predator::new(cx.clamp(0.0, w), cy.clamp(0.0, h)));
                    self.log_buffer.push("Predator offspring born!".to_string());
                    break;
                }
            }
        }
        self.predators.extend(new_predators);

        // Predator death (old age or starved)
        self.predators.retain(|p| {
            p.energy > 0.0 && p.age < PREDATOR_MAX_AGE
        });
        // Ensure at least 1 predator exists
        if self.predators.is_empty() {
            self.predators.push(Predator::new(Math::random() * w, Math::random() * h));
        }

        // 3. Update Agents
        let social_signals: Vec<(f64, u8, f64)> = self.brains.iter()
            .map(|b| (b.social.signal_out, b.social.signal_type, b.social.reputation))
            .collect();
        let positions_snap: Vec<(f64, f64)> = self.positions.clone();
        let energies_snap: Vec<f64> = self.energies.clone();
        let voices_snap: Vec<f64> = self.voices.clone();
        let tick_now = self.tick;
        let mut mated_this_tick = vec![false; total_agents];

        for i in 0..total_agents {
            if self.energies[i] <= 0.0 { continue; }

            let (my_x, my_y) = self.positions[i];
            let my_angle = self.angles[i];

            // --- Biome at agent position ---
            let biome = self.biome_at(my_x, my_y);
            let biome_speed = Self::biome_speed_mult(biome);

            // --- Food perception ---
            let mut closest_food_dist = 9999.0;
            let mut food_angle_diff = 0.0;
            let mut closest_food_index = 0;
            for (idx, (fx, fy)) in self.food.iter().enumerate() {
                let dx = fx - my_x; let dy = fy - my_y;
                let dist = dx.hypot(dy);
                if dist < closest_food_dist {
                    closest_food_dist = dist; closest_food_index = idx;
                    food_angle_diff = dy.atan2(dx) - my_angle;
                }
            }

            // --- Poison perception ---
            let mut closest_poison_dist = 9999.0;
            let mut poison_angle_diff = 0.0;
            let mut closest_poison_index = 0;
            for (idx, (px, py)) in self.poison.iter().enumerate() {
                let dx = px - my_x; let dy = py - my_y;
                let dist = dx.hypot(dy);
                if dist < closest_poison_dist {
                    closest_poison_dist = dist; closest_poison_index = idx;
                    poison_angle_diff = dy.atan2(dx) - my_angle;
                }
            }

            // --- Neighbor perception ---
            let mut closest_friend_dist = 9999.0;
            let mut hearing_vol = 0.0;
            let mut nearby_count = 0.0;
            let mut avg_signal_type: f64 = 0.0;
            let mut avg_reputation: f64 = 0.0;
            let mut nearest_same_tribe_dist = 9999.0;

            let neighbors = self.grid.query(my_x, my_y);
            for &j in &neighbors {
                if i == j { continue; }
                let (fx, fy) = positions_snap[j];
                let dist = (fx - my_x).hypot(fy - my_y);
                if dist < closest_friend_dist { closest_friend_dist = dist; }
                if dist < SOCIAL_RADIUS {
                    nearby_count += 1.0;
                    hearing_vol += voices_snap[j] * (1.0 - dist / SOCIAL_RADIUS);
                    avg_signal_type += social_signals[j].1 as f64;
                    avg_reputation += social_signals[j].2;
                    if self.colors[j] == self.colors[i] && dist < nearest_same_tribe_dist {
                        nearest_same_tribe_dist = dist;
                    }
                }
            }
            if nearby_count > 0.0 {
                avg_signal_type /= nearby_count;
                avg_reputation /= nearby_count;
            }

            // --- Predator perception ---
            let mut closest_pred_dist = 9999.0;
            let mut pred_angle_diff = 0.0;
            let mut closest_pred_index = 0;
            for (idx, pred) in self.predators.iter().enumerate() {
                let dx = pred.x - my_x; let dy = pred.y - my_y;
                let dist = dx.hypot(dy);
                if dist < closest_pred_dist {
                    closest_pred_dist = dist; closest_pred_index = idx;
                    pred_angle_diff = dy.atan2(dx) - my_angle;
                }
            }

            // --- Obstacle whiskers ---
            let check_obstacle = |angle_offset: f64| -> f64 {
                let angle = my_angle + angle_offset;
                let rx = my_x + angle.cos() * WHISKER_LEN;
                let ry = my_y + angle.sin() * WHISKER_LEN;
                if rx < 0.0 || rx > self.width || ry < 0.0 || ry > self.height { return 1.0; }
                for (rock_x, rock_y, rock_r) in &self.rocks { if (rx - rock_x).hypot(ry - rock_y) < *rock_r { return 1.0; } }
                0.0
            };
            let wall_l = check_obstacle(-0.78);
            let wall_c = check_obstacle(0.0);
            let wall_r = check_obstacle(0.78);

            let mut in_mud = 0.0;
            for (mx, my, mr) in &self.mud { if (my_x - mx).hypot(my_y - my) < *mr { in_mud = 1.0; break; } }

            // --- Shelter detection ---
            let in_shelter = self.shelters.iter().any(|s| (my_x - s.0).hypot(my_y - s.1) < SHELTER_RADIUS);
            // Environment signal: -1 mud, 0 normal, 1 shelter (shelter overrides mud)
            let env_signal = if in_shelter { 1.0 } else if in_mud > 0.0 { -1.0 } else { 0.0 };

            // --- Long-term memory readout ---
            let quadrant = MemorySystem::get_quadrant(my_x, my_y, self.width, self.height);
            let ltm = &self.brains[i].memory.long_term;
            let ltm_food = if quadrant * 2 < ltm.len() { ltm[quadrant * 2] } else { 0.0 };
            let ltm_danger = if quadrant * 2 + 1 < ltm.len() { ltm[quadrant * 2 + 1] } else { 0.0 };

            // --- Episodic memory recall ---
            let episodic_recall = self.brains[i].episodic.recall_nearby(my_x, my_y, 200.0);

            // --- Neurochemistry readout ---
            let chem = self.brains[i].neurochemistry.as_slice();

            // === BUILD 22-element input vector ===
            let inputs: [f64; 22] = [
                (closest_food_dist / self.width).min(1.0),
                food_angle_diff.sin(),
                food_angle_diff.cos(),
                (closest_pred_dist / self.width).min(1.0),
                pred_angle_diff.sin(),
                pred_angle_diff.cos(),
                self.energies[i] / ENERGY_CAP,
                (closest_friend_dist / 200.0).min(1.0),
                wall_l, wall_c, wall_r,
                hearing_vol.min(1.0),
                env_signal,
                (ltm_food + episodic_recall * 0.5).clamp(-1.0, 1.0),
                ltm_danger,
                (avg_signal_type / 3.0).min(1.0),
                avg_reputation,
                self.season as f64 / 3.0,
                chem[0], chem[1], chem[2], chem[3],
            ];

            // === Update goal system ===
            // Threat is only high when predator is CLOSE (within 300px), not across the whole map
            let threat_level = if closest_pred_dist < 300.0 {
                (1.0 - closest_pred_dist / 300.0).clamp(0.0, 1.0)
            } else { 0.0 };
            let chem_snapshot = self.brains[i].neurochemistry.clone();
            self.brains[i].goals.update(self.energies[i], threat_level, &chem_snapshot);

            // === Brain forward pass ===
            let outputs = self.brains[i].process(&inputs);

            let turn_force = outputs[0] * TURN_SPEED;
            let mut speed = (outputs[1] + 1.0) * AGENT_SPEED_MODIFIER;
            self.voices[i] = outputs[2].max(0.0);
            let build_intent = outputs[3];
            let flee_intent = outputs[4];
            let mate_intent = outputs[5];
            let explore_intent = outputs[6];

            // Neurochemistry modulates speed
            let cortisol = self.brains[i].neurochemistry.cortisol;
            let dopamine = self.brains[i].neurochemistry.dopamine;
            if flee_intent > 0.5 { speed *= 1.0 + cortisol * 0.5; }
            if explore_intent > 0.3 { speed *= 1.0 + dopamine * 0.3; }

            // Biome affects speed
            speed *= biome_speed;
            if in_mud > 0.0 { speed *= 0.3; }

            // Update behavior vector
            self.brains[i].behavior.update(speed, turn_force, self.voices[i]);

            self.angles[i] += turn_force;
            let vx = self.angles[i].cos() * speed;
            let vy = self.angles[i].sin() * speed;
            let mut new_x = my_x + vx;
            let mut new_y = my_y + vy;

            // Rock collision: push out if inside, block if entering
            for (rx, ry, rr) in &self.rocks {
                let dx = new_x - rx;
                let dy = new_y - ry;
                let dist = dx.hypot(dy);
                if dist < rr + 4.0 {
                    if dist < 0.01 {
                        new_x = rx + rr + 5.0;
                        new_y = ry + rr + 5.0;
                    } else {
                        let push = (rr + 5.0 - dist) / dist;
                        new_x += dx * push;
                        new_y += dy * push;
                    }
                }
            }
            self.positions[i] = (new_x, new_y);
            self.positions[i].0 = self.positions[i].0.clamp(5.0, self.width - 5.0);
            self.positions[i].1 = self.positions[i].1.clamp(5.0, self.height - 5.0);

            // --- Movement cost ---
            let mut cost = speed * MOVE_COST;
            if in_mud > 0.0 { cost *= 1.5; }
            cost += self.voices[i] * 0.03;
            if self.season == 3 {
                if in_shelter { cost *= 1.05; } else { cost *= 1.3; }
            } else if in_shelter {
                cost *= 0.5; // shelter reduces energy cost
            }
            // Swamp extra poison damage (reduced)
            if biome == BIOME_SWAMP && !in_shelter && Math::random() < 0.003 {
                self.energies[i] -= 3.0;
            }
            self.energies[i] -= cost;

            // --- Curiosity: visit tracking ---
            self.brains[i].curiosity.visit(self.positions[i].0, self.positions[i].1, self.width, self.height);

            // --- Reward signal ---
            let mut reward = 0.0;
            let mut social_signal: f64 = 0.0;

            // --- Eat food ---
            if closest_food_dist < EAT_RADIUS {
                let food_bonus = FOOD_ENERGY * Self::biome_food_mult(biome);
                self.energies[i] += food_bonus;
                if self.energies[i] > ENERGY_CAP { self.energies[i] = ENERGY_CAP; }
                self.food[closest_food_index] = (Math::random() * self.width, Math::random() * self.height);
                reward += 1.0;
                // Eating food makes agents happy — boost dopamine and serotonin, reduce cortisol
                self.brains[i].neurochemistry.dopamine = (self.brains[i].neurochemistry.dopamine + 0.15).min(1.0);
                self.brains[i].neurochemistry.serotonin = (self.brains[i].neurochemistry.serotonin + 0.1).min(1.0);
                self.brains[i].neurochemistry.cortisol = (self.brains[i].neurochemistry.cortisol - 0.1).max(0.0);
                let q = MemorySystem::get_quadrant(my_x, my_y, self.width, self.height);
                self.brains[i].memory.record_event(q, true, 1.0);
                self.brains[i].episodic.record(my_x, my_y, 1.0, 0, tick_now);
                self.brains[i].behavior.food_efficiency += 0.01;
            }

            // --- Poison ---
            if closest_poison_dist < EAT_RADIUS {
                let dmg = POISON_DAMAGE * self.curriculum_poison_mult();
                self.energies[i] -= dmg;
                self.poison[closest_poison_index] = (Math::random() * self.width, Math::random() * self.height);
                reward -= 1.5;
                let q = MemorySystem::get_quadrant(my_x, my_y, self.width, self.height);
                self.brains[i].memory.record_event(q, false, 1.0);
                self.brains[i].episodic.record(my_x, my_y, -1.5, 1, tick_now);
                self.log_buffer.push(format!("Agent {} ate poison!", i));
            }

            // --- Food sharing / Social games ---
            if self.energies[i] > ENERGY_CAP * 0.7 {
                for &j in &neighbors {
                    if i == j || self.energies[j] <= 0.0 { continue; }
                    let dist = (positions_snap[j].0 - my_x).hypot(positions_snap[j].1 - my_y);
                    if dist < SHARE_RADIUS && self.colors[j] == self.colors[i] && energies_snap[j] < 40.0 {
                        // Betrayal check: high deception + low reputation neighbor = steal
                        let my_deception = self.brains[i].social.deception_tendency;
                        if my_deception > 0.6 && Math::random() < my_deception * 0.3 {
                            // Betray: steal energy instead of sharing
                            let stolen = FOOD_SHARE_AMOUNT * 0.5;
                            self.energies[i] += stolen;
                            self.energies[j] -= stolen;
                            self.brains[i].social.update_reputation(false);
                            self.brains[i].social.betrayal_count += 1;
                            reward -= 0.2; // small guilt penalty
                            self.brains[i].episodic.record(my_x, my_y, -0.5, 3, tick_now);
                        } else {
                            // Genuine sharing
                            self.energies[i] -= FOOD_SHARE_AMOUNT;
                            self.energies[j] += FOOD_SHARE_AMOUNT;
                            self.brains[i].social.update_reputation(true);
                            social_signal += 0.5;
                            self.brains[i].behavior.social_score += 0.01;
                            self.brains[i].episodic.record(my_x, my_y, 0.5, 3, tick_now);
                        }
                        break;
                    }
                }
            }

            // --- Deceptive signaling cost ---
            let my_deception = self.brains[i].social.deception_tendency;
            if my_deception > 0.5 && self.brains[i].social.signal_type == 1 && closest_food_dist > 100.0 {
                // Emitting fake "food here" signal — costs energy
                self.energies[i] -= DECEPTION_COST;
            }

            // --- Predator encounter (shelters protect!) ---
            if closest_pred_dist < PREDATOR_KILL_RADIUS && !in_shelter {
                if self.energies[i] > WARRIOR_THRESHOLD {
                    // Agent kills predator
                    self.predators[closest_pred_index].energy = 0.0;
                    self.energies[i] -= BATTLE_COST;
                    reward += 0.5;
                    self.brains[i].episodic.record(my_x, my_y, 0.5, 2, tick_now);
                    self.log_buffer.push(format!("Agent {} killed a Predator!", i));
                } else {
                    // Predator eats agent — predator gains energy
                    self.predators[closest_pred_index].energy = (self.predators[closest_pred_index].energy + PREDATOR_KILL_ENERGY).min(PREDATOR_ENERGY_CAP);
                    self.energies[i] = -10.0;
                    reward -= 2.0;
                    let q = MemorySystem::get_quadrant(my_x, my_y, self.width, self.height);
                    self.brains[i].memory.record_event(q, false, 1.0);
                    self.brains[i].episodic.record(my_x, my_y, -2.0, 2, tick_now);
                }
            }

            // --- Shelter building ---
            if build_intent > 0.5 && self.energies[i] > SHELTER_BUILD_COST + 20.0
                && self.shelters.len() < SHELTER_MAX
            {
                // Must be near a rock to build
                let near_rock = self.rocks.iter().any(|(rx, ry, rr)| {
                    (my_x - rx).hypot(my_y - ry) < rr + SHELTER_ROCK_RANGE
                });
                // Don't build if already a shelter nearby
                let shelter_exists = self.shelters.iter().any(|s| {
                    (my_x - s.0).hypot(my_y - s.1) < SHELTER_RADIUS * 2.0
                });
                if near_rock && !shelter_exists {
                    self.shelters.push((my_x, my_y, SHELTER_DURABILITY));
                    self.energies[i] -= SHELTER_BUILD_COST;
                    reward += 0.8;
                    self.log_buffer.push(format!("Agent {} built a shelter!", i));
                }
            }

            // --- Proactive mating (requires opposite gender + same tribe) ---
            if mate_intent > 0.2 && !mated_this_tick[i]
                && self.energies[i] > self.reproduction_threshold
                && tick_now.saturating_sub(self.brains[i].last_mate_tick) > MATING_COOLDOWN
            {
                let my_gender = self.genders[i];
                let mut best_mate: Option<usize> = None;
                let mut best_dist = MATING_RADIUS;
                for &j in &neighbors {
                    if j == i || mated_this_tick[j] || self.energies[j] <= 0.0 { continue; }
                    if self.energies[j] <= self.reproduction_threshold * 0.7 { continue; }
                    if self.colors[j] != self.colors[i] { continue; }
                    if self.genders[j] == my_gender { continue; } // Must be opposite gender
                    let dist = (positions_snap[j].0 - my_x).hypot(positions_snap[j].1 - my_y);
                    if dist < best_dist {
                        best_dist = dist;
                        best_mate = Some(j);
                    }
                }
                if let Some(j) = best_mate {
                    let mut child_brain = self.brains[i].crossover(&self.brains[j]);
                    child_brain = child_brain.mutate(self.mutation_rate);
                    if child_brain.generation > self.max_generation {
                        self.max_generation = child_brain.generation;
                    }
                    // Find a dead slot to place the child
                    let mut child_slot: Option<usize> = None;
                    for k in 0..total_agents {
                        if self.energies[k] <= 0.0 {
                            child_slot = Some(k);
                            break;
                        }
                    }
                    if let Some(slot) = child_slot {
                        self.brains[slot] = child_brain;
                        self.colors[slot] = self.colors[i].clone();
                        self.genders[slot] = Math::random() > 0.5;
                        self.positions[slot] = (
                            my_x + (Math::random() - 0.5) * 15.0,
                            my_y + (Math::random() - 0.5) * 15.0,
                        );
                        self.energies[slot] = 60.0;
                        self.energies[i] -= MATING_ENERGY_COST;
                        self.energies[j] -= MATING_ENERGY_COST;
                        self.brains[i].last_mate_tick = tick_now;
                        self.brains[j].last_mate_tick = tick_now;
                        mated_this_tick[i] = true;
                        mated_this_tick[j] = true;
                        self.total_births += 1;
                        reward += 0.5;
                    }
                }
            }

            // --- Intrinsic reward ---
            let intrinsic = self.brains[i].get_intrinsic_reward();
            reward += intrinsic;

            // --- Update neurochemistry ---
            self.brains[i].neurochemistry.update(
                reward.max(0.0_f64).min(1.0),
                threat_level,
                social_signal.min(1.0_f64),
            );
            self.brains[i].memory.last_reward = reward;
            self.brains[i].memory.lifetime_reward += reward;

            // --- Online learning (REINFORCE) ---
            if tick_now % LEARNING_INTERVAL == 0 {
                self.brains[i].learn_from_reward(reward, ONLINE_LEARNING_RATE);
            }

            // --- Death (no random respawn — only mating creates new agents) ---
            if self.energies[i] <= 0.0 {
                self.total_deaths += 1;
                self.energies[i] = 0.0; // Stay dead; slot reused by proactive mating
                self.voices[i] = 0.0;
            }
        }

        // --- Logical food growth (spreads near existing food, biome-dependent) ---
        if self.food.len() < FOOD_MAX {
            let food_count = self.food.len();
            for fi in 0..food_count {
                if Math::random() > FOOD_GROW_CHANCE { continue; }
                let (fx, fy) = self.food[fi];
                // Determine biome at this food's position
                let bc = (fx / BIOME_CELL_SIZE) as usize;
                let br = (fy / BIOME_CELL_SIZE) as usize;
                let bidx = br * self.biome_cols + bc;
                let biome = if bidx < self.biome_grid.len() { self.biome_grid[bidx] } else { 0 };
                let biome_mult = match biome {
                    BIOME_FOREST => 2.0,
                    BIOME_PLAINS => 1.0,
                    BIOME_SWAMP  => 0.6,
                    BIOME_DESERT => 0.15,
                    _ => 1.0,
                };
                if Math::random() > biome_mult * self.season_food_mult { continue; }
                let angle = Math::random() * std::f64::consts::TAU;
                let dist = Math::random() * FOOD_GROW_RADIUS;
                let nx = (fx + angle.cos() * dist).clamp(5.0, self.width - 5.0);
                let ny = (fy + angle.sin() * dist).clamp(5.0, self.height - 5.0);
                // Don't grow on rocks
                let on_rock = self.rocks.iter().any(|(rx, ry, rr)| (nx - rx).hypot(ny - ry) < *rr);
                if !on_rock {
                    self.food.push((nx, ny));
                    if self.food.len() >= FOOD_MAX { break; }
                }
            }
        }
        // Poison respawn (curriculum-adjusted)
        let poison_chance = 0.01 * self.curriculum_poison_mult();
        if Math::random() < poison_chance && self.poison.len() < POISON_COUNT * 2 {
            self.poison.push((Math::random() * self.width, Math::random() * self.height));
        }

        // --- Minimum population safety net ---
        // If population drops critically low, revive some dead agents to prevent extinction
        let alive = self.energies.iter().filter(|e| **e > 0.0).count();
        let min_pop = 30;
        if alive < min_pop {
            let mut revived = 0;
            for k in 0..total_agents {
                if revived >= (min_pop - alive) { break; }
                if self.energies[k] <= 0.0 {
                    self.energies[k] = STARTING_ENERGY;
                    self.positions[k] = (
                        Math::random() * self.width,
                        Math::random() * self.height,
                    );
                    self.genders[k] = Math::random() > 0.5;
                    self.voices[k] = 0.0;
                    revived += 1;
                }
            }
        }

        // --- Sample metrics ---
        self.sample_metrics();
    }

    // ============================================================
    //  DRAW
    // ============================================================
    pub fn draw(&self, context: &web_sys::CanvasRenderingContext2d) {
        // Background varies by season
        let bg = match self.season {
            0 => "#0a1a0a",
            1 => "#0a0a00",
            2 => "#1a0f00",
            3 => "#050510",
            _ => "#111",
        };
        context.set_fill_style(&JsValue::from_str(bg));
        context.fill_rect(0.0, 0.0, self.width, self.height);
        context.save();
        context.scale(self.zoom, self.zoom).unwrap();
        context.translate(-self.view_x, -self.view_y).unwrap();

        // Draw biome grid
        for r in 0..self.biome_rows {
            for c in 0..self.biome_cols {
                let idx = r * self.biome_cols + c;
                let biome = if idx < self.biome_grid.len() { self.biome_grid[idx] } else { 0 };
                let color = match biome {
                    BIOME_FOREST => "rgba(0, 40, 0, 0.3)",
                    BIOME_DESERT => "rgba(40, 30, 0, 0.3)",
                    BIOME_SWAMP  => "rgba(0, 20, 30, 0.4)",
                    _ => "rgba(0, 0, 0, 0)",
                };
                if biome != BIOME_PLAINS {
                    context.set_fill_style(&JsValue::from_str(color));
                    context.fill_rect(c as f64 * BIOME_CELL_SIZE, r as f64 * BIOME_CELL_SIZE, BIOME_CELL_SIZE, BIOME_CELL_SIZE);
                }
            }
        }

        // Border
        context.set_stroke_style(&JsValue::from_str("#222"));
        context.set_line_width(5.0);
        context.stroke_rect(0.0, 0.0, self.width, self.height);

        // Mud
        context.set_fill_style(&JsValue::from_str("#1a2b3c"));
        for (mx, my, mr) in &self.mud { context.begin_path(); context.arc(*mx, *my, *mr, 0.0, 6.28).unwrap(); context.fill(); }

        // Rocks
        context.set_fill_style(&JsValue::from_str("#555"));
        for (rx, ry, rr) in &self.rocks { context.begin_path(); context.arc(*rx, *ry, *rr, 0.0, 6.28).unwrap(); context.fill(); }

        // Shelters
        for &(sx, sy, dur) in &self.shelters {
            let alpha = (dur as f64 / SHELTER_DURABILITY as f64).clamp(0.2, 0.6);
            let color = format!("rgba(180, 140, 60, {})", alpha);
            context.set_fill_style(&JsValue::from_str(&color));
            context.begin_path();
            context.arc(sx, sy, SHELTER_RADIUS, 0.0, 6.28).unwrap();
            context.fill();
            // Hut icon (triangle roof)
            context.set_fill_style(&JsValue::from_str("rgba(120, 80, 30, 0.8)"));
            context.begin_path();
            context.move_to(sx, sy - 12.0);
            context.line_to(sx + 10.0, sy - 2.0);
            context.line_to(sx - 10.0, sy - 2.0);
            context.fill();
            // Hut body
            context.set_fill_style(&JsValue::from_str("rgba(100, 70, 30, 0.7)"));
            context.fill_rect(sx - 7.0, sy - 2.0, 14.0, 10.0);
        }

        // Food (green)
        context.set_fill_style(&JsValue::from_str("#00ff00"));
        for (fx, fy) in &self.food { context.begin_path(); context.arc(*fx, *fy, 3.0, 0.0, 6.28).unwrap(); context.fill(); }

        // Poison (yellowish-green)
        context.set_fill_style(&JsValue::from_str("#88ff00"));
        for (px, py) in &self.poison { context.begin_path(); context.arc(*px, *py, 3.0, 0.0, 6.28).unwrap(); context.fill(); }

        // Predators (triangles — red=female, dark-red=male, faded when old)
        for pred in &self.predators {
            if pred.energy <= 0.0 { continue; }
            let age_ratio = pred.age as f64 / PREDATOR_MAX_AGE as f64;
            let alpha = if age_ratio > 0.75 { 0.5 } else { 1.0 };
            let color = if pred.female {
                format!("rgba(255, 50, 50, {})", alpha)
            } else {
                format!("rgba(180, 30, 30, {})", alpha)
            };
            context.set_fill_style(&JsValue::from_str(&color));
            context.begin_path();
            context.move_to(pred.x, pred.y - 10.0);
            context.line_to(pred.x + 10.0, pred.y + 10.0);
            context.line_to(pred.x - 10.0, pred.y + 10.0);
            context.fill();
        }

        // Agents
        for i in 0..self.positions.len() {
            if self.energies[i] <= 0.0 { continue; }
            let (x, y) = self.positions[i];
            context.set_fill_style(&JsValue::from_str(&self.colors[i]));
            context.set_global_alpha((self.energies[i] / ENERGY_CAP).clamp(0.2, 1.0));
            context.save();
            context.translate(x, y).unwrap();
            context.rotate(self.angles[i]).unwrap();
            context.begin_path();
            context.move_to(6.0, 0.0);
            context.line_to(-4.0, 4.0);
            context.line_to(-4.0, -4.0);
            context.fill();

            // Warrior glow
            if self.energies[i] > WARRIOR_THRESHOLD {
                context.set_stroke_style(&JsValue::from_str("#ffffff"));
                context.set_line_width(2.0);
                context.stroke();
            }
            context.restore();

            // Voice ring
            if self.voices[i] > 0.5 {
                let sig_color = match self.brains[i].social.signal_type {
                    1 => "rgba(0, 255, 0, 0.3)",
                    2 => "rgba(255, 0, 0, 0.3)",
                    3 => "rgba(255, 0, 255, 0.3)",
                    _ => "rgba(255, 255, 255, 0.2)",
                };
                context.set_stroke_style(&JsValue::from_str(sig_color));
                context.set_line_width(1.0);
                context.begin_path();
                context.arc(x, y, 15.0 + (self.voices[i] * 10.0), 0.0, 6.28).unwrap();
                context.stroke();
            }

            // Drive indicator dot
            let drive_color = match self.brains[i].last_drive {
                0 => "#0f0",
                1 => "#f00",
                2 => "#f0f",
                3 => "#0ff",
                4 => "#ff0",
                _ => "#fff",
            };
            context.set_fill_style(&JsValue::from_str(drive_color));
            context.set_global_alpha(0.7);
            context.begin_path();
            context.arc(x, y - 8.0, 2.0, 0.0, 6.28).unwrap();
            context.fill();

            // Gender indicator (pink=female, blue=male)
            if i < self.genders.len() {
                let g_color = if self.genders[i] { "#ff69b4" } else { "#4488ff" };
                context.set_fill_style(&JsValue::from_str(g_color));
                context.set_global_alpha(0.8);
                context.begin_path();
                context.arc(x, y + 8.0, 1.5, 0.0, 6.28).unwrap();
                context.fill();
            }
        }
        context.set_global_alpha(1.0);
        context.restore();
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}
