use crate::ai::PathFinder;
use crate::enemy::{EnemyAIState, EnemyManager, MAX_ENEMIES};
use crate::food::{Food, FoodSize};
use crate::menu;
use crate::rng::Rng;
use crate::snake::{Direction, Point, Snake, GRID_SIZE, MIN_SNAKE_LENGTH};
use crate::supply::Supply;
use crate::wasm4::*;

/// Size of each cell in pixels
const CELL_SIZE: u32 = 8;
/// Base frames between snake movements (60 FPS / 15 = 4 moves/sec)
const BASE_MOVE_INTERVAL: u32 = 15;
/// Frames between music notes (60 FPS / 8 = 7.5 notes/sec)
const MUSIC_INTERVAL: u32 = 8;
/// AI decision interval (frames)
const AI_DECISION_INTERVAL: u8 = 30;

// Background music melody (frequencies in Hz, 0 = rest)
const MELODY: [u32; 16] = [
    262, 294, 330, 294, // C D E D
    262, 330, 392, 330, // C E G E
    349, 330, 294, 262, // F E D C
    294, 330, 294, 0, // D E D (rest)
];

/// Game states
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    MainMenu,
    Settings,
    DifficultySelect,
    Playing,
    GameOver,
}

/// Difficulty levels
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Difficulty {
    Classic = 0,
    Noob = 1,
    Normal = 2,
    Hell = 3,
    Nightmare = 4,
}

impl Difficulty {
    /// Maximum number of enemies for this difficulty
    pub const fn max_enemies(&self) -> usize {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 2,
            Difficulty::Normal => 3,
            Difficulty::Hell => 5,
            Difficulty::Nightmare => 8,
        }
    }

    /// Enemy movement speed (frames per move, lower = faster)
    pub const fn enemy_speed(&self) -> u8 {
        match self {
            Difficulty::Classic => 20,
            Difficulty::Noob => 18,
            Difficulty::Normal => 15,
            Difficulty::Hell => 12,
            Difficulty::Nightmare => 10,
        }
    }

    /// Percentage of time enemies chase player (0-100)
    pub const fn chase_ratio(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 30,
            Difficulty::Normal => 50,
            Difficulty::Hell => 70,
            Difficulty::Nightmare => 85,
        }
    }

    /// AI intelligence level (affects pathfinding depth)
    pub const fn ai_intelligence(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 3,
            Difficulty::Normal => 5,
            Difficulty::Hell => 8,
            Difficulty::Nightmare => 12,
        }
    }

    /// Frames between enemy spawns
    pub const fn spawn_interval(&self) -> u16 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 300,
            Difficulty::Normal => 240,
            Difficulty::Hell => 180,
            Difficulty::Nightmare => 120,
        }
    }

    /// Get difficulty from index
    pub const fn from_index(index: u8) -> Self {
        match index {
            0 => Difficulty::Classic,
            1 => Difficulty::Noob,
            2 => Difficulty::Normal,
            3 => Difficulty::Hell,
            _ => Difficulty::Nightmare,
        }
    }

    /// Get index from difficulty
    pub const fn to_index(self) -> u8 {
        self as u8
    }

    /// Maximum snake length for this difficulty (0 = unlimited)
    /// Only non-Classic has length limits
    pub const fn max_snake_length(&self) -> usize {
        match self {
            Difficulty::Classic => 50, // Use array max, effectively unlimited
            Difficulty::Noob => 20,
            Difficulty::Normal => 18,
            Difficulty::Hell => 15,
            Difficulty::Nightmare => 12,
        }
    }

    /// Whether enemies can use energy in this difficulty
    pub const fn enemies_use_energy(&self) -> bool {
        !matches!(self, Difficulty::Classic)
    }

    /// Probability (0-100) that AI will go for supply pack
    pub const fn supply_aggression(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 10,
            Difficulty::Normal => 30,
            Difficulty::Hell => 50,
            Difficulty::Nightmare => 75,
        }
    }

    /// Probability (0-100) that AI will flee when in danger
    pub const fn escape_intelligence(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 20,
            Difficulty::Normal => 40,
            Difficulty::Hell => 60,
            Difficulty::Nightmare => 80,
        }
    }

    /// Probability (0-100) that AI will use energy efficiently
    pub const fn energy_efficiency(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 30,
            Difficulty::Normal => 50,
            Difficulty::Hell => 70,
            Difficulty::Nightmare => 90,
        }
    }

    /// Probability (0-100) that AI will use slowdown ability
    pub const fn slowdown_tendency(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 20,
            Difficulty::Normal => 40,
            Difficulty::Hell => 60,
            Difficulty::Nightmare => 80,
        }
    }

    /// Probability (0-100) that AI will attack even when shorter than target.
    /// Higher difficulty = more willing to sacrifice length for attack opportunities.
    pub const fn sacrifice_willingness(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 0, // Never sacrifices, always flees when shorter
            Difficulty::Normal => 15, // Occasionally takes risks
            Difficulty::Hell => 40, // Often attacks despite being shorter
            Difficulty::Nightmare => 70, // Very aggressive, frequently sacrifices length
        }
    }

    /// Probability (0-100) that AI will attempt head-to-head collision.
    /// Higher difficulty = more willing to trade damage for potential kills.
    pub const fn head_clash_willingness(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 0,       // Always avoids head-to-head
            Difficulty::Normal => 10,    // Occasionally risks it
            Difficulty::Hell => 25,      // Sometimes attempts head clash
            Difficulty::Nightmare => 45, // Frequently attempts mutual damage
        }
    }

    /// Probability (0-100) that AI will aggressively seek head-to-head when longer.
    /// New head-to-head rules reward longer snakes, so high difficulty AI exploits this.
    pub const fn offensive_head_attack(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 0,
            Difficulty::Normal => 20,
            Difficulty::Hell => 45,
            Difficulty::Nightmare => 70,
        }
    }

    /// Probability (0-100) that AI will prioritize attacking player over other enemies.
    pub const fn player_focus(&self) -> u8 {
        match self {
            Difficulty::Classic => 0,
            Difficulty::Noob => 40,
            Difficulty::Normal => 60,
            Difficulty::Hell => 75,
            Difficulty::Nightmare => 90,
        }
    }
}

/// Check if two directions are facing each other (opposite directions)
fn is_facing_each_other(dir_a: Direction, dir_b: Direction) -> bool {
    dir_a == dir_b.opposite()
}

/// Boost duration in frames (2 seconds at 60 FPS)
const BOOST_DURATION: u16 = 120;
/// Boost/slow cooldown in frames (5 seconds at 60 FPS)
const ABILITY_COOLDOWN: u16 = 300;
/// Boosted move interval (faster than normal)
const BOOSTED_MOVE_INTERVAL: u32 = 8;
/// Slowed move interval multiplier (2x slower)
const SLOW_MULTIPLIER: u32 = 2;

/// Main game struct
pub struct Game {
    snake: Snake,
    food: Food,
    supply: Supply,
    enemies: EnemyManager,
    pathfinder: PathFinder,
    rng: Rng,
    state: GameState,
    difficulty: Difficulty,
    frame_count: u32,
    score: u32,
    move_interval: u32,
    prev_gamepad: u8,
    music_index: usize,
    menu_selection: u8,
    high_scores: [u32; 5],
    blink_timer: u8,
    obstacles_cache: [Point; 128], // Cache for AI pathfinding (limited for stack safety)
    obstacles_count: usize,
    // Sound settings
    music_enabled: bool,
    sfx_enabled: bool,
    // Cheat mode (unlimited boost, no cooldown)
    cheat_enabled: bool,
    // Boost/Slow timers (non-Classic only)
    boost_timer: u16,    // Remaining boost frames
    boost_cooldown: u16, // Cooldown until next boost
    slow_timer: u16,     // Remaining slow frames
    slow_cooldown: u16,  // Cooldown until next slow
    // Collision flash effect
    collision_flash_pos: Option<Point>, // Flash position (None = no flash)
    collision_flash_timer: u8,          // Remaining flash frames
}

impl Game {
    /// Create a new game
    pub fn new() -> Self {
        let mut rng = Rng::new(12345);
        let snake = Snake::new();
        let food = Food::new(&mut rng, &snake);

        let mut game = Self {
            snake,
            food,
            supply: Supply::new(),
            enemies: EnemyManager::new(),
            pathfinder: PathFinder::new(),
            rng,
            state: GameState::MainMenu,
            difficulty: Difficulty::Normal,
            frame_count: 0,
            score: 0,
            move_interval: BASE_MOVE_INTERVAL,
            prev_gamepad: 0,
            music_index: 0,
            menu_selection: 0,
            high_scores: [0; 5],
            blink_timer: 0,
            obstacles_cache: [Point::new(0, 0); 128],
            obstacles_count: 0,
            music_enabled: true,
            sfx_enabled: true,
            cheat_enabled: false,
            boost_timer: 0,
            boost_cooldown: 0,
            slow_timer: 0,
            slow_cooldown: 0,
            collision_flash_pos: None,
            collision_flash_timer: 0,
        };

        game.load_high_scores();
        game
    }

    /// Reset game state for a new round
    fn reset_game(&mut self) {
        self.snake = Snake::new();
        self.food.respawn(&mut self.rng, &self.snake);
        self.supply.reset();
        self.enemies.reset();
        self.score = 0;
        self.move_interval = BASE_MOVE_INTERVAL;
        self.music_index = 0;
        self.boost_timer = 0;
        self.boost_cooldown = 0;
        self.slow_timer = 0;
        self.slow_cooldown = 0;
        self.collision_flash_pos = None;
        self.collision_flash_timer = 0;
        self.state = GameState::Playing;
    }

    /// Main update function called every frame
    pub fn update(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
        self.blink_timer = self.blink_timer.wrapping_add(1);

        self.set_palette();
        self.handle_input();

        match self.state {
            GameState::MainMenu => {
                self.play_menu_music();
                self.draw_main_menu();
            }
            GameState::Settings => {
                self.play_menu_music();
                self.draw_settings_menu();
            }
            GameState::DifficultySelect => {
                self.draw_difficulty_select();
            }
            GameState::Playing => {
                // Update boost/slow timers (non-Classic only)
                if self.difficulty != Difficulty::Classic {
                    self.update_ability_timers();
                }

                // Update collision flash timer
                if self.collision_flash_timer > 0 {
                    self.collision_flash_timer -= 1;
                    if self.collision_flash_timer == 0 {
                        self.collision_flash_pos = None;
                    }
                }

                // Calculate effective move interval based on boost/slow state
                let effective_interval = self.get_effective_move_interval();

                // Update player snake
                if self.frame_count.is_multiple_of(effective_interval) {
                    self.update_game_logic();
                }

                // Update enemies
                self.update_enemies();

                // Update supply (Classic mode has no supply)
                if self.difficulty != Difficulty::Classic {
                    self.update_supply();
                }

                // Play background music
                self.play_music();
                self.draw_game();
            }
            GameState::GameOver => {
                self.draw_game();
                self.draw_game_over();
            }
        }
    }

    /// Build obstacles cache for AI pathfinding
    fn build_obstacles_cache(&mut self) {
        self.obstacles_count = 0;

        // Add player snake body (skip head for chasing)
        for i in 0..self.snake.length {
            if self.obstacles_count < self.obstacles_cache.len() {
                self.obstacles_cache[self.obstacles_count] = self.snake.body[i];
                self.obstacles_count += 1;
            }
        }

        // Add all enemy bodies
        for enemy in &self.enemies.enemies {
            if enemy.alive {
                for i in 0..enemy.length {
                    if self.obstacles_count < self.obstacles_cache.len() {
                        self.obstacles_cache[self.obstacles_count] = enemy.body[i];
                        self.obstacles_count += 1;
                    }
                }
            }
        }
    }

    /// Update enemy AI and movement
    fn update_enemies(&mut self) {
        let difficulty = self.difficulty;

        // Spawn new enemies
        self.enemies.update_spawning(
            &mut self.rng,
            difficulty.spawn_interval(),
            difficulty.max_enemies(),
            &self.snake.body,
            self.snake.length,
        );

        // Build obstacles cache
        self.build_obstacles_cache();

        let enemy_speed = difficulty.enemy_speed();
        let chase_ratio = difficulty.chase_ratio();
        let ai_intelligence = difficulty.ai_intelligence();
        let supply_aggression = difficulty.supply_aggression();
        let escape_intelligence = difficulty.escape_intelligence();
        let energy_efficiency = difficulty.energy_efficiency();
        let food_pos = self.food.position;
        let player_head = self.snake.head();
        let player_length = self.snake.length;
        let supply_active = self.supply.active;
        let supply_pos = self.supply.position;

        let enemies_use_energy = difficulty.enemies_use_energy();
        let max_len = difficulty.max_snake_length();

        // Update each enemy
        for i in 0..MAX_ENEMIES {
            let enemy = &mut self.enemies.enemies[i];
            if !enemy.alive {
                continue;
            }

            // Update boost/slow timers
            if enemy.boost_timer > 0 {
                enemy.boost_timer -= 1;
            }
            if enemy.slow_timer > 0 {
                enemy.slow_timer -= 1;
            }

            // Update move timer
            enemy.move_timer = enemy.move_timer.wrapping_add(1);

            // AI decision making
            enemy.decision_timer = enemy.decision_timer.wrapping_add(1);
            if enemy.decision_timer >= AI_DECISION_INTERVAL {
                enemy.decision_timer = 0;

                let enemy_head = enemy.head();
                let dist_to_player =
                    (enemy_head.x - player_head.x).abs() + (enemy_head.y - player_head.y).abs();
                let dist_to_supply = if supply_active {
                    (enemy_head.x - supply_pos.x).abs() + (enemy_head.y - supply_pos.y).abs()
                } else {
                    100 // Far away when no supply
                };

                // Calculate length difference for decisions
                // Positive = player is longer, negative = enemy is longer
                let length_diff = player_length as i32 - enemy.length as i32;
                let sacrifice_willingness = difficulty.sacrifice_willingness();
                let offensive_head_attack = difficulty.offensive_head_attack();
                let player_focus = difficulty.player_focus();

                // NEW: Check if AI should aggressively seek head-to-head when LONGER than player
                // With new rules, longer snake wins and absorbs shorter one
                let should_offensive_head = length_diff < 0 // Enemy is longer
                    && dist_to_player < 6
                    && self.rng.range(0, 100) < offensive_head_attack as i32;

                // Check if AI should sacrifice length to attack (aggressive behavior)
                // Only when: close to player, player is longer (but not too much), AI has length to spare
                let should_sacrifice_attack = dist_to_player < 5
                    && length_diff > 0
                    && length_diff <= 3 // Don't suicide against much longer snakes
                    && enemy.length > MIN_SNAKE_LENGTH + 1 // Have length to spare
                    && self.rng.range(0, 100) < sacrifice_willingness as i32;

                // Check if AI should attempt head-to-head collision
                // Only when: very close, has length to spare, player is near death or not much longer
                let should_head_clash = dist_to_player <= 2
                    && enemy.length > MIN_SNAKE_LENGTH
                    && (player_length <= MIN_SNAKE_LENGTH + 1 || length_diff <= 0)
                    && self.rng.range(0, 100) < difficulty.head_clash_willingness() as i32;

                // Check if should flee (player is close and longer, AND not willing to sacrifice/clash)
                let should_flee = dist_to_player < 4
                    && length_diff > 0
                    && !should_sacrifice_attack
                    && !should_head_clash
                    && !should_offensive_head
                    && self.rng.range(0, 100) < escape_intelligence as i32;

                // Check if should grab supply
                let should_grab_supply = supply_active
                    && dist_to_supply < 8
                    && self.rng.range(0, 100) < supply_aggression as i32;

                // Decide AI state - offensive head attack and sacrifice attacks take priority
                if should_offensive_head || should_sacrifice_attack || should_head_clash {
                    enemy.ai_state = EnemyAIState::Chasing; // Aggressively chase for attack
                } else if should_flee {
                    enemy.ai_state = EnemyAIState::Fleeing;
                } else if should_grab_supply {
                    enemy.ai_state = EnemyAIState::GrabbingSupply;
                } else {
                    // Use player_focus to determine chase probability
                    let effective_chase =
                        chase_ratio.max(if self.rng.range(0, 100) < player_focus as i32 {
                            chase_ratio + 20 // Boost chase ratio when player-focused
                        } else {
                            chase_ratio
                        });
                    let roll = self.rng.range(0, 100) as u8;
                    if roll < effective_chase {
                        enemy.ai_state = EnemyAIState::Chasing;
                    } else {
                        enemy.ai_state = EnemyAIState::Seeking;
                    }
                }

                // Decide if should use boost (when has energy)
                if enemies_use_energy
                    && enemy.energy > 0
                    && enemy.boost_timer == 0
                    && self.rng.range(0, 100) < energy_efficiency as i32
                {
                    let should_boost = match enemy.ai_state {
                        // Boost when chasing and close to player
                        // Allow boost when longer OR when willing to sacrifice
                        EnemyAIState::Chasing => {
                            dist_to_player < (ai_intelligence as i32 + 3)
                                && (enemy.length >= player_length
                                    || self.rng.range(0, 100) < sacrifice_willingness as i32)
                        }
                        // Boost when fleeing from danger
                        EnemyAIState::Fleeing => dist_to_player < 4,
                        // Boost when close to supply (within 5 cells)
                        EnemyAIState::GrabbingSupply => dist_to_supply < 5,
                        _ => false,
                    };

                    if should_boost && enemy.use_energy() {
                        enemy.boost_timer = BOOST_DURATION;
                    }
                }

                // Decide if should use slow (for precise control or damage reduction)
                let slowdown_tendency = difficulty.slowdown_tendency();

                // Check if enemy head is colliding with player body (taking damage)
                let in_collision = self.snake.body[1..self.snake.length].contains(&enemy_head);

                // Force slowdown when in collision (damage reduction strategy)
                // Higher difficulty AI will use this more reliably
                if enemies_use_energy && enemy.slow_timer == 0 && enemy.boost_timer == 0 {
                    if in_collision {
                        // In collision: force slowdown based on slowdown_tendency
                        // Higher difficulty = higher tendency = always slows down
                        if self.rng.range(0, 100) < slowdown_tendency as i32 {
                            enemy.slow_timer = BOOST_DURATION / 2;
                            // Cancel any boost when taking damage
                            enemy.boost_timer = 0;
                        }
                    } else if self.rng.range(0, 100) < slowdown_tendency as i32 {
                        // Normal slowdown logic: near food or supply
                        let dist_to_food =
                            (enemy_head.x - food_pos.x).abs() + (enemy_head.y - food_pos.y).abs();
                        let should_slow = (dist_to_food <= 2) || (dist_to_supply <= 2);
                        if should_slow {
                            enemy.slow_timer = BOOST_DURATION / 2;
                        }
                    }
                }
            }

            // Calculate effective speed based on boost/slow state
            let base_speed = enemy_speed as u16;
            let effective_speed = if enemy.boost_timer > 0 {
                (base_speed / 2).max(4) // Faster when boosted
            } else if enemy.slow_timer > 0 {
                base_speed * 2 // Slower when slowed
            } else {
                base_speed
            };

            if enemy.move_timer as u16 >= effective_speed {
                enemy.move_timer = 0;

                // Get target based on AI state
                let target = match enemy.ai_state {
                    EnemyAIState::Chasing => player_head,
                    EnemyAIState::Seeking | EnemyAIState::Idle => food_pos,
                    EnemyAIState::GrabbingSupply => supply_pos,
                    EnemyAIState::Fleeing => {
                        // Flee in opposite direction from player
                        let dx = enemy.head().x - player_head.x;
                        let dy = enemy.head().y - player_head.y;
                        Point::new(
                            (enemy.head().x + dx.signum() * 5).rem_euclid(GRID_SIZE),
                            (enemy.head().y + dy.signum() * 5).rem_euclid(GRID_SIZE),
                        )
                    }
                };

                // Find direction using pathfinding
                if let Some(dir) = self.pathfinder.find_direction(
                    enemy.head(),
                    target,
                    &self.obstacles_cache[..self.obstacles_count],
                    ai_intelligence,
                ) {
                    enemy.set_direction(dir);
                } else {
                    // Fallback: find any safe direction
                    if let Some(safe_dir) = self.pathfinder.find_safe_direction(
                        enemy.head(),
                        &self.obstacles_cache[..self.obstacles_count],
                        enemy.direction,
                    ) {
                        enemy.set_direction(safe_dir);
                    }
                }

                // Check if moving would collide with player body (before moving)
                let next_head = enemy.peek_next_head();
                let would_hit_player_body =
                    self.snake.body[1..self.snake.length].contains(&next_head);
                let would_hit_player_head = next_head == player_head;

                if would_hit_player_head {
                    // Head-to-head collision with new rules
                    enemy.just_moved = true;

                    if difficulty == Difficulty::Classic {
                        // Classic mode: player dies
                        self.on_game_over();
                        return;
                    } else {
                        // New head-to-head rules based on direction and length
                        let player_dir = self.snake.direction;
                        let enemy_dir = self.enemies.enemies[i].direction;
                        let player_len = self.snake.length;
                        let enemy_len = self.enemies.enemies[i].length;
                        let facing = is_facing_each_other(player_dir, enemy_dir);

                        self.trigger_collision_flash(next_head);

                        if facing {
                            // Both facing each other: longer one wins
                            if enemy_len > player_len {
                                // Enemy wins - player dies
                                self.play_head_clash_sound();
                                self.on_game_over();
                                return;
                            } else if player_len > enemy_len {
                                // Player wins - absorb enemy's length
                                let absorbed = enemy_len;
                                for _ in 0..absorbed {
                                    self.snake.try_grow_or_energy(max_len);
                                }
                                self.score += (absorbed as u32) * 50;
                                self.enemies.kill(i);
                                self.play_head_kill_sound();
                            } else {
                                // Equal length: both lose 1
                                let player_survived = self.snake.shrink();
                                let enemy_survived = self.enemies.enemies[i].shrink();
                                self.play_head_clash_sound();
                                if !player_survived {
                                    self.on_game_over();
                                    return;
                                }
                                if !enemy_survived {
                                    self.enemies.kill(i);
                                    self.score += 15;
                                }
                            }
                        } else {
                            // Not facing: enemy is attacking (moving toward player)
                            if enemy_len > player_len {
                                // Enemy longer: player dies
                                self.play_head_clash_sound();
                                self.on_game_over();
                                return;
                            } else {
                                // Enemy equal or shorter: enemy loses 1, player gains 1
                                self.snake.try_grow_or_energy(max_len);
                                self.play_enemy_hurt_sound();
                                if !self.enemies.enemies[i].shrink() {
                                    self.enemies.kill(i);
                                }
                                self.score += 10;
                            }
                        }
                    }
                } else if would_hit_player_body {
                    // Don't move into player body, but apply damage here
                    enemy.just_moved = true;

                    // Apply damage: enemy shrinks, player grows
                    if difficulty == Difficulty::Classic {
                        // Classic mode: enemy dies
                        self.enemies.kill(i);
                        self.score += 50;
                    } else {
                        // Battle mode: enemy shrinks, player grows
                        self.snake.try_grow_or_energy(max_len);
                        self.play_enemy_hurt_sound();
                        self.trigger_collision_flash(next_head);
                        if !self.enemies.enemies[i].shrink() {
                            self.enemies.kill(i);
                        }
                        self.score += 10;
                    }
                } else {
                    enemy.update();
                    enemy.just_moved = true;
                }
            } else {
                enemy.just_moved = false;
            }
        }

        // Check collisions between enemies (only for enemies that just moved)
        self.check_enemy_collisions_with_shrink();

        // Check if enemies ate food
        for i in 0..MAX_ENEMIES {
            let enemy = &mut self.enemies.enemies[i];
            if enemy.alive && enemy.head() == self.food.position {
                let growth = self.food.size.growth_amount();
                for _ in 0..growth {
                    enemy.try_grow_or_energy(max_len);
                }
                self.food.respawn(&mut self.rng, &self.snake);
            }
        }
    }

    /// Check collisions between enemies with shrink rules
    /// Only applies damage when an enemy that just moved causes a collision
    fn check_enemy_collisions_with_shrink(&mut self) {
        let max_len = self.difficulty.max_snake_length();

        for i in 0..MAX_ENEMIES {
            if !self.enemies.enemies[i].alive {
                continue;
            }

            for j in (i + 1)..MAX_ENEMIES {
                if !self.enemies.enemies[j].alive {
                    continue;
                }

                // Only check collision if at least one enemy just moved
                let i_moved = self.enemies.enemies[i].just_moved;
                let j_moved = self.enemies.enemies[j].just_moved;
                if !i_moved && !j_moved {
                    continue;
                }

                let head_i = self.enemies.enemies[i].head();
                let head_j = self.enemies.enemies[j].head();

                // Head-to-head collision with new rules
                if head_i == head_j {
                    let dir_i = self.enemies.enemies[i].direction;
                    let dir_j = self.enemies.enemies[j].direction;
                    let len_i = self.enemies.enemies[i].length;
                    let len_j = self.enemies.enemies[j].length;
                    let facing = is_facing_each_other(dir_i, dir_j);

                    if facing {
                        // Both facing: longer wins and absorbs
                        if len_i > len_j {
                            // i wins
                            for _ in 0..len_j {
                                self.enemies.enemies[i].try_grow_or_energy(max_len);
                            }
                            self.enemies.kill(j);
                        } else if len_j > len_i {
                            // j wins
                            for _ in 0..len_i {
                                self.enemies.enemies[j].try_grow_or_energy(max_len);
                            }
                            self.enemies.kill(i);
                        } else {
                            // Equal: both lose 1
                            if !self.enemies.enemies[i].shrink() {
                                self.enemies.kill(i);
                            }
                            if !self.enemies.enemies[j].shrink() {
                                self.enemies.kill(j);
                            }
                        }
                    } else {
                        // Not facing: check who moved (attacker)
                        if i_moved && !j_moved {
                            // i is attacking j
                            if len_i > len_j {
                                for _ in 0..len_j {
                                    self.enemies.enemies[i].try_grow_or_energy(max_len);
                                }
                                self.enemies.kill(j);
                            } else {
                                // i loses 1, j gains 1
                                self.enemies.enemies[j].try_grow_or_energy(max_len);
                                if !self.enemies.enemies[i].shrink() {
                                    self.enemies.kill(i);
                                }
                            }
                        } else if j_moved && !i_moved {
                            // j is attacking i
                            if len_j > len_i {
                                for _ in 0..len_i {
                                    self.enemies.enemies[j].try_grow_or_energy(max_len);
                                }
                                self.enemies.kill(i);
                            } else {
                                // j loses 1, i gains 1
                                self.enemies.enemies[i].try_grow_or_energy(max_len);
                                if !self.enemies.enemies[j].shrink() {
                                    self.enemies.kill(j);
                                }
                            }
                        } else {
                            // Both moved: treat as mutual, longer wins
                            if len_i > len_j {
                                for _ in 0..len_j {
                                    self.enemies.enemies[i].try_grow_or_energy(max_len);
                                }
                                self.enemies.kill(j);
                            } else if len_j > len_i {
                                for _ in 0..len_i {
                                    self.enemies.enemies[j].try_grow_or_energy(max_len);
                                }
                                self.enemies.kill(i);
                            } else {
                                // Equal: both lose 1
                                if !self.enemies.enemies[i].shrink() {
                                    self.enemies.kill(i);
                                }
                                if !self.enemies.enemies[j].shrink() {
                                    self.enemies.kill(j);
                                }
                            }
                        }
                    }
                    continue;
                }

                // Check if j's head hits i's body (only if j just moved)
                if j_moved
                    && self.enemies.enemies[i].length > 1
                    && self.enemies.enemies[i].body[1..self.enemies.enemies[i].length]
                        .contains(&head_j)
                {
                    // j's head hit i's body: i grows, j shrinks
                    self.enemies.enemies[i].try_grow_or_energy(max_len);
                    if !self.enemies.enemies[j].shrink() {
                        self.enemies.kill(j);
                    }
                }

                // Check if i's head hits j's body (only if i just moved)
                if i_moved
                    && self.enemies.enemies[j].length > 1
                    && self.enemies.enemies[j].body[1..self.enemies.enemies[j].length]
                        .contains(&head_i)
                {
                    // i's head hit j's body: j grows, i shrinks
                    self.enemies.enemies[j].try_grow_or_energy(max_len);
                    if !self.enemies.enemies[i].shrink() {
                        self.enemies.kill(i);
                    }
                }
            }
        }
    }

    /// Set the color palette
    fn set_palette(&self) {
        unsafe {
            (*PALETTE)[0] = 0x1a1c2c; // Dark blue (background)
            (*PALETTE)[1] = 0x5d275d; // Purple
            (*PALETTE)[2] = 0x38b764; // Green (snake body)
            (*PALETTE)[3] = 0xf6c64f; // Yellow (snake head, food)
        }
    }

    /// Handle gamepad input
    fn handle_input(&mut self) {
        let gamepad = unsafe { *GAMEPAD1 };
        let just_pressed = gamepad & (gamepad ^ self.prev_gamepad);

        match self.state {
            GameState::MainMenu => {
                if just_pressed & BUTTON_UP != 0 {
                    if self.menu_selection > 0 {
                        self.menu_selection -= 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_DOWN != 0 {
                    if self.menu_selection < 1 {
                        self.menu_selection += 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_1 != 0 {
                    if self.menu_selection == 0 {
                        // Start
                        self.state = GameState::DifficultySelect;
                        self.menu_selection = self.difficulty.to_index();
                    } else {
                        // Settings
                        self.state = GameState::Settings;
                        self.menu_selection = 0;
                    }
                    self.play_menu_sound();
                }
            }
            GameState::Settings => {
                if just_pressed & BUTTON_UP != 0 {
                    if self.menu_selection > 0 {
                        self.menu_selection -= 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_DOWN != 0 {
                    if self.menu_selection < 2 {
                        self.menu_selection += 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_1 != 0 {
                    // Toggle setting
                    match self.menu_selection {
                        0 => self.music_enabled = !self.music_enabled,
                        1 => self.sfx_enabled = !self.sfx_enabled,
                        _ => self.cheat_enabled = !self.cheat_enabled,
                    }
                    self.save_settings();
                    self.play_menu_sound();
                } else if just_pressed & BUTTON_2 != 0 {
                    // Back to main menu
                    self.state = GameState::MainMenu;
                    self.menu_selection = 0;
                    self.play_menu_sound();
                }
            }
            GameState::DifficultySelect => {
                if just_pressed & BUTTON_UP != 0 {
                    if self.menu_selection > 0 {
                        self.menu_selection -= 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_DOWN != 0 {
                    if self.menu_selection < 4 {
                        self.menu_selection += 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_1 != 0 {
                    self.difficulty = Difficulty::from_index(self.menu_selection);
                    self.rng.seed(self.frame_count.wrapping_mul(31337));
                    self.reset_game();
                    self.play_start_sound();
                } else if just_pressed & BUTTON_2 != 0 {
                    self.state = GameState::MainMenu;
                    self.menu_selection = 0;
                    self.play_menu_sound();
                }
            }
            GameState::Playing => {
                // Direction controls
                if just_pressed & BUTTON_UP != 0 {
                    self.snake.set_direction(Direction::Up);
                } else if just_pressed & BUTTON_DOWN != 0 {
                    self.snake.set_direction(Direction::Down);
                } else if just_pressed & BUTTON_LEFT != 0 {
                    self.snake.set_direction(Direction::Left);
                } else if just_pressed & BUTTON_RIGHT != 0 {
                    self.snake.set_direction(Direction::Right);
                }

                // Cheat: A+B to grow
                if self.cheat_enabled
                    && (just_pressed & (BUTTON_1 | BUTTON_2)) != 0
                    && (gamepad & BUTTON_1) != 0
                    && (gamepad & BUTTON_2) != 0
                {
                    self.snake.grow();
                    self.play_eat_sound();
                }
                // Boost/Slow abilities (non-Classic only)
                else if self.difficulty != Difficulty::Classic {
                    // BUTTON_1 (X) = Boost
                    if just_pressed & BUTTON_1 != 0 && self.boost_timer == 0 && self.slow_timer == 0
                    {
                        let can_boost = if self.cheat_enabled {
                            true
                        } else {
                            self.boost_cooldown == 0 && self.snake.use_energy()
                        };

                        if can_boost {
                            self.boost_timer = BOOST_DURATION;
                            self.play_boost_sound();
                        }
                    }

                    // BUTTON_2 (Z) = Slow
                    if just_pressed & BUTTON_2 != 0
                        && self.slow_cooldown == 0
                        && self.slow_timer == 0
                    {
                        if self.boost_timer > 0 {
                            self.boost_timer = 0;
                            if !self.cheat_enabled {
                                self.boost_cooldown = ABILITY_COOLDOWN;
                            }
                        }
                        self.slow_timer = BOOST_DURATION;
                        self.play_slow_sound();
                    }
                }
            }
            GameState::GameOver => {
                if just_pressed & BUTTON_UP != 0 {
                    if self.menu_selection > 0 {
                        self.menu_selection -= 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_DOWN != 0 {
                    if self.menu_selection < 1 {
                        self.menu_selection += 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_1 != 0 {
                    if self.menu_selection == 0 {
                        self.rng.seed(self.frame_count.wrapping_mul(31337));
                        self.reset_game();
                    } else {
                        self.state = GameState::MainMenu;
                        self.menu_selection = 0;
                    }
                    self.play_menu_sound();
                }
            }
        }

        self.prev_gamepad = gamepad;
    }

    /// Update supply spawning and collection
    fn update_supply(&mut self) {
        // Create a closure to check enemy collisions
        let enemies = &self.enemies;
        let enemy_check =
            |pos: Point| -> bool { enemies.enemies.iter().any(|e| e.alive && e.contains(pos)) };

        // Update spawning
        self.supply.update_spawning(
            &mut self.rng,
            &self.snake.body,
            self.snake.length,
            self.food.position,
            enemy_check,
        );

        // Check if player collected supply
        if self.supply.is_at(self.snake.head()) {
            self.supply.collect();
            // Add 1 energy (up to max)
            if self.snake.energy < crate::snake::MAX_ENERGY {
                self.snake.energy += 1;
            }
            self.play_supply_sound();
        }

        // Check if enemies collected supply
        for i in 0..MAX_ENEMIES {
            let enemy = &mut self.enemies.enemies[i];
            if enemy.alive && self.supply.is_at(enemy.head()) {
                self.supply.collect();
                if enemy.energy < crate::snake::MAX_ENERGY {
                    enemy.energy += 1;
                }
                break; // Only one can collect per frame
            }
        }
    }

    /// Update boost and slow ability timers
    fn update_ability_timers(&mut self) {
        // Update boost timer
        if self.boost_timer > 0 {
            self.boost_timer -= 1;
            if self.boost_timer == 0 && !self.cheat_enabled {
                self.boost_cooldown = ABILITY_COOLDOWN;
            }
        }

        // Update boost cooldown (skip in cheat mode)
        if self.boost_cooldown > 0 && !self.cheat_enabled {
            self.boost_cooldown -= 1;
        }

        // Update slow timer
        if self.slow_timer > 0 {
            self.slow_timer -= 1;
            if self.slow_timer == 0 {
                self.slow_cooldown = ABILITY_COOLDOWN;
            }
        }

        // Update slow cooldown
        if self.slow_cooldown > 0 {
            self.slow_cooldown -= 1;
        }
    }

    /// Get the effective move interval based on boost/slow state
    fn get_effective_move_interval(&self) -> u32 {
        if self.boost_timer > 0 {
            // Boosted: faster movement
            BOOSTED_MOVE_INTERVAL
        } else if self.slow_timer > 0 {
            // Slowed: half speed
            self.move_interval * SLOW_MULTIPLIER
        } else {
            // Normal speed
            self.move_interval
        }
    }

    /// Update game logic (movement, collision, scoring)
    fn update_game_logic(&mut self) {
        let next_head = self.snake.peek_next_head();
        let is_classic = self.difficulty == Difficulty::Classic;
        let max_len = self.difficulty.max_snake_length();

        // Check if moving would cause collision (before moving)
        let mut should_move = true;
        let mut game_over = false;

        // Check self-collision BEFORE moving - snake cannot pass through itself
        if self.snake.would_collide_with_self() {
            should_move = false; // Stay in place
            if is_classic {
                game_over = true;
            } else {
                // Battle mode: take damage but don't move
                self.play_self_hurt_sound();
                self.trigger_collision_flash(next_head);
                if !self.snake.shrink() {
                    game_over = true;
                }
            }
        }

        // Check enemy collisions only if we haven't already determined to stop
        if should_move {
            for i in 0..MAX_ENEMIES {
                if !self.enemies.enemies[i].alive {
                    continue;
                }

                let enemy_head = self.enemies.enemies[i].head();

                // Head-to-head collision check with new rules
                if next_head == enemy_head {
                    should_move = false; // Don't move into collision
                    if is_classic {
                        game_over = true;
                    } else {
                        // New head-to-head rules based on direction and length
                        let player_dir = self.snake.direction;
                        let enemy_dir = self.enemies.enemies[i].direction;
                        let player_len = self.snake.length;
                        let enemy_len = self.enemies.enemies[i].length;
                        let facing = is_facing_each_other(player_dir, enemy_dir);

                        self.trigger_collision_flash(next_head);

                        if facing {
                            // Both facing each other: longer one wins
                            if player_len > enemy_len {
                                // Player wins - absorb enemy's length
                                let absorbed = enemy_len;
                                for _ in 0..absorbed {
                                    self.snake.try_grow_or_energy(max_len);
                                }
                                self.score += (absorbed as u32) * 50;
                                self.enemies.kill(i);
                                self.play_head_kill_sound();
                            } else if enemy_len > player_len {
                                // Enemy wins - player dies
                                game_over = true;
                                self.play_head_clash_sound();
                            } else {
                                // Equal length: both lose 1
                                let player_survived = self.snake.shrink();
                                let enemy_survived = self.enemies.enemies[i].shrink();
                                self.play_head_clash_sound();
                                if !player_survived {
                                    game_over = true;
                                }
                                if !enemy_survived {
                                    self.enemies.kill(i);
                                    self.score += 15;
                                }
                            }
                        } else {
                            // Not facing each other: player is attacking (moving toward enemy)
                            if player_len > enemy_len {
                                // Player longer: enemy dies, player absorbs
                                let absorbed = enemy_len;
                                for _ in 0..absorbed {
                                    self.snake.try_grow_or_energy(max_len);
                                }
                                self.score += (absorbed as u32) * 50;
                                self.enemies.kill(i);
                                self.play_head_kill_sound();
                            } else {
                                // Player equal or shorter: player loses 1, enemy gains 1
                                self.enemies.enemies[i].try_grow_or_energy(max_len);
                                self.play_player_hurt_sound();
                                if !self.snake.shrink() {
                                    game_over = true;
                                }
                            }
                        }
                    }
                    break;
                }

                // Player head vs enemy body collision check
                if self.enemies.enemies[i].length > 1
                    && self.enemies.enemies[i].body[1..self.enemies.enemies[i].length]
                        .contains(&next_head)
                {
                    should_move = false; // Don't move into collision
                    if is_classic {
                        game_over = true;
                    } else {
                        self.enemies.enemies[i].try_grow_or_energy(max_len);
                        self.play_player_hurt_sound();
                        self.trigger_collision_flash(next_head);
                        if !self.snake.shrink() {
                            game_over = true;
                        }
                        self.score = self.score.saturating_sub(10);
                    }
                    break;
                }
            }
        } // End of if should_move (enemy collision checks)

        if game_over {
            self.on_game_over();
            return;
        }

        // Only move if no collision detected
        if should_move {
            self.snake.update();
        }

        // Check if snake ate food
        if self.snake.head() == self.food.position {
            let growth = self.food.size.growth_amount();
            let score = self.food.size.score_value();

            for _ in 0..growth {
                self.snake.try_grow_or_energy(max_len);
            }
            self.score += score;

            let new_seed = self.rng.next_u32() ^ self.frame_count;
            self.rng.seed(new_seed);
            self.food.respawn(&mut self.rng, &self.snake);
            self.play_eat_sound();
        }

        // Self-collision is now checked BEFORE movement (snake cannot pass through itself)
    }

    /// Handle game over
    fn on_game_over(&mut self) {
        self.state = GameState::GameOver;
        self.menu_selection = 0;

        let diff_idx = self.difficulty.to_index() as usize;
        if self.score > self.high_scores[diff_idx] {
            self.high_scores[diff_idx] = self.score;
            self.save_high_scores();
        }

        self.play_game_over_sound();
    }

    /// Get direction from p1 to p2 (with wrap-around)
    fn direction_to(p1: Point, p2: Point) -> Option<Direction> {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        // Handle wrap-around
        if dx == 1 || dx == -(GRID_SIZE - 1) {
            Some(Direction::Right)
        } else if dx == -1 || dx == GRID_SIZE - 1 {
            Some(Direction::Left)
        } else if dy == 1 || dy == -(GRID_SIZE - 1) {
            Some(Direction::Down)
        } else if dy == -1 || dy == GRID_SIZE - 1 {
            Some(Direction::Up)
        } else {
            None
        }
    }

    /// Draw a continuous body segment (only draw edges not connected to neighbors)
    fn draw_body_segment(
        &self,
        x: i32,
        y: i32,
        prev: Option<Point>,
        next: Option<Point>,
        current: Point,
    ) {
        // Check which neighbors exist
        let has_top = prev.is_some_and(|p| Self::direction_to(current, p) == Some(Direction::Up))
            || next.is_some_and(|p| Self::direction_to(current, p) == Some(Direction::Up));
        let has_bottom = prev
            .is_some_and(|p| Self::direction_to(current, p) == Some(Direction::Down))
            || next.is_some_and(|p| Self::direction_to(current, p) == Some(Direction::Down));
        let has_left = prev
            .is_some_and(|p| Self::direction_to(current, p) == Some(Direction::Left))
            || next.is_some_and(|p| Self::direction_to(current, p) == Some(Direction::Left));
        let has_right = prev
            .is_some_and(|p| Self::direction_to(current, p) == Some(Direction::Right))
            || next.is_some_and(|p| Self::direction_to(current, p) == Some(Direction::Right));

        // Only draw edges that don't connect to a neighbor
        if !has_top {
            hline(x, y, CELL_SIZE);
        }
        if !has_bottom {
            hline(x, y + CELL_SIZE as i32 - 1, CELL_SIZE);
        }
        if !has_left {
            vline(x, y, CELL_SIZE);
        }
        if !has_right {
            vline(x + CELL_SIZE as i32 - 1, y, CELL_SIZE);
        }
    }

    /// Draw pointed head (front pointed with blunt tip, back square)
    /// Creates a bullet-like shape: square on back, tapered front with blunt tip
    fn draw_pointed_head(&self, x: i32, y: i32, dir: Direction, fill_stroke: u16) {
        unsafe { *DRAW_COLORS = fill_stroke };
        // Draw based on direction - bullet shape with blunt tip
        match dir {
            Direction::Right => {
                // Bullet pointing right: square left, tapered right
                rect(x, y + 1, 5, 6); // Main body (left part)
                rect(x + 5, y + 2, 2, 4); // Taper
                rect(x + 7, y + 3, 1, 2); // Blunt tip
            }
            Direction::Left => {
                // Bullet pointing left: square right, tapered left
                rect(x + 3, y + 1, 5, 6); // Main body (right part)
                rect(x + 1, y + 2, 2, 4); // Taper
                rect(x, y + 3, 1, 2); // Blunt tip
            }
            Direction::Up => {
                // Bullet pointing up: square bottom, tapered top
                rect(x + 1, y + 3, 6, 5); // Main body (bottom part)
                rect(x + 2, y + 1, 4, 2); // Taper
                rect(x + 3, y, 2, 1); // Blunt tip
            }
            Direction::Down => {
                // Bullet pointing down: square top, tapered bottom
                rect(x + 1, y, 6, 5); // Main body (top part)
                rect(x + 2, y + 5, 4, 2); // Taper
                rect(x + 3, y + 7, 2, 1); // Blunt tip
            }
        }
    }

    /// Draw triangular tail pointing away from body
    /// Creates a triangle that tapers to a point in the direction of away_dir
    fn draw_tail(&self, x: i32, y: i32, away_dir: Direction) {
        // Draw triangle pointing in away_dir direction using rect calls
        match away_dir {
            Direction::Right => {
                // Triangle pointing right: wide on left, point on right
                rect(x, y + 1, 2, 6); // Base (wide)
                rect(x + 2, y + 2, 2, 4); // Middle
                rect(x + 4, y + 3, 2, 2); // Tip
            }
            Direction::Left => {
                // Triangle pointing left: wide on right, point on left
                rect(x + 6, y + 1, 2, 6); // Base (wide)
                rect(x + 4, y + 2, 2, 4); // Middle
                rect(x + 2, y + 3, 2, 2); // Tip
            }
            Direction::Down => {
                // Triangle pointing down: wide on top, point on bottom
                rect(x + 1, y, 6, 2); // Base (wide)
                rect(x + 2, y + 2, 4, 2); // Middle
                rect(x + 3, y + 4, 2, 2); // Tip
            }
            Direction::Up => {
                // Triangle pointing up: wide on bottom, point on top
                rect(x + 1, y + 6, 6, 2); // Base (wide)
                rect(x + 2, y + 4, 4, 2); // Middle
                rect(x + 3, y + 2, 2, 2); // Tip
            }
        }
    }

    /// Draw the game (snake, food, enemies, score)
    fn draw_game(&self) {
        // Draw player snake
        self.draw_snake(
            &self.snake.body,
            self.snake.length,
            self.snake.visual_direction,
            0x03, // body stroke (green)
            0x43, // head fill+stroke (yellow/green)
            true, // is_player (for boost/slow effects)
        );

        // Draw enemies
        self.draw_enemies();

        // Draw food (with size indication)
        self.draw_food();

        // Draw supply (if active)
        self.draw_supply();

        // Draw score
        self.draw_score();

        // Draw energy indicator (non-Classic only)
        self.draw_energy_indicator();

        // Draw collision flash effect (if active)
        self.draw_collision_flash();
    }

    /// Draw a snake with continuous body, pointed head, and triangular tail
    fn draw_snake(
        &self,
        body: &[Point],
        length: usize,
        direction: Direction,
        body_stroke: u16,
        head_fill_stroke: u16,
        is_player: bool,
    ) {
        if length == 0 {
            return;
        }

        // Draw body segments (from index 1 to length-2, excluding head and tail)
        for i in 1..length.saturating_sub(1) {
            let p = body[i];
            let x = p.x * CELL_SIZE as i32;
            let y = p.y * CELL_SIZE as i32;

            // Boost/slow flash effects for player
            let color = if is_player && self.boost_timer > 0 {
                if ((self.frame_count / 4) as usize + i) % 8 < 4 {
                    body_stroke
                } else {
                    0x04 // Yellow flash
                }
            } else {
                body_stroke
            };
            unsafe { *DRAW_COLORS = color };

            // Get previous and next segments for connectivity
            let prev = if i > 0 { Some(body[i - 1]) } else { None };
            let next = if i + 1 < length {
                Some(body[i + 1])
            } else {
                None
            };

            self.draw_body_segment(x, y, prev, next, p);
        }

        // Draw tail (last segment) as triangle
        if length > 1 {
            let tail_idx = length - 1;
            let tail = body[tail_idx];
            let prev = body[tail_idx - 1];
            let x = tail.x * CELL_SIZE as i32;
            let y = tail.y * CELL_SIZE as i32;

            // Direction tail points = opposite of direction from prev to tail
            let tail_dir = Self::direction_to(prev, tail).unwrap_or(Direction::Right);

            let color = if is_player && self.boost_timer > 0 {
                if ((self.frame_count / 4) as usize + tail_idx) % 8 < 4 {
                    body_stroke
                } else {
                    0x04
                }
            } else {
                body_stroke
            };
            unsafe { *DRAW_COLORS = color };
            self.draw_tail(x, y, tail_dir);
        }

        // Draw head (first segment) with pointed shape
        let head = body[0];
        let hx = head.x * CELL_SIZE as i32;
        let hy = head.y * CELL_SIZE as i32;

        // Head flash effects for player
        let head_color = if is_player {
            if self.slow_timer > 0 {
                if (self.frame_count / 6).is_multiple_of(2) {
                    head_fill_stroke
                } else {
                    0x23
                }
            } else if self.boost_timer > 0 {
                if (self.frame_count / 4) % 8 < 4 {
                    head_fill_stroke
                } else {
                    0x23
                }
            } else {
                head_fill_stroke
            }
        } else {
            head_fill_stroke
        };

        self.draw_pointed_head(hx, hy, direction, head_color);
    }

    /// Draw enemy snakes using the new continuous style
    fn draw_enemies(&self) {
        for enemy in &self.enemies.enemies {
            if !enemy.alive {
                continue;
            }

            // Colors based on color_index - all use visible colors (not 0x01 which is dark/background)
            let (body_stroke, head_fill_stroke) = match enemy.color_index {
                1 => (0x02, 0x21), // Purple stroke, Purple fill
                2 => (0x02, 0x42), // Purple stroke, Yellow fill
                _ => (0x02, 0x41), // Purple stroke, Yellow fill (was 0x01 = invisible)
            };

            self.draw_snake(
                &enemy.body,
                enemy.length,
                enemy.direction,
                body_stroke,
                head_fill_stroke,
                false, // not player
            );
        }
    }

    /// Draw food with size indication
    fn draw_food(&self) {
        let pos = self.food.position;
        let visual_size = self.food.size.visual_size();
        let offset = (CELL_SIZE - visual_size) / 2;

        // Color varies by size
        match self.food.size {
            FoodSize::Large => {
                unsafe { *DRAW_COLORS = 0x40 }; // Full yellow
            }
            FoodSize::Medium => {
                unsafe { *DRAW_COLORS = 0x30 }; // Green
            }
            FoodSize::Small => {
                unsafe { *DRAW_COLORS = 0x20 }; // Purple
            }
        }

        rect(
            pos.x * CELL_SIZE as i32 + offset as i32,
            pos.y * CELL_SIZE as i32 + offset as i32,
            visual_size,
            visual_size,
        );
    }

    /// Draw supply pack with blinking effect (yellow/green alternating)
    fn draw_supply(&self) {
        if !self.supply.active {
            return;
        }

        let pos = self.supply.position;
        let phase = self.supply.flash_phase(self.frame_count);

        // Alternate between yellow and green
        if phase == 0 {
            unsafe { *DRAW_COLORS = 0x40 }; // Yellow
        } else {
            unsafe { *DRAW_COLORS = 0x30 }; // Green
        }

        // Draw as a small diamond shape (4x4 centered in cell)
        let cx = pos.x * CELL_SIZE as i32 + 4;
        let cy = pos.y * CELL_SIZE as i32 + 4;

        // Draw diamond pattern
        rect(cx - 1, cy - 2, 2, 1); // Top
        rect(cx - 2, cy - 1, 4, 2); // Middle
        rect(cx - 1, cy + 1, 2, 1); // Bottom
    }

    /// Draw the score in top-left corner
    fn draw_score(&self) {
        unsafe { *DRAW_COLORS = 0x04 };

        let mut buf = [0u8; 12];
        let score_str = self.format_number(self.score, &mut buf);
        text(score_str, 2, 2);
    }

    /// Draw energy indicator in top-right corner (only in non-Classic mode)
    fn draw_energy_indicator(&self) {
        if self.difficulty == Difficulty::Classic {
            return; // No energy in Classic mode
        }

        use crate::snake::MAX_ENERGY;

        // Draw energy bar from right to left
        for i in 0..MAX_ENERGY {
            if i < self.snake.energy {
                unsafe { *DRAW_COLORS = 0x03 }; // Green for filled
            } else {
                unsafe { *DRAW_COLORS = 0x01 }; // Dark for empty
            }
            rect(157 - i as i32 * 3, 2, 2, 4);
        }
    }

    /// Draw main menu
    fn draw_main_menu(&self) {
        menu::draw_main_menu(self.menu_selection, self.frame_count);
    }

    /// Draw settings menu
    fn draw_settings_menu(&self) {
        menu::draw_settings_menu(
            self.menu_selection,
            self.music_enabled,
            self.sfx_enabled,
            self.cheat_enabled,
        );
    }

    /// Draw difficulty select
    fn draw_difficulty_select(&self) {
        menu::draw_difficulty_select(self.menu_selection, &self.high_scores);
    }

    /// Draw game over screen
    fn draw_game_over(&self) {
        let diff_idx = self.difficulty.to_index() as usize;
        let high_score = self.high_scores[diff_idx];
        menu::draw_game_over_menu(self.score, high_score, self.menu_selection);
    }

    /// Format a number into a byte buffer
    fn format_number<'a>(&self, mut n: u32, buf: &'a mut [u8]) -> &'a [u8] {
        if n == 0 {
            buf[0] = b'0';
            return &buf[..1];
        }

        let mut i = 0;
        while n > 0 && i < buf.len() {
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
        }

        buf[..i].reverse();
        &buf[..i]
    }

    /// Play a sound when eating food
    fn play_eat_sound(&self) {
        if self.sfx_enabled {
            tone(440 | (880 << 16), 5, 80, TONE_PULSE1);
        }
    }

    /// Play a sound when collecting supply
    fn play_supply_sound(&self) {
        if self.sfx_enabled {
            // Higher pitched, sparkly sound for energy pickup
            tone(880 | (1320 << 16), 8, 60, TONE_PULSE1);
        }
    }

    /// Play a sound when activating boost
    fn play_boost_sound(&self) {
        if self.sfx_enabled {
            // Rising whoosh sound for speed boost
            tone(330 | (880 << 16), 15, 70, TONE_NOISE);
        }
    }

    /// Play a sound when activating slow
    fn play_slow_sound(&self) {
        if self.sfx_enabled {
            // Descending sound for slowdown
            tone(440 | (220 << 16), 15, 50, TONE_TRIANGLE);
        }
    }

    /// Head-to-head collision sound - both snakes take damage
    fn play_head_clash_sound(&self) {
        if self.sfx_enabled {
            // Low clash sound with slight rise
            tone(200 | (300 << 16), 6 | (4 << 8), 70, TONE_NOISE);
        }
    }

    /// Player hurt sound - hit by enemy
    fn play_player_hurt_sound(&self) {
        if self.sfx_enabled {
            // Descending tone indicates damage taken
            tone(330 | (110 << 16), 5 | (10 << 8), 60, TONE_TRIANGLE);
        }
    }

    /// Enemy hurt sound - successful defense/counterattack
    fn play_enemy_hurt_sound(&self) {
        if self.sfx_enabled {
            // Rising tone indicates successful hit
            tone(220 | (440 << 16), 3 | (5 << 8), 50, TONE_PULSE1);
        }
    }

    /// Self-hurt sound - player hit their own body
    fn play_self_hurt_sound(&self) {
        if self.sfx_enabled {
            // Low triangle wave for self-damage
            tone(220 | (110 << 16), 8, 50, TONE_TRIANGLE);
        }
    }

    /// Head-kill sound - killed enemy via head-to-head collision
    fn play_head_kill_sound(&self) {
        if self.sfx_enabled {
            // Dramatic rising sound for head-to-head kill
            tone(
                220 | (880 << 16),
                10 | (8 << 8),
                80,
                TONE_PULSE1 | TONE_MODE2,
            );
        }
    }

    /// Trigger collision flash effect at the given position
    fn trigger_collision_flash(&mut self, pos: Point) {
        self.collision_flash_pos = Some(pos);
        self.collision_flash_timer = 12; // ~0.2 seconds at 60 FPS
    }

    /// Draw collision flash effect (cross shape)
    fn draw_collision_flash(&self) {
        if let Some(pos) = self.collision_flash_pos {
            // Blink every 3 frames
            if (self.collision_flash_timer / 3).is_multiple_of(2) {
                unsafe { *DRAW_COLORS = 0x40 }; // Yellow (color 4)
                let cx = pos.x * CELL_SIZE as i32 + 4;
                let cy = pos.y * CELL_SIZE as i32 + 4;
                // Cross flash
                hline(cx - 3, cy, 7);
                vline(cx, cy - 3, 7);
            }
        }
    }

    /// Play a sound on game over
    fn play_game_over_sound(&self) {
        if self.sfx_enabled {
            tone(440 | (110 << 16), 60, 60, TONE_TRIANGLE);
        }
    }

    /// Play a sound for menu navigation
    fn play_menu_sound(&self) {
        if self.sfx_enabled {
            tone(660, 3, 50, TONE_PULSE1);
        }
    }

    /// Play a sound when starting game
    fn play_start_sound(&self) {
        if self.sfx_enabled {
            tone(330 | (660 << 16), 10, 70, TONE_PULSE1);
        }
    }

    /// Play background music (faster when boosted)
    fn play_music(&mut self) {
        if !self.music_enabled {
            return;
        }

        // When boosted: double tempo (half interval) and higher pitch
        let (interval, freq_mult) = if self.boost_timer > 0 {
            (MUSIC_INTERVAL / 2, 2u32)
        } else {
            (MUSIC_INTERVAL, 1u32)
        };

        if !self.frame_count.is_multiple_of(interval) {
            return;
        }

        let freq = MELODY[self.music_index];
        if freq > 0 {
            tone(freq * freq_mult, interval - 2, 30, TONE_PULSE2);
        }

        self.music_index = (self.music_index + 1) % MELODY.len();
    }

    /// Play menu background music (slower, softer version)
    fn play_menu_music(&mut self) {
        if !self.music_enabled {
            return;
        }

        // Slower tempo for menu (16 frames between notes)
        let interval = MUSIC_INTERVAL * 2;

        if !self.frame_count.is_multiple_of(interval) {
            return;
        }

        let freq = MELODY[self.music_index];
        if freq > 0 {
            // Lower octave and softer volume for ambient feel
            tone(freq / 2, interval - 4, 20, TONE_TRIANGLE);
        }

        self.music_index = (self.music_index + 1) % MELODY.len();
    }

    /// Load high scores and settings from persistent storage
    /// Format v3: [SNAK][ver][scores x5][flags][checksum] = 27 bytes
    /// Flags: bit0=music, bit1=sfx, bit2=cheat
    fn load_high_scores(&mut self) {
        let mut buf = [0u8; 27];
        let read = unsafe { diskr(buf.as_mut_ptr(), 27) };

        if read >= 26 && &buf[0..4] == b"SNAK" {
            let version = buf[4];

            if version == 1 {
                // Legacy format: no sound settings
                let checksum = buf[5..25].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
                if checksum == buf[25] {
                    for i in 0..5 {
                        let offset = 5 + i * 4;
                        self.high_scores[i] = u32::from_le_bytes([
                            buf[offset],
                            buf[offset + 1],
                            buf[offset + 2],
                            buf[offset + 3],
                        ]);
                    }
                }
            } else if (version == 2 || version == 3) && read >= 27 {
                // v2/v3 format with settings
                let checksum = buf[5..26].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
                if checksum == buf[26] {
                    for i in 0..5 {
                        let offset = 5 + i * 4;
                        self.high_scores[i] = u32::from_le_bytes([
                            buf[offset],
                            buf[offset + 1],
                            buf[offset + 2],
                            buf[offset + 3],
                        ]);
                    }
                    // Load settings from flags byte
                    let flags = buf[25];
                    self.music_enabled = (flags & 0x01) != 0;
                    self.sfx_enabled = (flags & 0x02) != 0;
                    self.cheat_enabled = (flags & 0x04) != 0;
                }
            }
        }
    }

    /// Save high scores and settings to persistent storage
    /// Format v3: [SNAK][ver][scores x5][flags][checksum] = 27 bytes
    fn save_high_scores(&self) {
        self.save_data();
    }

    /// Save settings only (without updating high scores)
    fn save_settings(&self) {
        self.save_data();
    }

    /// Internal: save all data to persistent storage
    fn save_data(&self) {
        let mut buf = [0u8; 27];

        buf[0..4].copy_from_slice(b"SNAK");
        buf[4] = 3; // Version 3

        for i in 0..5 {
            let offset = 5 + i * 4;
            let bytes = self.high_scores[i].to_le_bytes();
            buf[offset..offset + 4].copy_from_slice(&bytes);
        }

        // Settings flags: bit0=music, bit1=sfx, bit2=cheat
        let mut flags: u8 = 0;
        if self.music_enabled {
            flags |= 0x01;
        }
        if self.sfx_enabled {
            flags |= 0x02;
        }
        if self.cheat_enabled {
            flags |= 0x04;
        }
        buf[25] = flags;

        // Checksum covers bytes 5-25 (scores + flags)
        buf[26] = buf[5..26].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

        unsafe { diskw(buf.as_ptr(), 27) };
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = GameState::MainMenu;
        assert_eq!(state, GameState::MainMenu);
    }

    #[test]
    fn test_game_state_transitions() {
        assert_ne!(GameState::MainMenu, GameState::Playing);
        assert_ne!(GameState::Playing, GameState::GameOver);
        assert_ne!(GameState::MainMenu, GameState::GameOver);
    }

    #[test]
    fn test_difficulty_values() {
        assert_eq!(Difficulty::Classic.max_enemies(), 0);
        assert_eq!(Difficulty::Noob.max_enemies(), 2);
        assert_eq!(Difficulty::Normal.max_enemies(), 3);
        assert_eq!(Difficulty::Hell.max_enemies(), 5);
        assert_eq!(Difficulty::Nightmare.max_enemies(), 8);
    }

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(Difficulty::from_index(0), Difficulty::Classic);
        assert_eq!(Difficulty::from_index(1), Difficulty::Noob);
        assert_eq!(Difficulty::from_index(2), Difficulty::Normal);
        assert_eq!(Difficulty::from_index(3), Difficulty::Hell);
        assert_eq!(Difficulty::from_index(4), Difficulty::Nightmare);
        assert_eq!(Difficulty::from_index(99), Difficulty::Nightmare);
    }

    #[test]
    fn test_difficulty_to_index() {
        assert_eq!(Difficulty::Classic.to_index(), 0);
        assert_eq!(Difficulty::Noob.to_index(), 1);
        assert_eq!(Difficulty::Normal.to_index(), 2);
        assert_eq!(Difficulty::Hell.to_index(), 3);
        assert_eq!(Difficulty::Nightmare.to_index(), 4);
    }

    #[test]
    fn test_cell_size_grid_alignment() {
        use crate::snake::GRID_SIZE;
        assert_eq!(CELL_SIZE as i32 * GRID_SIZE, 160);
    }

    #[test]
    fn test_difficulty_max_snake_length() {
        // Classic has no practical limit (uses array max)
        assert_eq!(Difficulty::Classic.max_snake_length(), 50);
        // Battle modes have decreasing limits
        assert_eq!(Difficulty::Noob.max_snake_length(), 20);
        assert_eq!(Difficulty::Normal.max_snake_length(), 18);
        assert_eq!(Difficulty::Hell.max_snake_length(), 15);
        assert_eq!(Difficulty::Nightmare.max_snake_length(), 12);
    }

    #[test]
    fn test_difficulty_enemies_use_energy() {
        // Classic: no energy cost for speed
        assert!(!Difficulty::Classic.enemies_use_energy());
        // Battle modes: enemies use energy
        assert!(Difficulty::Noob.enemies_use_energy());
        assert!(Difficulty::Normal.enemies_use_energy());
        assert!(Difficulty::Hell.enemies_use_energy());
        assert!(Difficulty::Nightmare.enemies_use_energy());
    }
}
