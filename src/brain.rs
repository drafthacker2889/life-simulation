use js_sys::Math;
use serde::{Serialize, Deserialize};

// ============================================================
//  CONSTANTS FOR BRAIN ARCHITECTURE
// ============================================================
pub const RAW_INPUT_SIZE: usize = 22;
pub const ENCODED_SIZE: usize = 12;
pub const MEMORY_SIZE: usize = 16;
pub const CORE_INPUT: usize = 28;
pub const CORE_HIDDEN: usize = 24;
pub const POLICY_OUTPUTS: usize = 7;
pub const VALUE_OUTPUTS: usize = 1;
pub const WORLD_MODEL_OUT: usize = 12;
pub const NUM_DRIVES: usize = 5;

// ============================================================
//  NEUROCHEMISTRY
// ============================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct Neurochemistry {
    pub dopamine: f64,
    pub cortisol: f64,
    pub oxytocin: f64,
    pub serotonin: f64,
}

impl Neurochemistry {
    pub fn new() -> Self {
        Neurochemistry { dopamine: 0.5, cortisol: 0.2, oxytocin: 0.3, serotonin: 0.5 }
    }
    pub fn update(&mut self, reward: f64, threat: f64, social: f64) {
        self.dopamine  = (self.dopamine * 0.85 + reward * 0.15).clamp(0.0, 1.0);
        // Cortisol decays faster (0.75) so agents recover from fear quickly
        self.cortisol  = (self.cortisol * 0.75 + threat * 0.25).clamp(0.0, 1.0);
        self.oxytocin  = (self.oxytocin * 0.88 + social * 0.12).clamp(0.0, 1.0);
        // Serotonin (happiness) rises faster when not stressed, and from food rewards
        self.serotonin = (self.serotonin * 0.9 + (1.0 - self.cortisol) * 0.08 + reward * 0.02).clamp(0.0, 1.0);
    }
    pub fn as_slice(&self) -> [f64; 4] {
        [self.dopamine, self.cortisol, self.oxytocin, self.serotonin]
    }
}

// ============================================================
//  MEMORY SYSTEM — short-term (GRU) + long-term traces
// ============================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct MemorySystem {
    pub hidden_state: Vec<f64>,
    pub long_term: Vec<f64>,
    pub last_reward: f64,
    pub lifetime_reward: f64,
}

impl MemorySystem {
    pub fn new() -> Self {
        MemorySystem {
            hidden_state: vec![0.0; MEMORY_SIZE],
            long_term: vec![0.0; 8],
            last_reward: 0.0,
            lifetime_reward: 0.0,
        }
    }
    pub fn record_event(&mut self, quadrant: usize, is_food: bool, value: f64) {
        let idx = quadrant * 2 + if is_food { 0 } else { 1 };
        if idx < self.long_term.len() {
            self.long_term[idx] = (self.long_term[idx] * 0.95 + value * 0.05).clamp(-1.0, 1.0);
        }
    }
    pub fn get_quadrant(x: f64, y: f64, w: f64, h: f64) -> usize {
        let qx = if x < w * 0.5 { 0 } else { 1 };
        let qy = if y < h * 0.5 { 0 } else { 1 };
        qy * 2 + qx
    }
}

// ============================================================
//  EPISODIC MEMORY — stores top-K significant life events
// ============================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct EpisodicEvent {
    pub x: f64,
    pub y: f64,
    pub reward: f64,
    pub event_type: u8, // 0=food, 1=poison, 2=predator, 3=social
    pub tick: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub events: Vec<EpisodicEvent>,
    pub max_events: usize,
}

impl EpisodicMemory {
    pub fn new() -> Self {
        EpisodicMemory { events: Vec::new(), max_events: 8 }
    }
    pub fn record(&mut self, x: f64, y: f64, reward: f64, event_type: u8, tick: u64) {
        if reward.abs() < 0.3 { return; }
        self.events.push(EpisodicEvent { x, y, reward, event_type, tick });
        if self.events.len() > self.max_events {
            self.events.sort_by(|a, b| b.reward.abs().partial_cmp(&a.reward.abs()).unwrap());
            self.events.truncate(self.max_events);
        }
    }
    pub fn recall_nearby(&self, x: f64, y: f64, radius: f64) -> f64 {
        let mut score = 0.0;
        for e in &self.events {
            let dist = (e.x - x).hypot(e.y - y);
            if dist < radius {
                score += e.reward * (1.0 - dist / radius);
            }
        }
        score.clamp(-1.0, 1.0)
    }
}

// ============================================================
//  BEHAVIOR VECTOR — for species divergence tracking
// ============================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct BehaviorVector {
    pub avg_speed: f64,
    pub avg_turn: f64,
    pub avg_voice: f64,
    pub food_efficiency: f64,
    pub social_score: f64,
    pub exploration_score: f64,
    pub samples: u32,
}

impl BehaviorVector {
    pub fn new() -> Self {
        BehaviorVector {
            avg_speed: 0.0, avg_turn: 0.0, avg_voice: 0.0,
            food_efficiency: 0.0, social_score: 0.0, exploration_score: 0.0, samples: 0,
        }
    }
    pub fn update(&mut self, speed: f64, turn: f64, voice: f64) {
        self.samples += 1;
        let alpha = 0.01;
        self.avg_speed = self.avg_speed * (1.0 - alpha) + speed.abs() * alpha;
        self.avg_turn = self.avg_turn * (1.0 - alpha) + turn.abs() * alpha;
        self.avg_voice = self.avg_voice * (1.0 - alpha) + voice * alpha;
    }
    pub fn distance(&self, other: &BehaviorVector) -> f64 {
        let ds = (self.avg_speed - other.avg_speed).powi(2);
        let dt = (self.avg_turn - other.avg_turn).powi(2);
        let dv = (self.avg_voice - other.avg_voice).powi(2);
        let df = (self.food_efficiency - other.food_efficiency).powi(2);
        let dsc = (self.social_score - other.social_score).powi(2);
        let de = (self.exploration_score - other.exploration_score).powi(2);
        (ds + dt + dv + df + dsc + de).sqrt()
    }
}

// ============================================================
//  GOAL ARBITRATION SYSTEM
// ============================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct GoalSystem {
    pub drive_weights: [f64; NUM_DRIVES],
    pub base_weights: [f64; NUM_DRIVES],
}

impl GoalSystem {
    pub fn new() -> Self {
        let base = [
            0.3 + Math::random() * 0.4,
            0.3 + Math::random() * 0.4,
            0.1 + Math::random() * 0.3,
            0.1 + Math::random() * 0.3,
            0.2 + Math::random() * 0.4,
        ];
        GoalSystem { drive_weights: base, base_weights: base }
    }
    pub fn update(&mut self, energy: f64, threat: f64, chem: &Neurochemistry) {
        let hunger_urgency = (1.0 - energy / 200.0).clamp(0.0, 1.0);
        let fear_urgency = threat;
        let repro_urgency = (energy / 200.0).clamp(0.0, 1.0) * (1.0 - chem.cortisol);
        let social_urgency = chem.oxytocin;
        let curiosity_urgency = chem.dopamine * (1.0 - chem.cortisol);
        let raw = [hunger_urgency, fear_urgency, repro_urgency, social_urgency, curiosity_urgency];
        let mut total = 0.0;
        for i in 0..NUM_DRIVES {
            self.drive_weights[i] = self.base_weights[i] * raw[i];
            total += self.drive_weights[i];
        }
        if total > 0.0 {
            for i in 0..NUM_DRIVES { self.drive_weights[i] /= total; }
        }
    }
    pub fn dominant_drive(&self) -> usize {
        let mut best = 0;
        for i in 1..NUM_DRIVES {
            if self.drive_weights[i] > self.drive_weights[best] { best = i; }
        }
        best
    }
}

// ============================================================
//  CURIOSITY MODULE
// ============================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct CuriosityModule {
    pub prediction_error: f64,
    pub novelty_bonus: f64,
    pub visited_cells: Vec<u8>,
    pub visit_cols: usize,
    pub visit_rows: usize,
}

impl CuriosityModule {
    pub fn new(w: f64, h: f64) -> Self {
        let cols = (w / 200.0).ceil().max(1.0) as usize;
        let rows = (h / 200.0).ceil().max(1.0) as usize;
        CuriosityModule {
            prediction_error: 0.0,
            novelty_bonus: 0.0,
            visited_cells: vec![0; cols * rows],
            visit_cols: cols,
            visit_rows: rows,
        }
    }
    pub fn visit(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let c = ((x / w) * self.visit_cols as f64).floor().clamp(0.0, (self.visit_cols - 1) as f64) as usize;
        let r = ((y / h) * self.visit_rows as f64).floor().clamp(0.0, (self.visit_rows - 1) as f64) as usize;
        let idx = r * self.visit_cols + c;
        if idx < self.visited_cells.len() {
            let count = self.visited_cells[idx];
            self.novelty_bonus = if count < 5 { (5 - count) as f64 / 5.0 } else { 0.0 };
            if count < 255 { self.visited_cells[idx] = count + 1; }
        }
    }
}

// ============================================================
//  SOCIAL SYSTEM
// ============================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct SocialSystem {
    pub signal_out: f64,
    pub signal_type: u8,
    pub trust_score: f64,
    pub reputation: f64,
    pub cooperation_count: u32,
    pub deception_tendency: f64,
    pub betrayal_count: u32,
}

impl SocialSystem {
    pub fn new() -> Self {
        SocialSystem {
            signal_out: 0.0, signal_type: 0, trust_score: 0.5,
            reputation: 0.5, cooperation_count: 0,
            deception_tendency: Math::random() * 0.3, betrayal_count: 0,
        }
    }
    pub fn update_reputation(&mut self, positive: bool) {
        if positive {
            self.reputation = (self.reputation + 0.02).min(1.0);
            self.trust_score = (self.trust_score + 0.01).min(1.0);
            self.cooperation_count += 1;
        } else {
            self.reputation = (self.reputation - 0.05).max(0.0);
            self.trust_score = (self.trust_score - 0.02).max(0.0);
        }
    }
}

// ============================================================
//  MAIN BRAIN — modular architecture
// ============================================================
#[derive(Clone, Serialize, Deserialize)]
pub struct Brain {
    // Perception Encoder: raw(22) -> encoded(12)
    pub w_enc: Vec<f64>,
    pub b_enc: Vec<f64>,
    // GRU-lite gates: input = encoded(12) + prev_hidden(16) = 28
    pub w_gate_z: Vec<f64>,
    pub w_gate_r: Vec<f64>,
    pub w_gate_h: Vec<f64>,
    pub b_gate_z: Vec<f64>,
    pub b_gate_r: Vec<f64>,
    pub b_gate_h: Vec<f64>,
    // Recurrent Core: memory(16) -> core_hidden(24)
    pub w_core: Vec<f64>,
    pub b_core: Vec<f64>,
    // Policy Head: core_hidden(24) -> policy(7)
    pub w_policy: Vec<f64>,
    pub b_policy: Vec<f64>,
    // Value Head: core_hidden(24) -> value(1)
    pub w_value: Vec<f64>,
    pub b_value: Vec<f64>,
    // World Model: (core_hidden + policy) -> predicted_obs(12)
    pub w_world: Vec<f64>,
    pub b_world: Vec<f64>,

    // Subsystems
    pub memory: MemorySystem,
    pub neurochemistry: Neurochemistry,
    pub goals: GoalSystem,
    pub curiosity: CuriosityModule,
    pub social: SocialSystem,
    pub episodic: EpisodicMemory,
    pub behavior: BehaviorVector,

    // Diagnostics
    pub last_inputs: Vec<f64>,
    pub last_encoded: Vec<f64>,
    pub last_hidden: Vec<f64>,
    pub last_outputs: Vec<f64>,
    pub last_value: f64,
    pub last_world_pred: Vec<f64>,
    pub last_drive: usize,

    pub age: u64,
    pub generation: u32,
    pub lineage_id: u32,
    pub last_mate_tick: u64,
}

fn rand_vec(n: usize, scale: f64) -> Vec<f64> {
    (0..n).map(|_| (Math::random() * 2.0 - 1.0) * scale).collect()
}

fn sigmoid(x: f64) -> f64 { 1.0 / (1.0 + (-x).exp()) }

impl Brain {
    pub fn new() -> Brain {
        let init = 0.5;
        Brain {
            w_enc: rand_vec(RAW_INPUT_SIZE * ENCODED_SIZE, init),
            b_enc: vec![0.0; ENCODED_SIZE],
            w_gate_z: rand_vec(CORE_INPUT * MEMORY_SIZE, init),
            w_gate_r: rand_vec(CORE_INPUT * MEMORY_SIZE, init),
            w_gate_h: rand_vec(CORE_INPUT * MEMORY_SIZE, init),
            b_gate_z: vec![0.0; MEMORY_SIZE],
            b_gate_r: vec![0.0; MEMORY_SIZE],
            b_gate_h: vec![0.0; MEMORY_SIZE],
            w_core: rand_vec(MEMORY_SIZE * CORE_HIDDEN, init),
            b_core: vec![0.0; CORE_HIDDEN],
            w_policy: rand_vec(CORE_HIDDEN * POLICY_OUTPUTS, init),
            b_policy: vec![0.0; POLICY_OUTPUTS],
            w_value: rand_vec(CORE_HIDDEN * VALUE_OUTPUTS, init),
            b_value: vec![0.0; VALUE_OUTPUTS],
            w_world: rand_vec((CORE_HIDDEN + POLICY_OUTPUTS) * WORLD_MODEL_OUT, init),
            b_world: vec![0.0; WORLD_MODEL_OUT],
            memory: MemorySystem::new(),
            neurochemistry: Neurochemistry::new(),
            goals: GoalSystem::new(),
            curiosity: CuriosityModule::new(2000.0, 2000.0),
            social: SocialSystem::new(),
            episodic: EpisodicMemory::new(),
            behavior: BehaviorVector::new(),
            last_inputs: vec![0.0; RAW_INPUT_SIZE],
            last_encoded: vec![0.0; ENCODED_SIZE],
            last_hidden: vec![0.0; CORE_HIDDEN],
            last_outputs: vec![0.0; POLICY_OUTPUTS],
            last_value: 0.0,
            last_world_pred: vec![0.0; WORLD_MODEL_OUT],
            last_drive: 0,
            age: 0,
            generation: 0,
            lineage_id: (Math::random() * 1_000_000.0) as u32,
            last_mate_tick: 0,
        }
    }

    // -------------------------------------------------------
    //  CROSSOVER
    // -------------------------------------------------------
    pub fn crossover(&self, partner: &Brain) -> Brain {
        let mix = |a: &[f64], b: &[f64]| -> Vec<f64> {
            a.iter().zip(b.iter()).map(|(&w1, &w2)| {
                if Math::random() > 0.5 { w1 } else { w2 }
            }).collect()
        };
        let mix_goals = |a: &[f64; NUM_DRIVES], b: &[f64; NUM_DRIVES]| -> [f64; NUM_DRIVES] {
            let mut out = [0.0; NUM_DRIVES];
            for i in 0..NUM_DRIVES {
                out[i] = if Math::random() > 0.5 { a[i] } else { b[i] };
            }
            out
        };

        let mut child = Brain::new();
        child.w_enc = mix(&self.w_enc, &partner.w_enc);
        child.b_enc = mix(&self.b_enc, &partner.b_enc);
        child.w_gate_z = mix(&self.w_gate_z, &partner.w_gate_z);
        child.w_gate_r = mix(&self.w_gate_r, &partner.w_gate_r);
        child.w_gate_h = mix(&self.w_gate_h, &partner.w_gate_h);
        child.b_gate_z = mix(&self.b_gate_z, &partner.b_gate_z);
        child.b_gate_r = mix(&self.b_gate_r, &partner.b_gate_r);
        child.b_gate_h = mix(&self.b_gate_h, &partner.b_gate_h);
        child.w_core = mix(&self.w_core, &partner.w_core);
        child.b_core = mix(&self.b_core, &partner.b_core);
        child.w_policy = mix(&self.w_policy, &partner.w_policy);
        child.b_policy = mix(&self.b_policy, &partner.b_policy);
        child.w_value = mix(&self.w_value, &partner.w_value);
        child.b_value = mix(&self.b_value, &partner.b_value);
        child.w_world = mix(&self.w_world, &partner.w_world);
        child.b_world = mix(&self.b_world, &partner.b_world);
        child.goals.base_weights = mix_goals(&self.goals.base_weights, &partner.goals.base_weights);
        // Inherit social traits
        child.social.deception_tendency = if Math::random() > 0.5 {
            self.social.deception_tendency
        } else {
            partner.social.deception_tendency
        };
        child.generation = self.generation.max(partner.generation) + 1;
        child.lineage_id = if Math::random() > 0.5 { self.lineage_id } else { partner.lineage_id };
        child.last_mate_tick = 0;
        child
    }

    // -------------------------------------------------------
    //  MUTATION
    // -------------------------------------------------------
    pub fn mutate(&self, rate: f64) -> Brain {
        let chance = 0.2;
        let mutate_vec = |vals: &[f64]| -> Vec<f64> {
            vals.iter().map(|&v| {
                if Math::random() < chance {
                    (v + (Math::random() * 2.0 - 1.0) * rate).clamp(-3.0, 3.0)
                } else { v }
            }).collect()
        };
        let mutate_arr = |vals: &[f64; NUM_DRIVES]| -> [f64; NUM_DRIVES] {
            let mut out = *vals;
            for i in 0..NUM_DRIVES {
                if Math::random() < chance {
                    out[i] = (out[i] + (Math::random() * 2.0 - 1.0) * rate * 0.3).clamp(0.05, 1.0);
                }
            }
            out
        };

        let mut child = self.clone();
        child.w_enc = mutate_vec(&self.w_enc);
        child.b_enc = mutate_vec(&self.b_enc);
        child.w_gate_z = mutate_vec(&self.w_gate_z);
        child.w_gate_r = mutate_vec(&self.w_gate_r);
        child.w_gate_h = mutate_vec(&self.w_gate_h);
        child.b_gate_z = mutate_vec(&self.b_gate_z);
        child.b_gate_r = mutate_vec(&self.b_gate_r);
        child.b_gate_h = mutate_vec(&self.b_gate_h);
        child.w_core = mutate_vec(&self.w_core);
        child.b_core = mutate_vec(&self.b_core);
        child.w_policy = mutate_vec(&self.w_policy);
        child.b_policy = mutate_vec(&self.b_policy);
        child.w_value = mutate_vec(&self.w_value);
        child.b_value = mutate_vec(&self.b_value);
        child.w_world = mutate_vec(&self.w_world);
        child.b_world = mutate_vec(&self.b_world);
        child.goals.base_weights = mutate_arr(&self.goals.base_weights);
        // Mutate deception tendency
        if Math::random() < chance {
            child.social.deception_tendency =
                (child.social.deception_tendency + (Math::random() * 2.0 - 1.0) * rate * 0.2).clamp(0.0, 1.0);
        }
        // Reset runtime state for newborn
        child.memory = MemorySystem::new();
        child.neurochemistry = Neurochemistry::new();
        child.episodic = EpisodicMemory::new();
        child.behavior = BehaviorVector::new();
        child.age = 0;
        child.last_mate_tick = 0;
        child
    }

    // -------------------------------------------------------
    //  FORWARD PASS — the full thinking pipeline
    // -------------------------------------------------------
    pub fn process(&mut self, raw_inputs: &[f64]) -> Vec<f64> {
        self.last_inputs = raw_inputs.to_vec();
        self.age += 1;

        // === 1. Perception Encoder: raw(22) -> encoded(12) ===
        let mut encoded = vec![0.0; ENCODED_SIZE];
        for i in 0..ENCODED_SIZE {
            let mut sum = self.b_enc[i];
            for j in 0..RAW_INPUT_SIZE {
                sum += raw_inputs[j] * self.w_enc[i * RAW_INPUT_SIZE + j];
            }
            encoded[i] = sum.tanh();
        }
        self.last_encoded = encoded.clone();

        // === 2. GRU-lite Memory Update ===
        let prev_h = &self.memory.hidden_state;
        let mut gate_input: Vec<f64> = encoded.clone();
        gate_input.extend_from_slice(prev_h);

        let mut z = vec![0.0; MEMORY_SIZE];
        let mut r = vec![0.0; MEMORY_SIZE];
        let mut h_candidate = vec![0.0; MEMORY_SIZE];

        for i in 0..MEMORY_SIZE {
            let mut sz = self.b_gate_z[i];
            let mut sr = self.b_gate_r[i];
            for j in 0..CORE_INPUT {
                sz += gate_input[j] * self.w_gate_z[i * CORE_INPUT + j];
                sr += gate_input[j] * self.w_gate_r[i * CORE_INPUT + j];
            }
            z[i] = sigmoid(sz);
            r[i] = sigmoid(sr);
        }

        let mut reset_input: Vec<f64> = encoded.clone();
        for i in 0..MEMORY_SIZE {
            reset_input.push(r[i] * prev_h[i]);
        }
        for i in 0..MEMORY_SIZE {
            let mut s = self.b_gate_h[i];
            for j in 0..CORE_INPUT {
                s += reset_input[j] * self.w_gate_h[i * CORE_INPUT + j];
            }
            h_candidate[i] = s.tanh();
        }

        let mut new_hidden = vec![0.0; MEMORY_SIZE];
        for i in 0..MEMORY_SIZE {
            new_hidden[i] = (1.0 - z[i]) * prev_h[i] + z[i] * h_candidate[i];
        }
        self.memory.hidden_state = new_hidden.clone();

        // === 3. Recurrent Core: memory(16) -> core_hidden(24) ===
        let mut core_hidden = vec![0.0; CORE_HIDDEN];
        for i in 0..CORE_HIDDEN {
            let mut sum = self.b_core[i];
            for j in 0..MEMORY_SIZE {
                sum += new_hidden[j] * self.w_core[i * MEMORY_SIZE + j];
            }
            core_hidden[i] = sum.tanh();
        }
        self.last_hidden = core_hidden.clone();

        // === 4. Policy Head: core_hidden(24) -> actions(7) ===
        let mut policy = vec![0.0; POLICY_OUTPUTS];
        for i in 0..POLICY_OUTPUTS {
            let mut sum = self.b_policy[i];
            for j in 0..CORE_HIDDEN {
                sum += core_hidden[j] * self.w_policy[i * CORE_HIDDEN + j];
            }
            policy[i] = sum.tanh();
        }

        // Stochastic policy: noise scaled by dopamine
        let noise_scale = 0.05 * self.neurochemistry.dopamine;
        for i in 0..POLICY_OUTPUTS {
            policy[i] = (policy[i] + (Math::random() * 2.0 - 1.0) * noise_scale).clamp(-1.0, 1.0);
        }
        self.last_outputs = policy.clone();

        // === 5. Value Head ===
        let mut val = self.b_value[0];
        for j in 0..CORE_HIDDEN {
            val += core_hidden[j] * self.w_value[j];
        }
        self.last_value = val.tanh();

        // === 6. World Model ===
        let wm_in_size = CORE_HIDDEN + POLICY_OUTPUTS;
        let mut wm_input: Vec<f64> = core_hidden.clone();
        wm_input.extend_from_slice(&policy);
        let mut world_pred = vec![0.0; WORLD_MODEL_OUT];
        for i in 0..WORLD_MODEL_OUT {
            let mut sum = self.b_world[i];
            for j in 0..wm_in_size {
                sum += wm_input[j] * self.w_world[i * wm_in_size + j];
            }
            world_pred[i] = sum.tanh();
        }
        self.last_world_pred = world_pred.clone();

        // === 7. Intrinsic motivation: prediction error ===
        let mut pred_err = 0.0;
        for i in 0..WORLD_MODEL_OUT.min(ENCODED_SIZE) {
            let diff = world_pred[i] - encoded[i];
            pred_err += diff * diff;
        }
        pred_err = (pred_err / WORLD_MODEL_OUT as f64).sqrt();
        self.curiosity.prediction_error = pred_err;

        // === 8. Social signal decision ===
        self.social.signal_out = (policy[2].abs() + 0.1).min(1.0);
        if policy[4] > 0.5 { self.social.signal_type = 2; }      // danger
        else if policy[3] > 0.3 { self.social.signal_type = 1; }  // food_here
        else if policy[5] > 0.3 { self.social.signal_type = 3; }  // mate_call
        else { self.social.signal_type = 0; }

        self.last_drive = self.goals.dominant_drive();
        policy
    }

    // -------------------------------------------------------
    //  ONLINE LEARNING — REINFORCE-style policy gradient
    // -------------------------------------------------------
    pub fn learn_from_reward(&mut self, reward: f64, learning_rate: f64) {
        if reward.abs() < 0.01 { return; }
        let lr = learning_rate * reward.clamp(-1.0, 1.0);
        for i in 0..POLICY_OUTPUTS {
            for j in 0..CORE_HIDDEN {
                let idx = i * CORE_HIDDEN + j;
                if idx < self.w_policy.len() {
                    let delta = lr * self.last_hidden[j] * self.last_outputs[i];
                    self.w_policy[idx] = (self.w_policy[idx] + delta).clamp(-3.0, 3.0);
                }
            }
            if i < self.b_policy.len() {
                self.b_policy[i] = (self.b_policy[i] + lr * self.last_outputs[i] * 0.1).clamp(-3.0, 3.0);
            }
        }
    }

    pub fn get_intrinsic_reward(&self) -> f64 {
        self.curiosity.prediction_error * 0.3 + self.curiosity.novelty_bonus * 0.2
    }
}
