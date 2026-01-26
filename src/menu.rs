use crate::wasm4::*;

/// Draw the main menu screen with animated snake
pub fn draw_main_menu(selected: u8, frame: u32) {
    // Title
    unsafe { *DRAW_COLORS = 0x04 }; // Yellow
    text(b"SNAKE", 56, 20);

    // Draw animated snake eating food
    draw_menu_animation(frame);

    // Menu options
    let options: [&[u8]; 2] = [b"Start", b"Settings"];
    for (i, option) in options.iter().enumerate() {
        let y = 95 + i as i32 * 16;

        // Selection indicator
        if i as u8 == selected {
            unsafe { *DRAW_COLORS = 0x04 }; // Yellow highlight
            text(b">", 48, y);
        }

        // Option text
        if i as u8 == selected {
            unsafe { *DRAW_COLORS = 0x04 }; // Yellow for selected
        } else {
            unsafe { *DRAW_COLORS = 0x03 }; // Green for others
        }
        text(*option, 60, y);
    }

    // Controls hint
    unsafe { *DRAW_COLORS = 0x02 };
    text(b"X: Select", 52, 145);
}

/// Draw animated snake eating food on menu
fn draw_menu_animation(frame: u32) {
    // Animation cycle: snake moves right, eats food, pauses, resets
    // 90 frames per cycle (~1.5 seconds)
    let cycle_frame = frame % 90;

    // Snake head position (moves from left to right)
    let base_x: i32 = 48;
    let y: i32 = 55;
    let food_x: i32 = 96;

    // Phase 1 (0-50): Snake approaches food
    // Phase 2 (50-60): Eating pause (snake at food position)
    // Phase 3 (60-90): Reset animation
    let head_x = if cycle_frame < 50 {
        // Moving towards food
        base_x + (cycle_frame as i32 * (food_x - base_x) / 50)
    } else if cycle_frame < 60 {
        // Eating pause - stay at food position
        food_x
    } else {
        // Reset phase - quick return
        let reset_progress = (cycle_frame - 60) as i32;
        food_x - (reset_progress * (food_x - base_x) / 30)
    };

    // Food visible: only in approach phase before snake reaches it
    let food_visible = cycle_frame < 45;

    // Draw food (small square with blink when about to be eaten)
    if food_visible {
        let blink = cycle_frame > 40 && (frame / 3).is_multiple_of(2);
        if !blink {
            unsafe { *DRAW_COLORS = 0x40 }; // Yellow
            rect(food_x + 2, y + 2, 5, 5);
        }
    }

    // Draw snake tail (triangular, pointing left - away from body)
    let tail_x = head_x - 3 * 8;
    if tail_x >= base_x - 16 && tail_x < 160 {
        unsafe { *DRAW_COLORS = 0x30 }; // Green
                                        // Triangle pointing left: wide on right, point on left
        rect(tail_x + 6, y + 1, 2, 6); // Base (wide)
        rect(tail_x + 4, y + 2, 2, 4); // Middle
        rect(tail_x + 2, y + 3, 2, 2); // Tip
    }

    // Draw snake body (continuous style - 2 middle segments)
    unsafe { *DRAW_COLORS = 0x03 }; // Green stroke only
    for i in 1..3 {
        let seg_x = head_x - i * 8;
        if seg_x >= base_x - 16 && seg_x < 160 {
            // Only draw top and bottom edges (left/right connect to neighbors)
            hline(seg_x, y, 8); // Top
            hline(seg_x, y + 7, 8); // Bottom
        }
    }

    // Draw snake head with bullet shape (pointed right, blunt tip)
    let eating = (45..55).contains(&cycle_frame);
    if eating && (frame / 4).is_multiple_of(2) {
        // Open mouth animation
        unsafe { *DRAW_COLORS = 0x43 }; // Yellow fill
        rect(head_x, y, 8, 3);
        rect(head_x, y + 5, 8, 3);
    } else {
        unsafe { *DRAW_COLORS = 0x43 }; // Yellow fill, green stroke
                                        // Bullet shape: square left, tapered right
        rect(head_x, y + 1, 5, 6); // Main body
        rect(head_x + 5, y + 2, 2, 4); // Taper
        rect(head_x + 7, y + 3, 1, 2); // Blunt tip
    }
}

/// Draw the settings menu
pub fn draw_settings_menu(selected: u8, music_on: bool, sfx_on: bool, cheat_on: bool) {
    // Title
    unsafe { *DRAW_COLORS = 0x04 }; // Yellow
    text(b"SETTINGS", 48, 20);

    // Settings options
    let options: [(&[u8], bool); 3] = [
        (b"Music", music_on),
        (b"Sound FX", sfx_on),
        (b"Cheat Mode", cheat_on),
    ];

    for (i, (label, enabled)) in options.iter().enumerate() {
        let y = 50 + i as i32 * 20;

        // Selection indicator
        if i as u8 == selected {
            unsafe { *DRAW_COLORS = 0x04 }; // Yellow highlight
            text(b">", 16, y);
        }

        // Label
        if i as u8 == selected {
            unsafe { *DRAW_COLORS = 0x04 };
        } else {
            unsafe { *DRAW_COLORS = 0x03 };
        }
        text(*label, 28, y);

        // ON/OFF indicator
        if *enabled {
            unsafe { *DRAW_COLORS = 0x03 }; // Green for ON
            text(b"ON", 120, y);
        } else {
            unsafe { *DRAW_COLORS = 0x02 }; // Purple for OFF
            text(b"OFF", 116, y);
        }
    }

    // Controls hint
    unsafe { *DRAW_COLORS = 0x02 };
    text(b"X:Toggle  Z:Back", 24, 145);
}

/// Draw the difficulty selection screen
pub fn draw_difficulty_select(selected: u8, high_scores: &[u32; 5]) {
    // Title
    unsafe { *DRAW_COLORS = 0x04 }; // Yellow
    text(b"SELECT LEVEL", 36, 15);

    // Difficulty options
    let options: [(&[u8], &[u8]); 5] = [
        (b"Classic", b"No enemies"),
        (b"Noob", b"2 enemies"),
        (b"Normal", b"3 enemies"),
        (b"Hell", b"5 enemies"),
        (b"Nightmare", b"8 enemies"),
    ];

    for (i, (name, desc)) in options.iter().enumerate() {
        let y = 35 + i as i32 * 22;

        // Selection indicator
        if i as u8 == selected {
            unsafe { *DRAW_COLORS = 0x04 }; // Yellow highlight
            text(b">", 8, y);
        }

        // Difficulty name
        if i as u8 == selected {
            unsafe { *DRAW_COLORS = 0x04 }; // Yellow for selected
        } else {
            unsafe { *DRAW_COLORS = 0x03 }; // Green for others
        }
        text(*name, 20, y);

        // Description
        unsafe { *DRAW_COLORS = 0x02 }; // Purple
        text(*desc, 20, y + 10);

        // High score (if any)
        if high_scores[i] > 0 {
            draw_high_score(high_scores[i], 120, y);
        }
    }

    // Controls hint
    unsafe { *DRAW_COLORS = 0x02 };
    text(b"UP/DOWN: Select", 24, 145);
}

/// Draw the game over screen with options
pub fn draw_game_over_menu(score: u32, high_score: u32, selected: u8) {
    // Overlay box
    unsafe { *DRAW_COLORS = 0x41 };
    rect(15, 35, 130, 90);

    // Game Over title
    unsafe { *DRAW_COLORS = 0x01 };
    text(b"GAME OVER", 44, 45);

    // Score
    unsafe { *DRAW_COLORS = 0x03 };
    text(b"Score:", 28, 62);
    draw_score_number(score, 80, 62);

    // High score
    if score >= high_score && high_score > 0 {
        unsafe { *DRAW_COLORS = 0x04 };
        text(b"NEW BEST!", 48, 75);
    } else if high_score > 0 {
        unsafe { *DRAW_COLORS = 0x02 };
        text(b"Best:", 36, 75);
        draw_score_number(high_score, 80, 75);
    }

    // Options
    let options: [&[u8]; 2] = [b"Retry", b"Menu"];
    for (i, option) in options.iter().enumerate() {
        let y = 95 + i as i32 * 12;

        if i as u8 == selected {
            unsafe { *DRAW_COLORS = 0x04 };
            text(b">", 48, y);
        }

        if i as u8 == selected {
            unsafe { *DRAW_COLORS = 0x04 };
        } else {
            unsafe { *DRAW_COLORS = 0x03 };
        }
        text(*option, 60, y);
    }
}

/// Helper: draw a score number at position
fn draw_score_number(score: u32, x: i32, y: i32) {
    let mut buf = [0u8; 10];
    let s = format_u32(score, &mut buf);
    unsafe { *DRAW_COLORS = 0x04 };
    text(s, x, y);
}

/// Helper: draw high score indicator
fn draw_high_score(score: u32, x: i32, y: i32) {
    let mut buf = [0u8; 10];
    let s = format_u32(score, &mut buf);
    unsafe { *DRAW_COLORS = 0x04 };
    text(s, x, y);
}

/// Format u32 to byte slice (no_std friendly)
fn format_u32(mut n: u32, buf: &mut [u8]) -> &[u8] {
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
