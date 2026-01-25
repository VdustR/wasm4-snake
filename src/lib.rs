// Only use no_std for WASM target
#![cfg_attr(target_arch = "wasm32", no_std)]

#[cfg(feature = "buddy-alloc")]
#[cfg(target_arch = "wasm32")]
mod alloc;

// Panic handler for WASM (no_std requires this)
#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    // In WASM-4, we just loop forever on panic
    loop {}
}

mod food;
mod rng;
mod snake;
mod supply;

// WASM-4 specific modules (only for WASM target)
#[cfg(target_arch = "wasm32")]
mod ai;
#[cfg(target_arch = "wasm32")]
mod enemy;
#[cfg(target_arch = "wasm32")]
mod game;
#[cfg(target_arch = "wasm32")]
mod menu;
#[cfg(target_arch = "wasm32")]
mod wasm4;

#[cfg(target_arch = "wasm32")]
use game::Game;

/// Global game instance
#[cfg(target_arch = "wasm32")]
static mut GAME: Option<Game> = None;

/// Called once at startup
#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[allow(static_mut_refs)]
fn start() {
    unsafe {
        GAME = Some(Game::new());
    }
}

/// Called every frame (60 times per second)
#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[allow(static_mut_refs)]
fn update() {
    unsafe {
        if let Some(game) = GAME.as_mut() {
            game.update();
        }
    }
}

// Re-export for tests
#[cfg(test)]
pub use snake::{Direction, Point, Snake, GRID_SIZE, MAX_SNAKE_LENGTH};
