// World Settings
pub const AGENT_COUNT: usize = 800;
pub const FOOD_COUNT: usize = 100;
pub const PREDATOR_COUNT: usize = 5;

// Physics
pub const AGENT_SPEED_MODIFIER: f64 = 1.5;
pub const TURN_SPEED: f64 = 0.2;

// Energy / Metabolism
pub const STARTING_ENERGY: f64 = 100.0;
pub const FOOD_ENERGY: f64 = 40.0;
pub const ENERGY_CAP: f64 = 200.0;
pub const MOVE_COST: f64 = 0.2;
pub const WARRIOR_THRESHOLD: f64 = 150.0;
pub const BATTLE_COST: f64 = 50.0;

// Radiuses
pub const EAT_RADIUS: f64 = 10.0;
pub const PREDATOR_KILL_RADIUS: f64 = 15.0;
pub const WHISKER_LEN: f64 = 50.0;

// Evolution
pub const BASE_MUTATION_RATE: f64 = 0.1;