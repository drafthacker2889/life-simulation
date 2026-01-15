use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Math;

// --- THE BRAIN (No changes) ---
#[derive(Clone)]
struct Brain {
    weights_input: Vec<f64>,
    weights_output: Vec<f64>,
    biases: Vec<f64>,
}

impl Brain {
    fn new() -> Brain {
        let mut weights_input = Vec::new();
        let mut weights_output = Vec::new();
        let mut biases = Vec::new();
        for _ in 0..30 { weights_input.push((Math::random() * 2.0) - 1.0); } 
        for _ in 0..12 { weights_output.push((Math::random() * 2.0) - 1.0); } 
        for _ in 0..8  { biases.push((Math::random() * 2.0) - 1.0); }        
        Brain { weights_input, weights_output, biases }
    }

    fn mutate(&self, rate: f64) -> Brain {
        let mutation_chance = 0.2; 
        let mutate_vec = |vals: &Vec<f64>| -> Vec<f64> {
            vals.iter().map(|&v| {
                if Math::random() < mutation_chance {
                    v + (Math::random() * 2.0 - 1.0) * rate 
                } else {
                    v
                }
            }).collect()
        };
        Brain {
            weights_input: mutate_vec(&self.weights_input),
            weights_output: mutate_vec(&self.weights_output),
            biases: mutate_vec(&self.biases),
        }
    }

    fn process(&self, inputs: &[f64]) -> Vec<f64> {
        let mut hidden = vec![0.0; 6];
        for i in 0..6 {
            let mut sum = 0.0;
            for j in 0..5 { sum += inputs[j] * self.weights_input[i * 5 + j]; }
            sum += self.biases[i];
            hidden[i] = sum.tanh();
        }
        let mut outputs = vec![0.0; 2];
        for i in 0..2 {
            let mut sum = 0.0;
            for j in 0..6 { sum += hidden[j] * self.weights_output[i * 6 + j]; }
            sum += self.biases[6 + i];
            outputs[i] = sum.tanh();
        }
        outputs
    }
}

// --- THE SIMULATION WORLD ---
#[wasm_bindgen]
pub struct Simulation {
    positions: Vec<(f64, f64)>, 
    angles: Vec<f64>,
    energies: Vec<f64>,
    brains: Vec<Brain>,
    colors: Vec<String>, 
    
    food: Vec<(f64, f64)>, 
    predators: Vec<(f64, f64)>,

    width: f64,
    height: f64,
    
    // CONFIGURABLE PARAMETERS
    mutation_rate: f64,
    predator_speed: f64,       // NEW
    reproduction_threshold: f64, // NEW

    // CAMERA
    view_x: f64,
    view_y: f64,
    zoom: f64,
}

#[wasm_bindgen]
impl Simulation {
    pub fn new(width: f64, height: f64) -> Simulation {
        let agent_count = 800;
        let food_count = 80;
        let predator_count = 5; 

        let mut positions = Vec::new();
        let mut angles = Vec::new();
        let mut energies = Vec::new();
        let mut brains = Vec::new();
        let mut colors = Vec::new();
        let mut food = Vec::new();
        let mut predators = Vec::new();

        let color_palette = ["#ff00cc", "#ccff00", "#00ccff", "#ffcc00"];

        for _ in 0..agent_count {
            positions.push((Math::random() * width, Math::random() * height));
            angles.push(Math::random() * 6.28);
            energies.push(100.0);
            brains.push(Brain::new());
            let color_idx = (Math::random() * 4.0) as usize;
            colors.push(color_palette[color_idx].to_string());
        }

        for _ in 0..food_count {
            food.push((Math::random() * width, Math::random() * height));
        }

        for _ in 0..predator_count {
            predators.push((Math::random() * width, Math::random() * height));
        }

        Simulation { 
            positions, angles, energies, brains, colors, food, predators, 
            width, height, 
            mutation_rate: 0.1,
            predator_speed: 2.5,       // Default Speed
            reproduction_threshold: 60.0, // Default Difficulty
            view_x: 0.0, view_y: 0.0, zoom: 1.0,
        }
    }

    // --- CONTROLS ---
    pub fn set_mutation_rate(&mut self, rate: f64) { self.mutation_rate = rate; }
    
    // NEW: Slider 1 - Predator Speed
    pub fn set_predator_speed(&mut self, speed: f64) { self.predator_speed = speed; }
    
    // NEW: Slider 2 - Reproduction Difficulty
    pub fn set_reproduction_threshold(&mut self, val: f64) { self.reproduction_threshold = val; }

    // NEW: Slider 3 - Food Count (Adds or Removes food dynamically)
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

    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
    }

    pub fn get_avg_energy(&self) -> f64 {
        if self.energies.is_empty() { return 0.0; }
        let sum: f64 = self.energies.iter().sum();
        sum / self.energies.len() as f64
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.view_x += dx / self.zoom;
        self.view_y += dy / self.zoom;
    }

    pub fn zoom_at(&mut self, factor: f64) {
        self.zoom *= factor;
        if self.zoom < 0.1 { self.zoom = 0.1; }
        if self.zoom > 5.0 { self.zoom = 5.0; }
    }

    pub fn step(&mut self) {
        let eat_radius = 10.0; 
        let pred_kill_radius = 15.0; 
        let total_agents = self.positions.len();

        // Predators
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

            let speed = self.predator_speed; // USE VARIABLE
            let dx = target_x - px;
            let dy = target_y - py;
            let dist = dx.hypot(dy);
            
            if dist > 0.0 {
                self.predators[i].0 += (dx / dist) * speed;
                self.predators[i].1 += (dy / dist) * speed;
            }
        }

        // Agents
        for i in 0..total_agents {
            let (my_x, my_y) = self.positions[i];

            let mut closest_food_dist_sq = 999999.0;
            let mut closest_food_dx = 0.0;
            let mut closest_food_dy = 0.0;
            let mut closest_food_index = 0; 

            for (idx, (fx, fy)) in self.food.iter().enumerate() {
                let dx = fx - my_x;
                let dy = fy - my_y;
                let dist_sq = dx*dx + dy*dy;
                if dist_sq < closest_food_dist_sq {
                    closest_food_dist_sq = dist_sq;
                    closest_food_dx = dx;
                    closest_food_dy = dy;
                    closest_food_index = idx;
                }
            }

            let mut closest_pred_dist_sq = 999999.0;
            let mut closest_pred_dx = 0.0;
            let mut closest_pred_dy = 0.0;

            for (px, py) in &self.predators {
                let dx = px - my_x;
                let dy = py - my_y;
                let dist_sq = dx*dx + dy*dy;
                if dist_sq < closest_pred_dist_sq {
                    closest_pred_dist_sq = dist_sq;
                    closest_pred_dx = dx;
                    closest_pred_dy = dy;
                }
            }

            let in_food_dx = closest_food_dx / self.width;
            let in_food_dy = closest_food_dy / self.height;
            let in_pred_dx = closest_pred_dx / self.width;
            let in_pred_dy = closest_pred_dy / self.height;
            let in_energy = self.energies[i] / 100.0;

            let outputs = self.brains[i].process(&[in_food_dx, in_food_dy, in_energy, in_pred_dx, in_pred_dy]);
            let turn_force = outputs[0] * 0.2; 
            let speed = (outputs[1] + 1.0) * 1.5; 

            self.angles[i] += turn_force;
            let vx = self.angles[i].cos() * speed;
            let vy = self.angles[i].sin() * speed;

            let (mut x, mut y) = self.positions[i];
            x += vx; y += vy;

            if x < 0.0 { x = 0.0; self.angles[i] += 3.14; }
            if x > self.width { x = self.width; self.angles[i] += 3.14; }
            if y < 0.0 { y = 0.0; self.angles[i] += 3.14; }
            if y > self.height { y = self.height; self.angles[i] += 3.14; }
            self.positions[i] = (x, y);

            self.energies[i] -= speed * 0.2; 

            if closest_food_dist_sq < eat_radius * eat_radius {
                 self.energies[i] += 40.0; 
                 if self.energies[i] > 200.0 { self.energies[i] = 200.0; } 
                 self.food[closest_food_index] = (Math::random() * self.width, Math::random() * self.height);
            }

            if closest_pred_dist_sq < pred_kill_radius * pred_kill_radius {
                self.energies[i] = -10.0; 
            }

            if self.energies[i] <= 0.0 {
                let mut best_parent_idx = 0;
                let mut max_energy = -1.0;

                for _ in 0..5 {
                    let r = (Math::random() * total_agents as f64) as usize;
                    if r != i && self.energies[r] > max_energy {
                        max_energy = self.energies[r];
                        best_parent_idx = r;
                    }
                }

                if max_energy > self.reproduction_threshold { // USE VARIABLE
                    let new_brain = self.brains[best_parent_idx].mutate(self.mutation_rate);
                    self.brains[i] = new_brain;
                    let new_color = self.colors[best_parent_idx].clone();
                    self.colors[i] = new_color;
                    let (px, py) = self.positions[best_parent_idx];
                    self.positions[i] = (px + (Math::random()-0.5)*10.0, py + (Math::random()-0.5)*10.0);
                    self.energies[i] = 60.0; 
                    self.energies[best_parent_idx] -= 20.0; 
                } else {
                    self.brains[i] = Brain::new();
                    self.positions[i] = (Math::random() * self.width, Math::random() * self.height);
                    self.energies[i] = 100.0;
                }
            }
        }
    }

    pub fn draw(&self, context: &web_sys::CanvasRenderingContext2d) {
        context.set_fill_style(&JsValue::from_str("#111"));
        context.fill_rect(0.0, 0.0, self.width, self.height);

        context.save();
        context.scale(self.zoom, self.zoom).unwrap();
        context.translate(-self.view_x, -self.view_y).unwrap();

        context.set_stroke_style(&JsValue::from_str("#222"));
        context.set_line_width(5.0);
        context.stroke_rect(0.0, 0.0, self.width, self.height);

        context.set_fill_style(&JsValue::from_str("#00ff00"));
        for (fx, fy) in &self.food {
            context.begin_path();
            context.arc(*fx, *fy, 3.0, 0.0, 6.28).unwrap();
            context.fill();
        }

        context.set_fill_style(&JsValue::from_str("#ff0000"));
        for (px, py) in &self.predators {
            context.begin_path();
            context.move_to(*px, *py - 10.0);
            context.line_to(*px + 8.0, *py + 8.0);
            context.line_to(*px - 8.0, *py + 8.0);
            context.fill();
        }

        for i in 0..self.positions.len() {
            let (x, y) = self.positions[i];
            let angle = self.angles[i];
            
            context.set_fill_style(&JsValue::from_str(&self.colors[i]));
            context.set_global_alpha(self.energies[i] / 100.0);
            
            context.save();
            context.translate(x, y).unwrap();
            context.rotate(angle).unwrap();
            context.begin_path();
            context.move_to(6.0, 0.0);    
            context.line_to(-4.0, 4.0);   
            context.line_to(-4.0, -4.0); 
            context.fill();
            context.restore();
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