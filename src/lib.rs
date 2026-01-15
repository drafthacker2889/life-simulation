use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Math; // Built-in JS Math for Wasm performance

// --- THE GOD STRUCT (ECS) ---
struct Simulation {
    // Entities (The Agents)
    positions: Vec<(f64, f64)>, 
    velocities: Vec<(f64, f64)>,
    colors: Vec<&'static str>, 
    energies: Vec<f64>, // Life force (Battery level)
    
    // Environment
    food: Vec<(f64, f64)>, // Food positions
    width: f64,
    height: f64,
}

impl Simulation {
    // Initialize the simulation with Agents and Food
    fn new(agent_count: usize, food_count: usize, width: f64, height: f64) -> Simulation {
        let mut positions = Vec::with_capacity(agent_count);
        let mut velocities = Vec::with_capacity(agent_count);
        let mut colors = Vec::with_capacity(agent_count);
        let mut energies = Vec::with_capacity(agent_count);
        let mut food = Vec::with_capacity(food_count);

        let color_palette = ["#ff00cc", "#ccff00", "#00ccff", "#ffcc00"];

        // 1. Spawn Agents
        for _ in 0..agent_count {
            // Random Position
            positions.push((
                Math::random() * width, 
                Math::random() * height
            ));
            
            // Random Velocity (Speed between -2.0 and 2.0)
            velocities.push((
                (Math::random() - 0.5) * 4.0, 
                (Math::random() - 0.5) * 4.0
            ));
            
            // Random color from palette
            let color_idx = (Math::random() * 4.0) as usize;
            colors.push(color_palette[color_idx]);
            
            // Start with full battery (100.0 energy)
            energies.push(100.0);
        }

        // 2. Spawn Food
        for _ in 0..food_count {
            food.push((
                Math::random() * width, 
                Math::random() * height
            ));
        }

        Simulation { positions, velocities, colors, energies, food, width, height }
    }

    // --- SYSTEM: PHYSICS, METABOLISM & EATING ---
    fn update(&mut self) {
        // A. Move Agents & Burn Energy
        for i in 0..self.positions.len() {
            // 1. Movement Logic
            let (x, y) = self.positions[i];
            let (vx, vy) = self.velocities[i];
            let mut next_x = x + vx;
            let mut next_y = y + vy;

            // Bounce off walls (Simple Physics)
            if next_x < 0.0 || next_x > self.width {
                self.velocities[i].0 *= -1.0; // Reverse X
                next_x = x;
            }
            if next_y < 0.0 || next_y > self.height {
                self.velocities[i].1 *= -1.0; // Reverse Y
                next_y = y;
            }
            self.positions[i] = (next_x, next_y);

            // 2. Metabolism (Burn Energy)
            // Moving costs energy!
            self.energies[i] -= 0.25; 

            // 3. Death & Respawn
            // If energy hits 0, they "die". We respawn them to keep the sim running.
            if self.energies[i] <= 0.0 {
                self.positions[i] = (
                    Math::random() * self.width, 
                    Math::random() * self.height
                );
                self.energies[i] = 100.0; // Reset battery
            }
        }

        // B. Eating System (Collision Detection)
        // Check every agent against every piece of food
        let eat_radius = 10.0; // How close they need to be to eat
        
        for i in 0..self.positions.len() {
            let (ax, ay) = self.positions[i];

            for j in 0..self.food.len() {
                let (fx, fy) = self.food[j];
                
                // Distance Check (Squared distance is faster than Sqrt)
                let dx = ax - fx;
                let dy = ay - fy;
                let dist_sq = dx*dx + dy*dy;

                if dist_sq < eat_radius * eat_radius {
                    // YUM! Eat the food.
                    self.energies[i] += 40.0; // Gain energy
                    
                    // Cap energy so they don't live forever without eating
                    if self.energies[i] > 150.0 { 
                        self.energies[i] = 150.0; 
                    } 

                    // Respawn food elsewhere immediately
                    self.food[j] = (
                        Math::random() * self.width, 
                        Math::random() * self.height
                    );
                }
            }
        }
    }
}

// --- THE MAIN ENTRY POINT ---
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    // --- RESIZE CANVAS TO FULL SCREEN ---
    // We grab the window size and set the canvas to match for maximum effect
    let width = window.inner_width()?.as_f64().unwrap();
    let height = window.inner_height()?.as_f64().unwrap();
    
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);
    
    let context = canvas.get_context("2d")?.unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    // Create World: 800 Agents, 60 Food Items
    let simulation = Rc::new(RefCell::new(
        Simulation::new(800, 60, width, height)
    ));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    
    // We clone the pointer to simulation so the Loop can access it
    let sim_loop = simulation.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // --- STEP 1: UPDATE PHYSICS ---
        sim_loop.borrow_mut().update();

        // --- STEP 2: RENDER ---
        // We borrow() because we are only READING to draw
        let sim = sim_loop.borrow();
        
        // 1. Clear Screen (Black background for contrast)
        context.set_fill_style(&JsValue::from_str("#111111"));
        context.fill_rect(0.0, 0.0, sim.width, sim.height);
        
        // 2. Draw Food (Green dots)
        context.set_fill_style(&JsValue::from_str("#00ff00"));
        for (fx, fy) in &sim.food {
            context.begin_path();
            context.arc(*fx, *fy, 3.0, 0.0, 6.28).unwrap(); // Draw circle
            context.fill();
        }

        // 3. Draw Agents
        for i in 0..sim.positions.len() {
            let (x, y) = sim.positions[i];
            
            // Visual Flair: Fade color if energy is low (Life Indicator)
            let alpha = sim.energies[i] / 100.0;
            
            // We use simple Rects for speed
            context.set_fill_style(&JsValue::from_str(sim.colors[i]));
            context.set_global_alpha(alpha); // Transparency based on energy
            
            context.fill_rect(x, y, 5.0, 5.0);
        }
        context.set_global_alpha(1.0); // Reset alpha so background isn't transparent next frame

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