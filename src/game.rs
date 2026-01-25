use crate::ai::PathFinder;
use crate::enemy::{EnemyAIState, EnemyManager, MAX_ENEMIES};
use crate::food::{Food, FoodSize};
use crate::menu;
use crate::rng::Rng;
use crate::snake::{Direction, Point, Snake, GRID_SIZE};
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
    // Boost/Slow timers (non-Classic only)
    boost_timer: u16,    // Remaining boost frames
    boost_cooldown: u16, // Cooldown until next boost
    slow_timer: u16,     // Remaining slow frames
    slow_cooldown: u16,  // Cooldown until next slow
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
            boost_timer: 0,
            boost_cooldown: 0,
            slow_timer: 0,
            slow_cooldown: 0,
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
                self.draw_main_menu();
            }
            GameState::DifficultySelect => {
                self.draw_difficulty_select();
            }
            GameState::Playing => {
                // Update boost/slow timers (non-Classic only)
                if self.difficulty != Difficulty::Classic {
                    self.update_ability_timers();
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

                // Check if should flee (player is close and longer)
                let should_flee = dist_to_player < 4
                    && player_length > enemy.length
                    && self.rng.range(0, 100) < escape_intelligence as i32;

                // Check if should grab supply
                let should_grab_supply = supply_active
                    && dist_to_supply < 8
                    && self.rng.range(0, 100) < supply_aggression as i32;

                // Decide AI state
                if should_flee {
                    enemy.ai_state = EnemyAIState::Fleeing;
                } else if should_grab_supply {
                    enemy.ai_state = EnemyAIState::GrabbingSupply;
                } else {
                    let roll = self.rng.range(0, 100) as u8;
                    if roll < chase_ratio {
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
                        EnemyAIState::Chasing => {
                            dist_to_player < (ai_intelligence as i32 + 3)
                                && enemy.length >= player_length
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

                // Decide if should use slow (for precise control near targets)
                let slowdown_tendency = difficulty.slowdown_tendency();
                if enemies_use_energy
                    && enemy.slow_timer == 0
                    && enemy.boost_timer == 0
                    && self.rng.range(0, 100) < slowdown_tendency as i32
                {
                    let dist_to_food =
                        (enemy_head.x - food_pos.x).abs() + (enemy_head.y - food_pos.y).abs();

                    // Slow down when very close to food or supply for precise control
                    let should_slow = (dist_to_food <= 2) || (dist_to_supply <= 2);
                    if should_slow {
                        enemy.slow_timer = BOOST_DURATION / 2; // Shorter slow duration for AI
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

                enemy.update();
            }
        }

        // Check collisions between enemies (with new shrink rules)
        self.check_enemy_collisions_with_shrink();

        // Check if enemies ate food
        let max_len = self.difficulty.max_snake_length();
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

        // Check player collision with enemies
        self.check_player_enemy_collisions();
    }

    /// Check collisions between enemies with shrink rules
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

                let head_i = self.enemies.enemies[i].head();
                let head_j = self.enemies.enemies[j].head();

                // Head-to-head collision
                if head_i == head_j {
                    // Both shrink
                    if !self.enemies.enemies[i].shrink() {
                        self.enemies.kill(i);
                    }
                    if !self.enemies.enemies[j].shrink() {
                        self.enemies.kill(j);
                    }
                    continue;
                }

                // Check if j's head hits i's body (only if both still alive with body)
                if self.enemies.enemies[i].length > 1
                    && self.enemies.enemies[i].body[1..self.enemies.enemies[i].length]
                        .contains(&head_j)
                {
                    // j's head hit i's body: i grows, j shrinks
                    self.enemies.enemies[i].try_grow_or_energy(max_len);
                    if !self.enemies.enemies[j].shrink() {
                        self.enemies.kill(j);
                    }
                }

                // Check if i's head hits j's body (only if both still alive with body)
                if self.enemies.enemies[j].length > 1
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

    /// Check player-enemy collisions with mode-specific rules
    fn check_player_enemy_collisions(&mut self) {
        let player_head = self.snake.head();
        let max_len = self.difficulty.max_snake_length();
        let is_classic = self.difficulty == Difficulty::Classic;

        for i in 0..MAX_ENEMIES {
            if !self.enemies.enemies[i].alive {
                continue;
            }

            let enemy_head = self.enemies.enemies[i].head();

            // Player head hits enemy (head or body)
            if self.enemies.enemies[i].contains(player_head) {
                if is_classic {
                    // Classic mode: instant death
                    self.on_game_over();
                    return;
                } else {
                    // Other modes: player shrinks, enemy grows
                    self.enemies.enemies[i].try_grow_or_energy(max_len);
                    if !self.snake.shrink() {
                        self.on_game_over();
                        return;
                    }
                    self.score = self.score.saturating_sub(10);
                }
            }

            // Enemy head hits player body (not head)
            if self.snake.body[1..self.snake.length].contains(&enemy_head) {
                if is_classic {
                    // Classic mode: enemy dies
                    self.enemies.kill(i);
                    self.score += 50;
                } else {
                    // Other modes: enemy shrinks, player grows
                    self.snake.try_grow_or_energy(max_len);
                    if !self.enemies.enemies[i].shrink() {
                        self.enemies.kill(i);
                    }
                    self.score += 10;
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
                if just_pressed & BUTTON_1 != 0 {
                    self.state = GameState::DifficultySelect;
                    self.menu_selection = self.difficulty.to_index();
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

                // Boost/Slow abilities (non-Classic only)
                if self.difficulty != Difficulty::Classic {
                    // BUTTON_1 (X) = Boost: costs 1 energy, 2s duration, 5s cooldown
                    if just_pressed & BUTTON_1 != 0
                        && self.boost_cooldown == 0
                        && self.boost_timer == 0
                        && self.slow_timer == 0
                        && self.snake.use_energy()
                    {
                        self.boost_timer = BOOST_DURATION;
                        self.play_boost_sound();
                    }

                    // BUTTON_2 (Z) = Slow: free, 2s duration, 5s cooldown, cancels boost
                    if just_pressed & BUTTON_2 != 0
                        && self.slow_cooldown == 0
                        && self.slow_timer == 0
                    {
                        // Cancel any active boost
                        if self.boost_timer > 0 {
                            self.boost_timer = 0;
                            self.boost_cooldown = ABILITY_COOLDOWN;
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
            if self.boost_timer == 0 {
                self.boost_cooldown = ABILITY_COOLDOWN;
            }
        }

        // Update boost cooldown
        if self.boost_cooldown > 0 {
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
        self.snake.update();

        // Check if snake ate food
        if self.snake.head() == self.food.position {
            let growth = self.food.size.growth_amount();
            let score = self.food.size.score_value();
            let max_len = self.difficulty.max_snake_length();

            for _ in 0..growth {
                self.snake.try_grow_or_energy(max_len);
            }
            self.score += score;

            let new_seed = self.rng.next_u32() ^ self.frame_count;
            self.rng.seed(new_seed);
            self.food.respawn(&mut self.rng, &self.snake);
            self.play_eat_sound();
        }

        // Check self collision
        if self.snake.collides_with_self() {
            // Classic mode: instant death
            // Other modes: shrink (death only at min length)
            if self.difficulty == Difficulty::Classic || !self.snake.shrink() {
                self.on_game_over();
            }
        }
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

    /// Draw the game (snake, food, enemies, score)
    fn draw_game(&self) {
        // Draw player snake body with boost/slow flash effects
        for i in 1..self.snake.length {
            let p = self.snake.body[i];

            // Boost: wave flash effect (head to tail)
            let body_visible = if self.boost_timer > 0 {
                // Wave pattern: (frame/4 + index) % 8 < 4
                ((self.frame_count / 4) as usize + i) % 8 < 4
            } else {
                true
            };

            if body_visible {
                unsafe { *DRAW_COLORS = 0x32 }; // Fill=3, Stroke=2
            } else {
                unsafe { *DRAW_COLORS = 0x42 }; // Brighter flash (yellow)
            }

            rect(
                p.x * CELL_SIZE as i32,
                p.y * CELL_SIZE as i32,
                CELL_SIZE,
                CELL_SIZE,
            );
        }

        // Draw player snake head
        let head = self.snake.head();

        // Slow: head-only flash effect
        let head_bright = if self.slow_timer > 0 {
            (self.frame_count / 6).is_multiple_of(2)
        } else if self.boost_timer > 0 {
            // During boost, head also flashes
            (self.frame_count / 4) % 8 < 4
        } else {
            true
        };

        if head_bright {
            unsafe { *DRAW_COLORS = 0x43 }; // Fill=4, Stroke=3
        } else {
            unsafe { *DRAW_COLORS = 0x23 }; // Dimmer flash
        }

        rect(
            head.x * CELL_SIZE as i32,
            head.y * CELL_SIZE as i32,
            CELL_SIZE,
            CELL_SIZE,
        );

        // Draw enemies
        self.draw_enemies();

        // Draw food (with size indication)
        self.draw_food();

        // Draw supply (if active)
        self.draw_supply();

        // Draw score
        self.draw_score();

        // Draw speed indicator
        self.draw_speed_indicator();

        // Draw energy indicator (non-Classic only)
        self.draw_energy_indicator();
    }

    /// Draw enemy snakes
    fn draw_enemies(&self) {
        for enemy in &self.enemies.enemies {
            if !enemy.alive {
                continue;
            }

            // Draw enemy body with different color based on color_index
            let (body_color, head_color) = match enemy.color_index {
                1 => (0x21, 0x12), // Purple tones
                2 => (0x21, 0x41), // Purple/Yellow
                _ => (0x12, 0x42), // Different combo
            };

            for i in 1..enemy.length {
                let p = enemy.body[i];
                unsafe { *DRAW_COLORS = body_color };
                rect(
                    p.x * CELL_SIZE as i32,
                    p.y * CELL_SIZE as i32,
                    CELL_SIZE,
                    CELL_SIZE,
                );
            }

            // Draw enemy head
            let head = enemy.head();
            unsafe { *DRAW_COLORS = head_color };
            rect(
                head.x * CELL_SIZE as i32,
                head.y * CELL_SIZE as i32,
                CELL_SIZE,
                CELL_SIZE,
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

    /// Draw speed indicator in top-right corner
    fn draw_speed_indicator(&self) {
        let speed_level = if self.move_interval <= 7 {
            5
        } else if self.move_interval <= 11 {
            4
        } else if self.move_interval <= 15 {
            3
        } else if self.move_interval <= 22 {
            2
        } else {
            1
        };

        for i in 0..5 {
            if i < speed_level {
                unsafe { *DRAW_COLORS = 0x04 };
            } else {
                unsafe { *DRAW_COLORS = 0x02 };
            }
            rect(145 + i * 3, 2, 2, 6);
        }
    }

    /// Draw energy indicator below speed indicator (only in non-Classic mode)
    fn draw_energy_indicator(&self) {
        if self.difficulty == Difficulty::Classic {
            return; // No energy in Classic mode
        }

        use crate::snake::MAX_ENERGY;

        // Draw energy bar (10 segments max)
        for i in 0..MAX_ENERGY {
            if i < self.snake.energy {
                unsafe { *DRAW_COLORS = 0x03 }; // Green for filled
            } else {
                unsafe { *DRAW_COLORS = 0x01 }; // Dark for empty
            }
            rect(145 + i as i32 * 3, 10, 2, 4);
        }
    }

    /// Draw main menu
    fn draw_main_menu(&self) {
        menu::draw_main_menu();

        if (self.blink_timer / 30).is_multiple_of(2) {
            unsafe { *DRAW_COLORS = 0x03 };
        } else {
            unsafe { *DRAW_COLORS = 0x02 };
        }
        text(b"Press X to Start", 24, 100);
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

    /// Load high scores and settings from persistent storage
    /// Format v2: [SNAK][ver][scores x5][flags][checksum] = 27 bytes
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
            } else if version == 2 && read >= 27 {
                // New format with sound settings
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
                    // Load sound settings from flags byte
                    let flags = buf[25];
                    self.music_enabled = (flags & 0x01) != 0;
                    self.sfx_enabled = (flags & 0x02) != 0;
                }
            }
        }
    }

    /// Save high scores and settings to persistent storage
    /// Format v2: [SNAK][ver][scores x5][flags][checksum] = 27 bytes
    fn save_high_scores(&self) {
        let mut buf = [0u8; 27];

        buf[0..4].copy_from_slice(b"SNAK");
        buf[4] = 2; // Version 2

        for i in 0..5 {
            let offset = 5 + i * 4;
            let bytes = self.high_scores[i].to_le_bytes();
            buf[offset..offset + 4].copy_from_slice(&bytes);
        }

        // Sound settings flags
        let mut flags: u8 = 0;
        if self.music_enabled {
            flags |= 0x01;
        }
        if self.sfx_enabled {
            flags |= 0x02;
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
