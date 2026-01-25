use crate::ai::PathFinder;
use crate::enemy::{EnemyAIState, EnemyManager, MAX_ENEMIES};
use crate::food::{Food, FoodSize};
use crate::menu;
use crate::rng::Rng;
use crate::snake::{Direction, Point, Snake};
use crate::wasm4::*;

/// Size of each cell in pixels
const CELL_SIZE: u32 = 8;
/// Base frames between snake movements (60 FPS / 15 = 4 moves/sec)
const BASE_MOVE_INTERVAL: u32 = 15;
/// Frames between music notes (60 FPS / 8 = 7.5 notes/sec)
const MUSIC_INTERVAL: u32 = 8;
/// Minimum speed (max frames per move)
const MIN_SPEED: u32 = 30;
/// Maximum speed (min frames per move)
const MAX_SPEED: u32 = 5;
/// Speed change amount per button press
const SPEED_STEP: u32 = 2;
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
    Paused,
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
    pub const fn to_index(&self) -> u8 {
        *self as u8
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
}

/// Main game struct
pub struct Game {
    snake: Snake,
    food: Food,
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
        };

        game.load_high_scores();
        game
    }

    /// Reset game state for a new round
    fn reset_game(&mut self) {
        self.snake = Snake::new();
        self.food.respawn(&mut self.rng, &self.snake);
        self.enemies.reset();
        self.score = 0;
        self.move_interval = BASE_MOVE_INTERVAL;
        self.music_index = 0;
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
                // Update player snake
                if self.frame_count % self.move_interval == 0 {
                    self.update_game_logic();
                }

                // Update enemies
                self.update_enemies();

                // Play background music
                self.play_music();
                self.draw_game();
            }
            GameState::Paused => {
                self.draw_game();
                self.draw_pause_menu();
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
        let food_pos = self.food.position;
        let player_head = self.snake.head();

        let enemies_use_energy = difficulty.enemies_use_energy();

        // Update each enemy
        for i in 0..MAX_ENEMIES {
            let enemy = &mut self.enemies.enemies[i];
            if !enemy.alive {
                continue;
            }

            // Update move timer
            enemy.move_timer = enemy.move_timer.wrapping_add(1);

            // AI decision making
            enemy.decision_timer = enemy.decision_timer.wrapping_add(1);
            if enemy.decision_timer >= AI_DECISION_INTERVAL {
                enemy.decision_timer = 0;

                // Decide behavior based on difficulty
                let roll = self.rng.range(0, 100) as u8;
                if roll < chase_ratio {
                    enemy.ai_state = EnemyAIState::Chasing;
                } else {
                    enemy.ai_state = EnemyAIState::Seeking;
                }
            }

            // Check if enemy should use energy for speed boost
            let mut use_energy_boost = false;
            if enemies_use_energy && enemy.energy > 0 {
                // Use energy when chasing and player is close, or when in danger
                if enemy.ai_state == EnemyAIState::Chasing {
                    let dist = (enemy.head().x - player_head.x).abs()
                        + (enemy.head().y - player_head.y).abs();
                    // Use energy boost when close to player (based on difficulty intelligence)
                    if dist < (ai_intelligence as i32 + 3) {
                        use_energy_boost = enemy.use_energy();
                    }
                }
            }

            // Move enemy at its speed (faster if using energy)
            let effective_speed = if use_energy_boost {
                enemy_speed / 2 // Double speed when using energy
            } else {
                enemy_speed
            };

            if enemy.move_timer >= effective_speed {
                enemy.move_timer = 0;

                // Get target based on AI state
                let target = match enemy.ai_state {
                    EnemyAIState::Chasing => player_head,
                    EnemyAIState::Seeking | EnemyAIState::Idle => food_pos,
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

                // Speed controls (speed up consumes energy)
                if just_pressed & BUTTON_1 != 0 {
                    // Speed up requires energy (except in Classic mode which has free speed control)
                    let can_speed_up = if self.difficulty == Difficulty::Classic {
                        true
                    } else {
                        self.snake.use_energy()
                    };

                    if can_speed_up && self.move_interval > MAX_SPEED {
                        self.move_interval = self.move_interval.saturating_sub(SPEED_STEP);
                        if self.move_interval < MAX_SPEED {
                            self.move_interval = MAX_SPEED;
                        }
                    }
                } else if just_pressed & BUTTON_2 != 0 {
                    // Speed down is always free
                    if self.move_interval < MIN_SPEED {
                        self.move_interval = self.move_interval.saturating_add(SPEED_STEP);
                        if self.move_interval > MIN_SPEED {
                            self.move_interval = MIN_SPEED;
                        }
                    }
                }

                // Pause with both buttons held
                if gamepad & (BUTTON_1 | BUTTON_2) == (BUTTON_1 | BUTTON_2) {
                    if self.prev_gamepad & (BUTTON_1 | BUTTON_2) != (BUTTON_1 | BUTTON_2) {
                        self.state = GameState::Paused;
                        self.menu_selection = 0;
                        self.play_menu_sound();
                    }
                }
            }
            GameState::Paused => {
                if just_pressed & BUTTON_UP != 0 {
                    if self.menu_selection > 0 {
                        self.menu_selection -= 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_DOWN != 0 {
                    if self.menu_selection < 3 {
                        self.menu_selection += 1;
                        self.play_menu_sound();
                    }
                } else if just_pressed & BUTTON_1 != 0 {
                    match self.menu_selection {
                        0 => self.state = GameState::Playing, // Continue
                        1 => {
                            self.music_enabled = !self.music_enabled;
                            self.save_high_scores(); // Save settings
                        }
                        2 => {
                            self.sfx_enabled = !self.sfx_enabled;
                            self.save_high_scores(); // Save settings
                        }
                        _ => self.state = GameState::MainMenu, // Quit
                    }
                    self.play_menu_sound();
                } else if just_pressed & BUTTON_2 != 0 {
                    self.state = GameState::Playing;
                    self.play_menu_sound();
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
            if self.difficulty == Difficulty::Classic {
                self.on_game_over();
            } else if !self.snake.shrink() {
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
        // Draw player snake body
        for i in 1..self.snake.length {
            let p = self.snake.body[i];
            unsafe { *DRAW_COLORS = 0x32 }; // Fill=3, Stroke=2
            rect(
                p.x * CELL_SIZE as i32,
                p.y * CELL_SIZE as i32,
                CELL_SIZE,
                CELL_SIZE,
            );
        }

        // Draw player snake head
        let head = self.snake.head();
        unsafe { *DRAW_COLORS = 0x43 }; // Fill=4, Stroke=3
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
            rect(145 + i as i32 * 3, 2, 2, 6);
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

        if (self.blink_timer / 30) % 2 == 0 {
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

    /// Draw pause menu
    fn draw_pause_menu(&self) {
        menu::draw_pause_menu(self.menu_selection, self.music_enabled, self.sfx_enabled);
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

    /// Play background music
    fn play_music(&mut self) {
        if !self.music_enabled {
            return;
        }
        if self.frame_count % MUSIC_INTERVAL != 0 {
            return;
        }

        let freq = MELODY[self.music_index];
        if freq > 0 {
            tone(freq, MUSIC_INTERVAL - 2, 30, TONE_PULSE2);
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
        assert_ne!(GameState::Playing, GameState::Paused);
        assert_ne!(GameState::Paused, GameState::GameOver);
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
