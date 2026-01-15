use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Math;

// --- THE BRAIN (Neural Network) ---
// A simple Feed-Forward Network: 3 Inputs -> 6 Hidden Neurons -> 2 Outputs
#[derive(Clone)]
struct Brain {
    weights_input: Vec<f64>,  // Weights from Input to Hidden
    weights_output: Vec<f64>, // Weights from Hidden to Output
    biases: Vec<f64>,         // Biases for neurons
}

impl Brain {
    fn new() -> Brain {
        let mut weights_input = Vec::new();
        let mut weights_output = Vec::new();
        let mut biases = Vec::new();

        // Initialize with random values between -1.0 and 1.0
        for _ in 0..18 { weights_input.push((Math::random() * 2.0) - 1.0); } // 3 inputs * 6 hidden
        for _ in 0..12 { weights_output.push((Math::random() * 2.0) - 1.0); } // 6 hidden * 2 output
        for _ in 0..8  { biases.push((Math::random() * 2.0) - 1.0); }        // 6 hidden + 2 output

        Brain { weights_input, weights_output, biases }
    }

    // The "Thinking" Process
    // Inputs: [Food_DX, Food_DY, Current_Energy]
    // Returns: [Turn_Force, Speed_Force]
    fn process(&self, inputs: &[f64]) -> Vec<f64> {
        // 1. Hidden Layer Processing
        let mut hidden = vec![0.0; 6];
        for i in 0..6 {
            let mut sum = 0.0;
            for j in 0..3 {
                sum += inputs[j] * self.weights_input[i * 3 + j];
            }
            sum += self.biases[i];
            hidden[i] = sum.tanh(); // Activation function (-1 to 1)
        }

        // 2. Output Layer Processing
        let mut outputs = vec![0.0; 2];
        for i in 0..2 {
            let mut sum = 0.0;
            for j in 0..6 {
                sum += hidden[j] * self.weights_output[i * 6 + j];
            }
            sum += self.biases[6 + i];
            outputs[i] = sum.tanh();
        }
        outputs
    }
}

// --- THE SIMULATION WORLD ---
struct Simulation {
    positions: Vec<(f64, f64)>, 
    angles: Vec<f64>,      // Direction they are facing (radians)
    energies: Vec<f64>,    // Health/Battery
    brains: Vec<Brain>,    // The AI for each agent
    colors: Vec<&'static str>, 
    
    food: Vec<(f64, f64)>, 
    width: f64,
    height: f64,
}

impl Simulation {
    fn new(agent_count: usize, food_count: usize, width: f64, height: f64) -> Simulation {
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
            angles.push(Math::random() * 6.28); // Random direction (0 to 2*PI)
            energies.push(100.0); // Full battery
            brains.push(Brain::new()); // Random brain
            
            let color_idx = (Math::random() * 4.0) as usize;
            colors.push(color_palette[color_idx]);
        }

        // Spawn Food
        for _ in 0..food_count {
            food.push((Math::random() * width, Math::random() * height));
        }

        Simulation { positions, angles, energies, brains, colors, food, width, height }
    }

    fn update(&mut self) {
        let eat_radius = 10.0; 

        for i in 0..self.positions.len() {
            let (my_x, my_y) = self.positions[i];

            // --- 1. SENSORS (The Eye) ---
            // Find vector to nearest food
            let mut closest_dist_sq = 999999.0;
            let mut closest_dx = 0.0;
            let mut closest_dy = 0.0;

            for (fx, fy) in &self.food {
                let dx = fx - my_x;
                let dy = fy - my_y;
                let dist_sq = dx*dx + dy*dy;
                if dist_sq < closest_dist_sq {
                    closest_dist_sq = dist_sq;
                    closest_dx = dx;
                    closest_dy = dy;
                }
            }

            // Normalize inputs so the Brain can understand them (roughly -1.0 to 1.0)
            let input_dx = closest_dx / self.width;
            let input_dy = closest_dy / self.height;
            let input_energy = self.energies[i] / 100.0;

            // --- 2. BRAIN (The Logic) ---
            let outputs = self.brains[i].process(&[input_dx, input_dy, input_energy]);
            
            // Output 0: Turn Left/Right
            let turn_force = outputs[0] * 0.2; 
            // Output 1: Speed Up/Down (Max speed 3.0)
            let speed = (outputs[1] + 1.0) * 1.5; 

            // --- 3. PHYSICS (The Body) ---
            self.angles[i] += turn_force;

            let vx = self.angles[i].cos() * speed;
            let vy = self.angles[i].sin() * speed;

            let (mut x, mut y) = self.positions[i];
            x += vx;
            y += vy;

            // Screen Wrapping (Pacman style)
            if x < 0.0 { x = self.width; }
            if x > self.width { x = 0.0; }
            if y < 0.0 { y = self.height; }
            if y > self.height { y = 0.0; }
            self.positions[i] = (x, y);

            // --- 4. LIFE MECHANICS ---
            // Metabolism: Moving fast burns energy
            self.energies[i] -= speed * 0.2; 

            // Eating: If close to food, gain energy
            if closest_dist_sq < eat_radius * eat_radius {
                 self.energies[i] += 40.0; 
                 if self.energies[i] > 150.0 { self.energies[i] = 150.0; }
                 
                 // Respawn the eaten food
                 // (Finding the exact food index again to move it)
                 for f_idx in 0..self.food.len() {
                     let (fx, fy) = self.food[f_idx];
                     if (fx - my_x).abs() < 1.0 && (fy - my_y).abs() < 1.0 {
                         self.food[f_idx] = (Math::random() * self.width, Math::random() * self.height);
                         break;
                     }
                 }
            }

            // Death: If energy 0, die and respawn with NEW RANDOM BRAIN
            if self.energies[i] <= 0.0 {
                self.positions[i] = (Math::random() * self.width, Math::random() * self.height);
                self.energies[i] = 100.0;
                self.brains[i] = Brain::new(); // Try a new strategy
            }
        }
    }
}

// --- ENTRY POINT ---
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    
    // Setup Canvas
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

    // Create World: 800 Agents, 80 Food
    let simulation = Rc::new(RefCell::new(
        Simulation::new(800, 80, width, height)
    ));

    // Start Game Loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let sim_loop = simulation.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        sim_loop.borrow_mut().update();

        // Rendering
        let sim = sim_loop.borrow();
        
        // Background
        context.set_fill_style(&JsValue::from_str("#111"));
        context.fill_rect(0.0, 0.0, sim.width, sim.height);
        
        // Draw Food
        context.set_fill_style(&JsValue::from_str("#00ff00"));
        for (fx, fy) in &sim.food {
            context.begin_path();
            context.arc(*fx, *fy, 3.0, 0.0, 6.28).unwrap();
            context.fill();
        }

        // Draw Agents (Triangles to show direction)
        for i in 0..sim.positions.len() {
            let (x, y) = sim.positions[i];
            let angle = sim.angles[i];
            
            context.set_fill_style(&JsValue::from_str(sim.colors[i]));
            // Fade out as they starve
            context.set_global_alpha(sim.energies[i] / 100.0);
            
            context.save();
            context.translate(x, y).unwrap();
            context.rotate(angle).unwrap();
            
            context.begin_path();
            context.move_to(6.0, 0.0);   // Nose
            context.line_to(-4.0, 4.0);  // Back Left
            context.line_to(-4.0, -4.0); // Back Right
            context.fill();
            
            context.restore();
        }
        context.set_global_alpha(1.0);

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