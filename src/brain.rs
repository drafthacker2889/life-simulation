use js_sys::Math;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Brain {
    pub weights_input: Vec<f64>,  
    pub weights_output: Vec<f64>, 
    pub biases: Vec<f64>,
    pub last_inputs: Vec<f64>,
    pub last_hidden: Vec<f64>,
    pub last_outputs: Vec<f64>,
}

impl Brain {
    pub fn new() -> Brain {
        let mut weights_input = Vec::new();
        let mut weights_output = Vec::new();
        let mut biases = Vec::new();

        // CHANGED: 13 Inputs (Added Cosine for Food/Pred) * 8 Hidden
        for _ in 0..(13 * 8) { weights_input.push((Math::random() * 2.0) - 1.0); } 
        // 8 Hidden * 3 Outputs
        for _ in 0..(8 * 3) { weights_output.push((Math::random() * 2.0) - 1.0); } 
        // 8 Hidden + 3 Outputs
        for _ in 0..(8 + 3) { biases.push((Math::random() * 2.0) - 1.0); }        

        Brain { 
            weights_input, weights_output, biases,
            last_inputs: vec![0.0; 13], // Resized buffer
            last_hidden: vec![0.0; 8],
            last_outputs: vec![0.0; 3],
        }
    }

    pub fn crossover(&self, partner: &Brain) -> Brain {
        let mix = |a: &Vec<f64>, b: &Vec<f64>| -> Vec<f64> {
            a.iter().zip(b.iter()).map(|(&w1, &w2)| {
                if Math::random() > 0.5 { w1 } else { w2 }
            }).collect()
        };

        let mut child = Brain::new();
        child.weights_input = mix(&self.weights_input, &partner.weights_input);
        child.weights_output = mix(&self.weights_output, &partner.weights_output);
        child.biases = mix(&self.biases, &partner.biases);
        child
    }

    pub fn mutate(&self, rate: f64) -> Brain {
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
        
        let mut child = self.clone(); 
        child.weights_input = mutate_vec(&self.weights_input);
        child.weights_output = mutate_vec(&self.weights_output);
        child.biases = mutate_vec(&self.biases);
        child
    }

    pub fn process(&mut self, inputs: &[f64]) -> Vec<f64> {
        self.last_inputs = inputs.to_vec();

        let mut hidden = vec![0.0; 8];
        for i in 0..8 {
            let mut sum = 0.0;
            // CHANGED: Loop 13 times
            for j in 0..13 { sum += inputs[j] * self.weights_input[i * 13 + j]; }
            sum += self.biases[i];
            hidden[i] = sum.tanh();
        }
        self.last_hidden = hidden.clone();

        let mut outputs = vec![0.0; 3];
        for i in 0..3 {
            let mut sum = 0.0;
            for j in 0..8 { sum += hidden[j] * self.weights_output[i * 8 + j]; }
            sum += self.biases[8 + i];
            outputs[i] = sum.tanh();
        }
        self.last_outputs = outputs.clone();

        outputs
    }
}