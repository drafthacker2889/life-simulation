use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Math;

// IMPORT MODULES
mod constants;
mod brain;

use brain::Brain;
use constants::*; 

// --- THE SIMULATION WORLD ---
#[wasm_bindgen]
pub struct Simulation {
    positions: Vec<(f64, f64)>, 
    angles: Vec<f64>,
    energies: Vec<f64>,
    brains: Vec<Brain>,
    colors: Vec<String>,
    voices: Vec<f64>,
    
    food: Vec<(f64, f64)>, 
    predators: Vec<(f64, f64)>,
    
    rocks: Vec<(f64, f64, f64)>, // x, y, radius
    mud: Vec<(f64, f64, f64)>,   // x, y, radius

    width: f64,
    height: f64,
    
    mutation_rate: f64,
    predator_speed: f64,       
    reproduction_threshold: f64, 

    view_x: f64, view_y: f64, zoom: f64,
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
        let mut food = Vec::new();
        let mut predators = Vec::new();
        let mut rocks = Vec::new();
        let mut mud = Vec::new();

        let color_palette = ["#ff00cc", "#ccff00", "#00ccff", "#ffcc00"];

        // Initialize Agents
        for _ in 0..AGENT_COUNT {
            positions.push((Math::random() * width, Math::random() * height));
            angles.push(Math::random() * 6.28);
            energies.push(STARTING_ENERGY);
            brains.push(Brain::new());
            let color_idx = (Math::random() * 4.0) as usize;
            colors.push(color_palette[color_idx].to_string());
            voices.push(0.0);
        }

        // Initialize Food
        for _ in 0..FOOD_COUNT {
            food.push((Math::random() * width, Math::random() * height));
        }

        // Initialize Predators
        for _ in 0..PREDATOR_COUNT {
            predators.push((Math::random() * width, Math::random() * height));
        }

        // Initialize Terrain
        for _ in 0..15 { rocks.push((Math::random() * width, Math::random() * height, 20.0 + Math::random() * 30.0)); }
        for _ in 0..10 { mud.push((Math::random() * width, Math::random() * height, 40.0 + Math::random() * 60.0)); }

        Simulation { 
            positions, angles, energies, brains, colors, voices, 
            food, predators, rocks, mud,
            width, height, 
            mutation_rate: BASE_MUTATION_RATE,
            predator_speed: 2.2, 
            reproduction_threshold: 60.0, 
            view_x: 0.0, view_y: 0.0, zoom: 1.0,
        }
    }

    pub fn get_tribe_stats(&self) -> Box<[i32]> {
        let mut stats = vec![0, 0, 0, 0];
        for color in &self.colors {
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

    // Controls
    pub fn set_mutation_rate(&mut self, rate: f64) { self.mutation_rate = rate; }
    pub fn set_predator_speed(&mut self, speed: f64) { self.predator_speed = speed; }
    pub fn set_reproduction_threshold(&mut self, val: f64) { self.reproduction_threshold = val; }
    pub fn set_food_count(&mut self, count: usize) {
        let current = self.food.len();
        if count > current {
            for _ in 0..(count - current) {
                self.food.push((Math::random() * self.width, Math::random() * self.height));
            }
        } else if count < current {
            self.food.truncate(count);
        }
    }
    pub fn resize(&mut self, width: f64, height: f64) { self.width = width; self.height = height; }
    pub fn pan(&mut self, dx: f64, dy: f64) { self.view_x += dx / self.zoom; self.view_y += dy / self.zoom; }
    pub fn zoom_at(&mut self, factor: f64) {
        self.zoom *= factor;
        if self.zoom < 0.1 { self.zoom = 0.1; }
        if self.zoom > 5.0 { self.zoom = 5.0; }
    }
    pub fn get_avg_energy(&self) -> f64 {
        if self.energies.is_empty() { return 0.0; }
        let sum: f64 = self.energies.iter().sum();
        sum / self.energies.len() as f64
    }

    // --- MAIN LOGIC LOOP ---
    pub fn step(&mut self) {
        let total_agents = self.positions.len();

        // 1. UPDATE PREDATORS
        for i in 0..self.predators.len() {
            let (px, py) = self.predators[i];
            let mut closest_agent_dist = 999999.0;
            let mut target_x = px; 
            let mut target_y = py;

            for j in 0..total_agents {
                if self.energies[j] <= 0.0 { continue; } 
                let (ax, ay) = self.positions[j];
                let dist = (px - ax).hypot(py - ay);
                if dist < closest_agent_dist {
                    closest_agent_dist = dist;
                    target_x = ax;
                    target_y = ay;
                }
            }

            let speed = self.predator_speed; 
            let mut dx = target_x - px;
            let mut dy = target_y - py;
            let dist = dx.hypot(dy);
            
            if dist > 0.0 { dx = (dx / dist) * speed; dy = (dy / dist) * speed; }

            // Separation
            for k in 0..self.predators.len() {
                if i == k { continue; }
                let (ox, oy) = self.predators[k];
                let sep_dist = (px - ox).hypot(py - oy);
                if sep_dist < 30.0 && sep_dist > 0.0 {
                    let push_x = (px - ox) / sep_dist;
                    let push_y = (py - oy) / sep_dist;
                    dx += push_x * 0.8; dy += push_y * 0.8;
                }
            }
            // Rocks block Predators too!
            let new_px = self.predators[i].0 + dx;
            let new_py = self.predators[i].1 + dy;
            let mut hit_rock = false;
            for (rx, ry, r_rad) in &self.rocks {
                if (new_px - rx).hypot(new_py - ry) < *r_rad { hit_rock = true; break; }
            }
            if !hit_rock {
                self.predators[i].0 = new_px;
                self.predators[i].1 = new_py;
            }

            // Wall Clamp
            if self.predators[i].0 < 0.0 { self.predators[i].0 = 0.0; }
            if self.predators[i].0 > self.width { self.predators[i].0 = self.width; }
            if self.predators[i].1 < 0.0 { self.predators[i].1 = 0.0; }
            if self.predators[i].1 > self.height { self.predators[i].1 = self.height; }
        }

        // 2. UPDATE AGENTS
        for i in 0..total_agents {
            let (my_x, my_y) = self.positions[i];
            let my_angle = self.angles[i];

            // SENSORS
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

            let mut closest_pred_dist = 9999.0;
            let mut pred_angle_diff = 0.0;
            let mut closest_pred_index = 0; 
            for (idx, (px, py)) in self.predators.iter().enumerate() {
                let dx = px - my_x; let dy = py - my_y;
                let dist = dx.hypot(dy);
                if dist < closest_pred_dist {
                    closest_pred_dist = dist; closest_pred_index = idx;
                    pred_angle_diff = dy.atan2(dx) - my_angle;
                }
            }

            let mut closest_friend_dist = 9999.0;
            let mut hearing_vol = 0.0; 
            for j in 0..total_agents {
                if i == j { continue; } 
                let (fx, fy) = self.positions[j];
                let dist = (fx - my_x).hypot(fy - my_y);
                if dist < closest_friend_dist { closest_friend_dist = dist; }
                
                if dist < 100.0 {
                    hearing_vol += self.voices[j] * (1.0 - dist/100.0);
                }
            }

            // WHISKERS
            let check_obstacle = |angle_offset: f64| -> f64 {
                let angle = my_angle + angle_offset;
                let rx = my_x + angle.cos() * WHISKER_LEN;
                let ry = my_y + angle.sin() * WHISKER_LEN;
                if rx < 0.0 || rx > self.width || ry < 0.0 || ry > self.height { return 1.0; }
                for (rock_x, rock_y, rock_r) in &self.rocks {
                    if (rx - rock_x).hypot(ry - rock_y) < *rock_r { return 1.0; }
                }
                0.0
            };
            let wall_l = check_obstacle(-0.78); 
            let wall_c = check_obstacle(0.0);
            let wall_r = check_obstacle(0.78); 

            let mut in_mud = 0.0;
            for (mx, my, mr) in &self.mud {
                if (my_x - mx).hypot(my_y - my) < *mr { in_mud = 1.0; break; }
            }

            // PROCESS BRAIN
            let inputs = [
                (closest_food_dist / self.width).min(1.0),
                food_angle_diff.sin(), 
                (closest_pred_dist / self.width).min(1.0),
                pred_angle_diff.sin(),
                self.energies[i] / 100.0,
                (closest_friend_dist / 200.0).min(1.0),
                wall_l, wall_c, wall_r,
                hearing_vol.min(1.0), 
                in_mud                
            ];
            
            let outputs = self.brains[i].process(&inputs);
            let turn_force = outputs[0] * TURN_SPEED; 
            let mut speed = (outputs[1] + 1.0) * AGENT_SPEED_MODIFIER; 
            self.voices[i] = outputs[2].max(0.0);

            // PHYSICS
            if in_mud > 0.0 { speed *= 0.3; }

            self.angles[i] += turn_force;
            let vx = self.angles[i].cos() * speed;
            let vy = self.angles[i].sin() * speed;
            let new_x = my_x + vx;
            let new_y = my_y + vy;

            let mut hit_rock = false;
            for (rx, ry, rr) in &self.rocks {
                if (new_x - rx).hypot(new_y - ry) < *rr { hit_rock = true; break; }
            }

            if !hit_rock {
                self.positions[i] = (new_x, new_y);
            }

            if self.positions[i].0 < 0.0 { self.positions[i].0 = 0.0; }
            if self.positions[i].0 > self.width { self.positions[i].0 = self.width; }
            if self.positions[i].1 < 0.0 { self.positions[i].1 = 0.0; }
            if self.positions[i].1 > self.height { self.positions[i].1 = self.height; }

            // ENERGY COST
            let mut cost = speed * MOVE_COST;
            if in_mud > 0.0 { cost *= 3.0; } 
            cost += self.voices[i] * 0.1;   
            self.energies[i] -= cost;

            // FOOD
            if closest_food_dist < EAT_RADIUS {
                 self.energies[i] += FOOD_ENERGY; 
                 if self.energies[i] > ENERGY_CAP { self.energies[i] = ENERGY_CAP; } 
                 self.food[closest_food_index] = (Math::random() * self.width, Math::random() * self.height);
            }

            // PREDATOR
            if closest_pred_dist < PREDATOR_KILL_RADIUS {
                if self.energies[i] > WARRIOR_THRESHOLD {
                    self.predators[closest_pred_index] = (Math::random() * self.width, Math::random() * self.height);
                    self.energies[i] -= BATTLE_COST;
                } else {
                    self.energies[i] = -10.0; 
                }
            }

            // EVOLUTION (SEXUAL)
            if self.energies[i] <= 0.0 {
                let mut p1_idx = 0; let mut max_e1 = -1.0;
                for _ in 0..5 {
                    let r = (Math::random() * total_agents as f64) as usize;
                    if r != i && self.energies[r] > max_e1 { max_e1 = self.energies[r]; p1_idx = r; }
                }
                
                let mut p2_idx = 0; let mut max_e2 = -1.0;
                for _ in 0..5 {
                    let r = (Math::random() * total_agents as f64) as usize;
                    if r != i && r != p1_idx && self.energies[r] > max_e2 { max_e2 = self.energies[r]; p2_idx = r; }
                }

                if max_e1 > self.reproduction_threshold && max_e2 > self.reproduction_threshold { 
                    let mut new_brain = self.brains[p1_idx].crossover(&self.brains[p2_idx]);
                    new_brain = new_brain.mutate(self.mutation_rate);
                    self.brains[i] = new_brain;
                    
                    self.colors[i] = self.colors[p1_idx].clone(); 
                    let (px, py) = self.positions[p1_idx];
                    self.positions[i] = (px + (Math::random()-0.5)*10.0, py + (Math::random()-0.5)*10.0);
                    self.energies[i] = 60.0; 
                    
                    self.energies[p1_idx] -= 20.0; 
                    self.energies[p2_idx] -= 20.0; 
                } else {
                    self.brains[i] = Brain::new();
                    self.positions[i] = (Math::random() * self.width, Math::random() * self.height);
                    self.energies[i] = 100.0;
                    self.voices[i] = 0.0;
                }
            }
        }
    }

    // DRAW FUNCTION
    pub fn draw(&self, context: &web_sys::CanvasRenderingContext2d) {
        context.set_fill_style(&JsValue::from_str("#111"));
        context.fill_rect(0.0, 0.0, self.width, self.height);

        context.save();
        context.scale(self.zoom, self.zoom).unwrap();
        context.translate(-self.view_x, -self.view_y).unwrap();

        context.set_stroke_style(&JsValue::from_str("#222"));
        context.set_line_width(5.0);
        context.stroke_rect(0.0, 0.0, self.width, self.height);

        // Terrain
        context.set_fill_style(&JsValue::from_str("#1a2b3c")); // Mud
        for (mx, my, mr) in &self.mud {
            context.begin_path(); context.arc(*mx, *my, *mr, 0.0, 6.28).unwrap(); context.fill();
        }
        context.set_fill_style(&JsValue::from_str("#555")); // Rocks
        for (rx, ry, rr) in &self.rocks {
            context.begin_path(); context.arc(*rx, *ry, *rr, 0.0, 6.28).unwrap(); context.fill();
        }

        // Food
        context.set_fill_style(&JsValue::from_str("#00ff00"));
        for (fx, fy) in &self.food {
            context.begin_path(); context.arc(*fx, *fy, 3.0, 0.0, 6.28).unwrap(); context.fill();
        }

        // Predators
        context.set_fill_style(&JsValue::from_str("#ff0000"));
        for (px, py) in &self.predators {
            context.begin_path(); context.move_to(*px, *py - 10.0); context.line_to(*px + 10.0, *py + 10.0); context.line_to(*px - 10.0, *py + 10.0); context.fill();
        }

        // Agents
        for i in 0..self.positions.len() {
            let (x, y) = self.positions[i];
            context.set_fill_style(&JsValue::from_str(&self.colors[i]));
            context.set_global_alpha(self.energies[i] / 100.0);
            
            context.save();
            context.translate(x, y).unwrap();
            context.rotate(self.angles[i]).unwrap();
            
            context.begin_path(); context.move_to(6.0, 0.0); context.line_to(-4.0, 4.0); context.line_to(-4.0, -4.0); context.fill();

            // Warrior Visual
            if self.energies[i] > WARRIOR_THRESHOLD {
                context.set_stroke_style(&JsValue::from_str("#ffffff"));
                context.set_line_width(2.0);
                context.stroke();
            }
            context.restore();

            // Voice Visual
            if self.voices[i] > 0.5 {
                context.set_stroke_style(&JsValue::from_str("rgba(255, 255, 255, 0.4)"));
                context.set_line_width(1.0);
                context.begin_path();
                context.arc(x, y, 15.0 + (self.voices[i] * 10.0), 0.0, 6.28).unwrap();
                context.stroke();
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