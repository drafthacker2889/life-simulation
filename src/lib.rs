use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Math;

// --- THE BRAIN ---
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

        // 9 INPUTS: 
        // (FoodDist, FoodAngle, PredDist, PredAngle, Energy, FriendDist, WallL, WallC, WallR)
        // 9 inputs * 6 hidden = 54 weights
        for _ in 0..54 { weights_input.push((Math::random() * 2.0) - 1.0); } 
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
            for j in 0..9 { sum += inputs[j] * self.weights_input[i * 9 + j]; }
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
    
    mutation_rate: f64,
    predator_speed: f64,       
    reproduction_threshold: f64, 

    view_x: f64, view_y: f64, zoom: f64,
}

#[wasm_bindgen]
impl Simulation {
    pub fn new(width: f64, height: f64) -> Simulation {
        let agent_count = 800;
        let food_count = 100;
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
            predator_speed: 2.2,
            reproduction_threshold: 60.0, 
            view_x: 0.0, view_y: 0.0, zoom: 1.0,
        }
    }

    // --- ADDED FROM BLOCK 2: STATS FUNCTION ---
    // Returns array of 4 integers: [PinkCount, GreenCount, BlueCount, OrangeCount]
    pub fn get_tribe_stats(&self) -> Box<[i32]> {
        let mut stats = vec![0, 0, 0, 0];
        for color in &self.colors {
            match color.as_str() {
                "#ff00cc" => stats[0] += 1, // Pink
                "#ccff00" => stats[1] += 1, // Green
                "#00ccff" => stats[2] += 1, // Blue
                "#ffcc00" => stats[3] += 1, // Orange
                _ => {},
            }
        }
        stats.into_boxed_slice()
    }

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

        // 1. UPDATE PREDATORS (With Separation & Wall Clamps)
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
            
            if dist > 0.0 {
                dx = (dx / dist) * speed;
                dy = (dy / dist) * speed;
            }

            // SEPARATION (Prevent Stacking) - Used Block 2's distance (30.0)
            for k in 0..self.predators.len() {
                if i == k { continue; }
                let (ox, oy) = self.predators[k];
                let sep_dist = (px - ox).hypot(py - oy);
                if sep_dist < 30.0 && sep_dist > 0.0 {
                    let push_x = (px - ox) / sep_dist;
                    let push_y = (py - oy) / sep_dist;
                    dx += push_x * 0.8; // Increased push force slightly
                    dy += push_y * 0.8;
                }
            }

            self.predators[i].0 += dx;
            self.predators[i].1 += dy;

            // WALL CLAMP
            if self.predators[i].0 < 0.0 { self.predators[i].0 = 0.0; }
            if self.predators[i].0 > self.width { self.predators[i].0 = self.width; }
            if self.predators[i].1 < 0.0 { self.predators[i].1 = 0.0; }
            if self.predators[i].1 > self.height { self.predators[i].1 = self.height; }
        }

        // 2. UPDATE AGENTS
        for i in 0..total_agents {
            let (my_x, my_y) = self.positions[i];
            let my_angle = self.angles[i];

            // --- SENSORS ---
            
            // FOOD
            let mut closest_food_dist = 9999.0;
            let mut food_angle_diff = 0.0;
            let mut closest_food_index = 0; 

            for (idx, (fx, fy)) in self.food.iter().enumerate() {
                let dx = fx - my_x;
                let dy = fy - my_y;
                let dist = dx.hypot(dy);
                if dist < closest_food_dist {
                    closest_food_dist = dist;
                    closest_food_index = idx;
                    let angle_to_food = dy.atan2(dx);
                    food_angle_diff = angle_to_food - my_angle;
                }
            }

            // PREDATOR
            let mut closest_pred_dist = 9999.0;
            let mut pred_angle_diff = 0.0;
            let mut closest_pred_index = 0; 

            for (idx, (px, py)) in self.predators.iter().enumerate() {
                let dx = px - my_x;
                let dy = py - my_y;
                let dist = dx.hypot(dy);
                if dist < closest_pred_dist {
                    closest_pred_dist = dist;
                    closest_pred_index = idx; 
                    let angle_to_pred = dy.atan2(dx);
                    pred_angle_diff = angle_to_pred - my_angle;
                }
            }

            // SOCIAL
            let mut closest_friend_dist = 9999.0;
            for j in 0..total_agents {
                if i == j { continue; } 
                let (fx, fy) = self.positions[j];
                let dist = (fx - my_x).hypot(fy - my_y);
                if dist < closest_friend_dist {
                    closest_friend_dist = dist;
                }
            }

            // WHISKERS
            let ray_len = 50.0;
            let check_wall = |angle_offset: f64| -> f64 {
                let angle = my_angle + angle_offset;
                let rx = my_x + angle.cos() * ray_len;
                let ry = my_y + angle.sin() * ray_len;
                if rx < 0.0 || rx > self.width || ry < 0.0 || ry > self.height { 1.0 } else { 0.0 }
            };
            let wall_l = check_wall(-0.78); 
            let wall_c = check_wall(0.0);
            let wall_r = check_wall(0.78); 

            // INPUTS
            let in_food_dist = (closest_food_dist / self.width).min(1.0);
            let in_food_angle = food_angle_diff.sin(); 
            let in_pred_dist = (closest_pred_dist / self.width).min(1.0);
            let in_pred_angle = pred_angle_diff.sin();
            let in_energy = self.energies[i] / 100.0;
            let in_friend_dist = (closest_friend_dist / 200.0).min(1.0);

            let inputs = [
                in_food_dist, in_food_angle, 
                in_pred_dist, in_pred_angle, 
                in_energy, 
                in_friend_dist, 
                wall_l, wall_c, wall_r 
            ];
            
            let outputs = self.brains[i].process(&inputs);
            let turn_force = outputs[0] * 0.2; 
            let speed = (outputs[1] + 1.0) * 1.5; 

            // PHYSICS
            self.angles[i] += turn_force;
            let vx = self.angles[i].cos() * speed;
            let vy = self.angles[i].sin() * speed;

            let (mut x, mut y) = self.positions[i];
            x += vx; y += vy;

            if x < 0.0 { x = 0.0; } if x > self.width { x = self.width; }
            if y < 0.0 { y = 0.0; } if y > self.height { y = self.height; }
            self.positions[i] = (x, y);

            self.energies[i] -= speed * 0.2; 

            // INTERACTION: Food
            if closest_food_dist < eat_radius {
                 self.energies[i] += 40.0; 
                 if self.energies[i] > 200.0 { self.energies[i] = 200.0; } 
                 self.food[closest_food_index] = (Math::random() * self.width, Math::random() * self.height);
            }

            // INTERACTION: Predator (Combat Logic PRESERVED from Block 1)
            if closest_pred_dist < pred_kill_radius {
                if self.energies[i] > 150.0 {
                    // WARRIOR MODE: Kill the predator
                    // Respawn the predator elsewhere
                    self.predators[closest_pred_index] = (Math::random() * self.width, Math::random() * self.height);
                    // Agent pays energy cost
                    self.energies[i] -= 50.0;
                } else {
                    // WEAK: Get eaten
                    self.energies[i] = -10.0; 
                }
            }

            // EVOLUTION
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

                if max_energy > self.reproduction_threshold { 
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

        // Food
        context.set_fill_style(&JsValue::from_str("#00ff00"));
        for (fx, fy) in &self.food {
            context.begin_path();
            context.arc(*fx, *fy, 3.0, 0.0, 6.28).unwrap();
            context.fill();
        }

        // Predators (Using Block 2 Size: +/- 10.0)
        context.set_fill_style(&JsValue::from_str("#ff0000"));
        for (px, py) in &self.predators {
            context.begin_path();
            context.move_to(*px, *py - 10.0);
            context.line_to(*px + 10.0, *py + 10.0);
            context.line_to(*px - 10.0, *py + 10.0);
            context.fill();
        }

        // Agents
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

            // HERO VISUAL: White border for Warriors (PRESERVED from Block 1)
            if self.energies[i] > 150.0 {
                context.set_stroke_style(&JsValue::from_str("#ffffff"));
                context.set_line_width(2.0);
                context.stroke();
            }

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