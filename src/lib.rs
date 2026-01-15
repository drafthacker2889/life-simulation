use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// This function runs AUTOMATICALLY when the Wasm loads
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Enable better error messages in the browser console
    console_error_panic_hook::set_once();

    // 1. Get the Window and Document (The "DOM")
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    
    // 2. Get the Canvas
    let canvas = document.get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
        
    // 3. Get the Context (The "Pen" we draw with)
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    // --- GAME STATE ---
    // We wrap variables in Rc<RefCell<...>> so they can survive inside the loop closure
    let x_pos = Rc::new(RefCell::new(0.0));
    let y_pos = Rc::new(RefCell::new(300.0));

    // --- THE LOOP ---
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    // Create the closure (function) that will run 60 times a second
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // A. UPDATE LOGIC (Rust logic)
        let mut x = x_pos.borrow_mut();
        *x += 2.0; // Move right by 2 pixels
        if *x > 800.0 { *x = 0.0; } // Reset if off screen

        // B. RENDER (Rust drawing directly to Canvas)
        context.clear_rect(0.0, 0.0, 800.0, 600.0);
        
        context.set_fill_style(&JsValue::from_str("#00ffcc"));
        context.fill_rect(*x, *y_pos.borrow(), 50.0, 50.0);

        // C. REQUEST NEXT FRAME
        request_animation_frame(f.borrow().as_ref().unwrap());
        
    }) as Box<dyn FnMut()>));

    // Kick off the loop!
    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

// Helper function to call the browser's requestAnimationFrame
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}