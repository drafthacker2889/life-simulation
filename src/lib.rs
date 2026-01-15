use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Math; // We use JS Math because it's built-in and fast for Wasm

// --- THE GOD STRUCT (ECS) ---
struct Simulation {
    // Parallel Arrays: index 'i' in all vectors refers to the same entity
    positions: Vec<(f64, f64)>, 
    velocities: Vec<(f64, f64)>,
    colors: Vec<&'static str>, 
    width: f64,
    height: f64,
}

impl Simulation {
    // Initialize 1,000 random particles
    fn new(count: usize, width: f64, height: f64) -> Simulation {
        let mut positions = Vec::with_capacity(count);
        let mut velocities = Vec::with_capacity(count);
        let mut colors = Vec::with_capacity(count);

        let color_palette = ["#00ffcc", "#ff00cc", "#ccff00", "#00ccff"];

        for _ in 0..count {
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

            // Random Color
            let color_idx = (Math::random() * 4.0) as usize;
            colors.push(color_palette[color_idx]);
        }

        Simulation { positions, velocities, colors, width, height }
    }

    // --- SYSTEM: MOVEMENT ---
    // Updates position based on velocity and handles wall bouncing
    fn update(&mut self) {
        for i in 0..self.positions.len() {
            let (x, y) = self.positions[i];
            let (vx, vy) = self.velocities[i];

            let mut next_x = x + vx;
            let mut next_y = y + vy;

            // Bounce off walls (Simple Physics)
            if next_x < 0.0 || next_x > self.width {
                self.velocities[i].0 *= -1.0; // Reverse X velocity
                next_x = x; // Reset position to avoid sticking
            }
            if next_y < 0.0 || next_y > self.height {
                self.velocities[i].1 *= -1.0; // Reverse Y velocity
                next_y = y;
            }

            self.positions[i] = (next_x, next_y);
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
    
    // Set canvas full screen for maximum effect
    canvas.set_width(1000);
    canvas.set_height(800);
    
    let context = canvas.get_context("2d")?.unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    // 1. Create the Simulation World (1000 entities)
    let simulation = Rc::new(RefCell::new(
        Simulation::new(1000, 800.0, 600.0)
    ));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    
    // We clone the pointer to simulation so the Loop can access it
    let sim_loop = simulation.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // --- STEP 1: UPDATE PHYSICS ---
        // We borrow_mut() because we are CHANGING positions
        sim_loop.borrow_mut().update();

        // --- STEP 2: RENDER ---
        context.clear_rect(0.0, 0.0, 800.0, 600.0);
        
        // We borrow() because we are only READING positions to draw
        let sim = sim_loop.borrow(); 
        
        for i in 0..sim.positions.len() {
            let (x, y) = sim.positions[i];
            
            // Optimization: Small Rects are faster than Arcs
            context.set_fill_style(&JsValue::from_str(sim.colors[i]));
            context.fill_rect(x, y, 4.0, 4.0); 
        }

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