use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Math;

// --- THE BRAIN (Neural Network) ---
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

        // Initialize with random values between -1.0 and 1.0
        for _ in 0..18 { weights_input.push((Math::random() * 2.0) - 1.0); }
        for _ in 0..12 { weights_output.push((Math::random() * 2.0) - 1.0); }
        for _ in 0..8  { biases.push((Math::random() * 2.0) - 1.0); }

        Brain { weights_input, weights_output, biases }
    }

    // FEATURE: Evolution with Dynamic Mutation Rate (from Block 2)
    fn mutate(&self, rate: f64) -> Brain {
        let mutation_chance = 0.2;

        let mutate_vec = |vals: &Vec<f64>| -> Vec<f64> {
            vals.iter().map(|&v| {
                if Math::random() < mutation_chance {
                    v + (Math::random() * 2.0 - 1.0) * rate // Dynamic rate applied here
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
        // Hidden Layer
        let mut hidden = vec![0.0; 6];
        for i in 0..6 {
            let mut sum = 0.0;
            for j in 0..3 { sum += inputs[j] * self.weights_input[i * 3 + j]; }
            sum += self.biases[i];
            hidden[i] = sum.tanh();
        }

        // Output Layer
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
// #[wasm_bindgen] allows JS to create/interact with this struct directly
#[wasm_bindgen]
pub struct Simulation {
    positions: Vec<(f64, f64)>,
    angles: Vec<f64>,
    energies: Vec<f64>,
    brains: Vec<Brain>,
    colors: Vec<String>, // Using String for JS compatibility
    
    food: Vec<(f64, f64)>,
    width: f64,
    height: f64,

    // FEATURE: Store the mutation rate from the slider
    mutation_rate: f64,
}

#[wasm_bindgen]
impl Simulation {
    // Constructor
    pub fn new(width: f64, height: f64) -> Simulation {
        let agent_count = 800; 
        let food_count = 80;

        let mut positions = Vec::new();
        let mut angles = Vec::new();
        let mut energies = Vec::new();
        let mut brains = Vec::new();
        let mut colors = Vec::new();
        let mut food = Vec::new();

        let color_palette = ["#ff00cc", "#ccff00", "#00ccff", "#ffcc00"];

        // Spawn Agents
        for _ in 0..agent_count {
            positions.push((Math::random() * width, Math::random() * height));
            angles.push(Math::random() * 6.28);
            energies.push(100.0);
            brains.push(Brain::new());
            
            let color_idx = (Math::random() * 4.0) as usize;
            colors.push(color_palette[color_idx].to_string());
        }

        // Spawn Food
        for _ in 0..food_count {
            food.push((Math::random() * width, Math::random() * height));
        }

        Simulation { 
            positions, angles, energies, brains, colors, food, width, height,
            mutation_rate: 0.1 // Default mutation rate
        }
    }

    // FEATURE: Allow JS to change mutation rate via slider
    pub fn set_mutation_rate(&mut self, rate: f64) {
        self.mutation_rate = rate;
    }

    // FEATURE: Allow JS to read average energy for stats
    pub fn get_avg_energy(&self) -> f64 {
        if self.energies.is_empty() { return 0.0; }
        let sum: f64 = self.energies.iter().sum();
        sum / self.energies.len() as f64
    }

    // Logic Step (Renamed from update to step for clarity in JS)
    pub fn step(&mut self) {
        let eat_radius = 10.0; 
        let total_agents = self.positions.len();

        for i in 0..total_agents {
            let (my_x, my_y) = self.positions[i];

            // 1. SENSORS (With Index Tracking Fix)
            let mut closest_dist_sq = 999999.0;
            let mut closest_dx = 0.0;
            let mut closest_dy = 0.0;
            let mut closest_food_index = 0; 

            for (idx, (fx, fy)) in self.food.iter().enumerate() {
                let dx = fx - my_x;
                let dy = fy - my_y;
                let dist_sq = dx*dx + dy*dy;
                if dist_sq < closest_dist_sq {
                    closest_dist_sq = dist_sq;
                    closest_dx = dx;
                    closest_dy = dy;
                    closest_food_index = idx; // Save the index
                }
            }

            let input_dx = closest_dx / self.width;
            let input_dy = closest_dy / self.height;
            let input_energy = self.energies[i] / 100.0;

            // 2. BRAIN PROCESS
            let outputs = self.brains[i].process(&[input_dx, input_dy, input_energy]);
            let turn_force = outputs[0] * 0.2; 
            let speed = (outputs[1] + 1.0) * 1.5; 

            // 3. PHYSICS & MOVEMENT
            self.angles[i] += turn_force;
            let vx = self.angles[i].cos() * speed;
            let vy = self.angles[i].sin() * speed;

            let (mut x, mut y) = self.positions[i];
            x += vx;
            y += vy;

            // Screen Wrap
            if x < 0.0 { x = self.width; }
            if x > self.width { x = 0.0; }
            if y < 0.0 { y = self.height; }
            if y > self.height { y = 0.0; }
            self.positions[i] = (x, y);

            // 4. METABOLISM & EATING
            self.energies[i] -= speed * 0.2; 

            // EATING LOGIC
            if closest_dist_sq < eat_radius * eat_radius {
                 self.energies[i] += 40.0; 
                 if self.energies[i] > 200.0 { self.energies[i] = 200.0; } 
                 
                 // Respawn the exact food item we saw using index
                 self.food[closest_food_index] = (Math::random() * self.width, Math::random() * self.height);
            }

            // 5. EVOLUTION (Tournament Selection)
            if self.energies[i] <= 0.0 {
                // Agent died. Find a survivor to clone.
                let mut best_parent_idx = 0;
                let mut max_energy = -1.0;

                // Pick 5 random agents and find the best one
                for _ in 0..5 {
                    let r = (Math::random() * total_agents as f64) as usize;
                    if r != i && self.energies[r] > max_energy {
                        max_energy = self.energies[r];
                        best_parent_idx = r;
                    }
                }

                if max_energy > 60.0 {
                    // SUCCESS: Clone Parent Brain + Mutate using DYNAMIC RATE
                    self.brains[i] = self.brains[best_parent_idx].mutate(self.mutation_rate);
                    self.colors[i] = self.colors[best_parent_idx].clone(); // Inherit Tribe Color
                    
                    let (px, py) = self.positions[best_parent_idx];
                    self.positions[i] = (px + (Math::random()-0.5)*10.0, py + (Math::random()-0.5)*10.0);
                    
                    self.energies[i] = 60.0; 
                    self.energies[best_parent_idx] -= 20.0; // Parent pays energy cost
                } else {
                    // FAILURE: Random Respawn
                    self.brains[i] = Brain::new();
                    self.positions[i] = (Math::random() * self.width, Math::random() * self.height);
                    self.energies[i] = 100.0;
                }
            }
        }
    }

    // Drawing Logic (Exposed to be called from Rust loop or JS loop)
    pub fn draw(&self, context: &web_sys::CanvasRenderingContext2d) {
        // Background
        context.set_fill_style(&JsValue::from_str("#111"));
        context.fill_rect(0.0, 0.0, self.width, self.height);
        
        // Draw Food
        context.set_fill_style(&JsValue::from_str("#00ff00"));
        for (fx, fy) in &self.food {
            context.begin_path();
            context.arc(*fx, *fy, 3.0, 0.0, 6.28).unwrap();
            context.fill();
        }

        // Draw Agents
        for i in 0..self.positions.len() {
            let (x, y) = self.positions[i];
            let angle = self.angles[i];
            
            context.set_fill_style(&JsValue::from_str(&self.colors[i]));
            context.set_global_alpha(self.energies[i] / 100.0);
            
            context.save();
            context.translate(x, y).unwrap();
            context.rotate(angle).unwrap();
            
            // Triangle Shape
            context.begin_path();
            context.move_to(6.0, 0.0);   
            context.line_to(-4.0, 4.0);  
            context.line_to(-4.0, -4.0); 
            context.fill();
            
            context.restore();
        }
        context.set_global_alpha(1.0);
    }
}

// --- ENTRY POINT (Auto-starts the simulation) ---
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let width = window.inner_width()?.as_f64().unwrap();
    let height = window.inner_height()?.as_f64().unwrap();
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);
    
    let context = canvas.get_context("2d")?.unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    // Create World using the new Struct definition
    let simulation = Rc::new(RefCell::new(
        Simulation::new(width, height)
    ));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let sim_loop = simulation.clone();
    let ctx_loop = context.clone(); // Clone context for the closure

    // GAME LOOP
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // 1. Update Physics/Brain
        sim_loop.borrow_mut().step();

        // 2. Draw using the method on the struct
        sim_loop.borrow().draw(&ctx_loop);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}