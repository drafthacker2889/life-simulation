// World Settings
pub const AGENT_COUNT: usize = 600;
pub const FOOD_COUNT: usize = 120;
pub const PREDATOR_COUNT: usize = 5;
pub const POISON_COUNT: usize = 15;

// Physics
pub const AGENT_SPEED_MODIFIER: f64 = 1.5;
pub const TURN_SPEED: f64 = 0.2;

// Energy / Metabolism
pub const STARTING_ENERGY: f64 = 100.0;
pub const FOOD_ENERGY: f64 = 50.0;
pub const POISON_DAMAGE: f64 = 60.0;
pub const ENERGY_CAP: f64 = 200.0;
pub const MOVE_COST: f64 = 0.08;
pub const WARRIOR_THRESHOLD: f64 = 150.0;
pub const BATTLE_COST: f64 = 50.0;

// Radiuses
pub const EAT_RADIUS: f64 = 10.0;
pub const PREDATOR_KILL_RADIUS: f64 = 15.0;
pub const WHISKER_LEN: f64 = 50.0;
pub const SOCIAL_RADIUS: f64 = 120.0;
pub const SHARE_RADIUS: f64 = 25.0;

// Evolution
pub const BASE_MUTATION_RATE: f64 = 0.1;

// Seasons
pub const SEASON_LENGTH: u64 = 1200;
pub const WINTER_FOOD_MULT: f64 = 0.3;
pub const SUMMER_FOOD_MULT: f64 = 1.5;

// Intrinsic Reward
pub const CURIOSITY_WEIGHT: f64 = 0.3;
pub const NOVELTY_WEIGHT: f64 = 0.2;

// Social
pub const FOOD_SHARE_AMOUNT: f64 = 10.0;
pub const COOPERATION_BONUS: f64 = 5.0;
pub const BETRAYAL_PENALTY: f64 = 15.0;
pub const DECEPTION_COST: f64 = 2.0;

// Biomes
pub const BIOME_CELL_SIZE: f64 = 200.0;
pub const BIOME_PLAINS: u8 = 0;
pub const BIOME_FOREST: u8 = 1;
pub const BIOME_DESERT: u8 = 2;
pub const BIOME_SWAMP: u8 = 3;

// Curriculum
pub const CURRICULUM_EASY_END: u64 = 5000;
pub const CURRICULUM_MEDIUM_END: u64 = 15000;

// Online Learning
pub const ONLINE_LEARNING_RATE: f64 = 0.001;
pub const LEARNING_INTERVAL: u64 = 5;

// Metrics
pub const METRICS_INTERVAL: u64 = 100;
pub const METRICS_MAX_POINTS: usize = 500;

// Shelter
pub const SHELTER_BUILD_COST: f64 = 25.0;
pub const SHELTER_RADIUS: f64 = 35.0;
pub const SHELTER_MAX: usize = 30;
pub const SHELTER_DURABILITY: u32 = 3000;
pub const SHELTER_ROCK_RANGE: f64 = 50.0;
pub const SHELTER_PREDATOR_REPEL: f64 = 60.0;

// Proactive Mating
pub const MATING_ENERGY_COST: f64 = 15.0;
pub const MATING_RADIUS: f64 = 35.0;
pub const MATING_COOLDOWN: u64 = 200;

// Predator Life Cycle
pub const PREDATOR_MAX_AGE: u64 = 20000;
pub const PREDATOR_ENERGY_START: f64 = 150.0;
pub const PREDATOR_ENERGY_CAP: f64 = 300.0;
pub const PREDATOR_KILL_ENERGY: f64 = 80.0;
pub const PREDATOR_MOVE_COST: f64 = 0.08;
pub const PREDATOR_MATE_COOLDOWN: u64 = 2000;
pub const PREDATOR_MATE_COST: f64 = 60.0;
pub const PREDATOR_MATE_THRESHOLD: f64 = 120.0;
pub const PREDATOR_MATE_RADIUS: f64 = 40.0;
pub const PREDATOR_MAX_COUNT: usize = 20;
pub const PREDATOR_OLD_AGE_THRESHOLD: u64 = 15000;

// Food Growth
pub const FOOD_GROW_RADIUS: f64 = 80.0;
pub const FOOD_GROW_CHANCE: f64 = 0.003;
pub const FOOD_MAX: usize = 400;
