//! Generator module.
//!
//! Procedurally generates logic puzzles at runtime based on configurable
//! parameters: difficulty, category, and seed.

use crate::difficulty::Difficulty;
use crate::puzzle::{Condition, Effect, Puzzle};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ── Category ─────────────────────────────────────────────────────────────────

/// Puzzle theme categories for procedural generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Category {
    /// Digital logic, circuits, switches, and relays.
    Logic,
    /// Cryptography, ciphers, and codes.
    Cryptography,
    /// Dungeons, traps, braziers, and ancient mechanisms.
    Dungeon,
    /// Alchemical recipes, crucibles, and reagents.
    Alchemy,
    /// Patterns, sequences, and frequency tuning.
    Pattern,
}

impl Category {
    /// Returns the string representation of this category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Logic => "Logic",
            Category::Cryptography => "Cryptography",
            Category::Dungeon => "Dungeon",
            Category::Alchemy => "Alchemy",
            Category::Pattern => "Pattern",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "logic" => Ok(Category::Logic),
            "cryptography" | "crypto" | "cipher" => Ok(Category::Cryptography),
            "dungeon" => Ok(Category::Dungeon),
            "alchemy" => Ok(Category::Alchemy),
            "pattern" => Ok(Category::Pattern),
            _ => Err(format!("Unknown puzzle category: '{s}'")),
        }
    }
}

impl From<&str> for Category {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or(Category::Logic)
    }
}

// ── Deterministic PRNG ────────────────────────────────────────────────────────

/// Simple pseudo-random number generator (PCG / Xorshift variant)
/// initialized with a seed to guarantee cross-platform determinism.
#[derive(Debug, Clone)]
pub struct Prng {
    state: u64,
}

impl Prng {
    /// Creates a new PRNG seeded with `seed`.
    pub fn new(seed: u64) -> Self {
        let mut s = seed.wrapping_add(0x9e3779b97f4a7c15);
        s = (s ^ (s >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        s = (s ^ (s >> 27)).wrapping_mul(0x94d049bb133111eb);
        Self {
            state: s ^ (s >> 31),
        }
    }

    /// Generates the next pseudo-random `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut x = self.state;
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58476d1ce4e5b9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94d049bb133111eb);
        x ^ (x >> 31)
    }

    /// Generates an integer in the inclusive range `[min, max]`.
    pub fn next_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        let span = (max - min + 1) as u64;
        min + (self.next_u64() % span) as usize
    }

    /// Randomly shuffles a slice in place.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.next_range(0, i);
            slice.swap(i, j);
        }
    }
}

// ── Difficulty Parameters ────────────────────────────────────────────────────

/// Scaling parameters determined by [`Difficulty`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DifficultyParams {
    /// Number of sequential steps required for the primary solution chain.
    pub chain_length: usize,
    /// Number of red-herring decoy conditions added to increase puzzle complexity.
    pub red_herrings: usize,
}

impl DifficultyParams {
    /// Returns the generation parameters for the given difficulty tier.
    pub fn for_difficulty(difficulty: Difficulty) -> Self {
        match difficulty {
            Difficulty::Easy => DifficultyParams {
                chain_length: 2,
                red_herrings: 1,
            },
            Difficulty::Medium => DifficultyParams {
                chain_length: 4,
                red_herrings: 2,
            },
            Difficulty::Hard => DifficultyParams {
                chain_length: 6,
                red_herrings: 4,
            },
        }
    }
}

// ── Category Step Pools ──────────────────────────────────────────────────────

fn step_pool_for_category(category: Category) -> Vec<(&'static str, &'static str)> {
    match category {
        Category::Logic => vec![
            ("inspect_circuit", "Inspect the primary logic circuit board"),
            ("toggle_switch_alpha", "Toggle the alpha input switch"),
            ("flip_relay_beta", "Flip the magnetic relay beta switch"),
            ("calibrate_gate_gamma", "Calibrate the gamma logic gate"),
            ("bridge_bus_connection", "Bridge the central data bus connection"),
            ("ground_wire_delta", "Ground the delta voltage wire"),
            ("bypass_safety_fuse", "Bypass the secondary safety fuse"),
            ("verify_truth_table", "Verify the output truth table state"),
            ("disarm_logic_trap", "Disarm the circuit feedback trap"),
            ("sync_clock_pulse", "Synchronize the master clock pulse"),
        ],
        Category::Cryptography => vec![
            ("intercept_cipher_text", "Intercept the encrypted cipher payload"),
            ("decode_substitution", "Decode the monoalphabetic substitution key"),
            ("shift_caesar_offset", "Shift the Caesar cipher rotary wheel"),
            ("calculate_crc_checksum", "Calculate the 32-bit CRC checksum"),
            ("apply_xor_mask", "Apply the stream cipher XOR mask"),
            ("decrypt_header_bytes", "Decrypt the payload header block"),
            ("verify_hash_signature", "Verify the cryptographic SHA signature"),
            ("crack_rsa_modulus", "Decompose the public RSA modulus"),
            ("extract_secret_nonce", "Extract the one-time secret initialization vector"),
            ("authorize_key_pair", "Authorize the decrypted master key pair"),
        ],
        Category::Dungeon => vec![
            ("find_iron_key", "Locate the heavy iron key in the wall alcove"),
            ("unlock_dungeon_grate", "Unlock the reinforced iron grate"),
            ("disarm_pressure_plate", "Disarm the concealed floor pressure plate"),
            ("ignite_stone_brazier", "Ignite the ritual stone brazier"),
            ("pull_shadow_lever", "Pull the concealed wall lever"),
            ("align_gargoyle_statue", "Align the twin gargoyle statues"),
            ("retrieve_ancient_relic", "Retrieve the relic from the stone pedestal"),
            ("quench_flame_trap", "Quench the fire-breathing wall trap"),
            ("rotate_sun_disk", "Rotate the celestial sun disk dial"),
            ("unseal_vault_door", "Unseal the heavy vault door mechanism"),
        ],
        Category::Alchemy => vec![
            ("gather_fire_herb", "Gather dried dragonfire herb leaves"),
            ("crush_moonstone_dust", "Crush moonstone into fine powder"),
            ("heat_brass_crucible", "Heat the brass crucible to boiling point"),
            ("distill_silver_essence", "Distill the pure silver liquid essence"),
            ("mix_catalyst_agent", "Mix the stabilizing catalyst agent"),
            ("quench_reaction_vial", "Quench the chemical reaction vial in water"),
            ("filter_sediment_precipitate", "Filter out the solid crystalline sediment"),
            ("seal_alchemical_flask", "Seal the glowing alchemical flask"),
            ("purify_elixir_compound", "Purify the master potion compound"),
            ("transmute_base_metal", "Transmute the base alloy into transmuted gold"),
        ],
        Category::Pattern => vec![
            ("observe_light_sequence", "Observe the repeating light sequence"),
            ("align_rhythmic_dial", "Align the rhythmic resonance dial"),
            ("match_color_nodes", "Match the glowing color node pairs"),
            ("rotate_concentric_rings", "Rotate the inner concentric rings"),
            ("balance_crystal_harmonics", "Balance the crystal acoustic harmonics"),
            ("calibrate_frequency_wave", "Calibrate the sine frequency wave"),
            ("solve_matrix_grid", "Solve the 3x3 symbol matrix grid"),
            ("synchronize_pulses", "Synchronize the harmonic energy pulses"),
            ("lock_pattern_phase", "Lock the phase lock loop in position"),
            ("stabilize_resonance_core", "Stabilize the main resonance core"),
        ],
    }
}

fn red_herring_pool_for_category(category: Category) -> Vec<(&'static str, &'static str)> {
    match category {
        Category::Logic => vec![
            ("cut_red_wire", "Cut the decoy red wire (herring)"),
            ("overload_capacitor", "Overload the secondary capacitor (herring)"),
            ("press_emergency_dump", "Press the emergency memory dump button (herring)"),
            ("short_circuit_bus", "Short circuit the auxiliary bus bar (herring)"),
            ("disable_cooling_fan", "Disable the chassis cooling fan (herring)"),
        ],
        Category::Cryptography => vec![
            ("parse_decoy_flag", "Parse the decoy secret flag (herring)"),
            ("inject_null_payload", "Inject null padding bytes into stream (herring)"),
            ("corrupt_index_table", "Corrupt the dictionary lookup table (herring)"),
            ("reverse_endianness", "Reverse payload byte endianness (herring)"),
            ("spoof_mac_address", "Spoof the network hardware address (herring)"),
        ],
        Category::Dungeon => vec![
            ("open_mimic_chest", "Open the suspicious wooden chest (herring)"),
            ("drink_murky_potion", "Drink the unlabelled green potion (herring)"),
            ("step_on_loose_tile", "Step on the loose crumbling floor tile (herring)"),
            ("touch_cursed_idol", "Touch the glowing jade idol (herring)"),
            ("ring_warning_bell", "Ring the rusted warning bell (herring)"),
        ],
        Category::Alchemy => vec![
            ("add_excess_sulfur", "Add excess sulfur powder to crucible (herring)"),
            ("boil_potion_dry", "Boil the mixture until completely dry (herring)"),
            ("spill_acid_solvent", "Spill acidic solvent onto laboratory bench (herring)"),
            ("inhale_noxious_vapor", "Inhale the strange purple vapor (herring)"),
            ("freeze_mixture_solid", "Freeze the liquid mixture prematurely (herring)"),
        ],
        Category::Pattern => vec![
            ("disrupt_wave_amplitude", "Disrupt the sound wave amplitude (herring)"),
            ("invert_color_spectrum", "Invert the visual color spectrum (herring)"),
            ("randomize_dial_positions", "Randomly spin all dials (herring)"),
            ("shatter_tuning_fork", "Shatter the acoustic tuning fork (herring)"),
            ("scramble_matrix_symbols", "Scramble the matrix tile order (herring)"),
        ],
    }
}

// ── Generator ────────────────────────────────────────────────────────────────

/// Procedurally generates a logic puzzle based on `difficulty`, `category`, and `seed`.
///
/// Generation is completely deterministic: identical `(difficulty, category, seed)`
/// calls will always yield identical [`Puzzle`] structs with identical condition
/// ordering and content hashes.
pub fn generate(difficulty: Difficulty, category: impl Into<Category>, seed: u64) -> Puzzle {
    let category = category.into();
    let params = DifficultyParams::for_difficulty(difficulty);
    let mut rng = Prng::new(seed);

    let step_pool = step_pool_for_category(category);
    let herring_pool = red_herring_pool_for_category(category);

    // Select step_pool items deterministically for main chain
    let mut selected_chain: Vec<(&'static str, &'static str)> = Vec::new();
    let mut indices: Vec<usize> = (0..step_pool.len()).collect();
    rng.shuffle(&mut indices);

    for &idx in indices.iter().take(params.chain_length) {
        selected_chain.push(step_pool[idx]);
    }

    // Select red herrings
    let mut selected_herrings: Vec<(&'static str, &'static str)> = Vec::new();
    let mut herring_indices: Vec<usize> = (0..herring_pool.len()).collect();
    rng.shuffle(&mut herring_indices);

    for &idx in herring_indices.iter().take(params.red_herrings) {
        selected_herrings.push(herring_pool[idx]);
    }

    // Combine conditions
    let mut conditions = Vec::new();
    for (id, desc) in &selected_chain {
        conditions.push(Condition::new(*id, *desc));
    }
    for (id, desc) in &selected_herrings {
        conditions.push(Condition::new(*id, *desc));
    }

    // Shuffle conditions so red herrings are interspersed deterministically
    rng.shuffle(&mut conditions);

    // Create puzzle ID & description
    let puzzle_id = format!(
        "{}-{:?}-{:08x}",
        category.as_str().to_lowercase(),
        difficulty,
        (seed & 0xffff_ffff) as u32
    );
    let description = format!(
        "[{}] Complete the procedural {} sequence to solve the puzzle.",
        category.as_str(),
        format!("{:?}", difficulty).to_lowercase()
    );

    // Effects scaling with difficulty
    let score = difficulty.apply_multiplier(100);
    let reward = difficulty.reward_amount();
    let effects = vec![
        Effect::AwardScore(score),
        Effect::GrantItem {
            item_id: format!("{}_badge", category.as_str().to_lowercase()),
            quantity: reward / 10,
        },
        Effect::UnlockAchievement(format!("Master of {}", category.as_str())),
    ];

    let mut puzzle = Puzzle::new(puzzle_id, description, conditions, effects);
    puzzle.generate_content_hash();
    puzzle
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_is_deterministic() {
        let p1 = generate(Difficulty::Medium, Category::Logic, 42);
        let p2 = generate(Difficulty::Medium, Category::Logic, 42);
        assert_eq!(p1, p2);
        assert_eq!(p1.content_hash, p2.content_hash);
    }

    #[test]
    fn different_seeds_produce_different_puzzles() {
        let p1 = generate(Difficulty::Medium, Category::Logic, 42);
        let p2 = generate(Difficulty::Medium, Category::Logic, 43);
        assert_ne!(p1.id, p2.id);
        assert_ne!(p1.conditions, p2.conditions);
    }

    #[test]
    fn difficulty_scaling_affects_condition_count() {
        let easy = generate(Difficulty::Easy, Category::Dungeon, 100);
        let medium = generate(Difficulty::Medium, Category::Dungeon, 100);
        let hard = generate(Difficulty::Hard, Category::Dungeon, 100);

        let params_easy = DifficultyParams::for_difficulty(Difficulty::Easy);
        let params_medium = DifficultyParams::for_difficulty(Difficulty::Medium);
        let params_hard = DifficultyParams::for_difficulty(Difficulty::Hard);

        assert_eq!(
            easy.conditions.len(),
            params_easy.chain_length + params_easy.red_herrings
        );
        assert_eq!(
            medium.conditions.len(),
            params_medium.chain_length + params_medium.red_herrings
        );
        assert_eq!(
            hard.conditions.len(),
            params_hard.chain_length + params_hard.red_herrings
        );

        assert!(easy.conditions.len() < medium.conditions.len());
        assert!(medium.conditions.len() < hard.conditions.len());
    }

    #[test]
    fn category_parsing_and_display() {
        assert_eq!("logic".parse::<Category>().unwrap(), Category::Logic);
        assert_eq!("dungeon".parse::<Category>().unwrap(), Category::Dungeon);
        assert_eq!("crypto".parse::<Category>().unwrap(), Category::Cryptography);
        assert_eq!(Category::Alchemy.to_string(), "Alchemy");
    }
}
