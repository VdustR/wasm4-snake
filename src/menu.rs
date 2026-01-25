use crate::wasm4::*;

/// Draw the main menu screen
pub fn draw_main_menu() {
    // Title
    unsafe { *DRAW_COLORS = 0x04 }; // Yellow
    text(b"SNAKE", 56, 30);

    // Subtitle
    unsafe { *DRAW_COLORS = 0x03 }; // Green
    text(b"WASM-4 Edition", 32, 50);

    // Press to start
    unsafe { *DRAW_COLORS = 0x02 }; // Purple (blinking effect via caller)
    text(b"Press X to Start", 24, 100);

    // Credits
    unsafe { *DRAW_COLORS = 0x02 };
    text(b"v1.0", 68, 145);
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

/// Draw the pause menu overlay
pub fn draw_pause_menu(selected: u8, music_enabled: bool, sfx_enabled: bool) {
    // Dark overlay
    unsafe { *DRAW_COLORS = 0x11 }; // Fill with color 1 (darkest)
    rect(20, 40, 120, 85);

    // Border
    unsafe { *DRAW_COLORS = 0x04 };
    rect(22, 42, 116, 81);
    unsafe { *DRAW_COLORS = 0x11 };
    rect(24, 44, 112, 77);

    // Title
    unsafe { *DRAW_COLORS = 0x04 };
    text(b"PAUSED", 56, 48);

    // Options: Continue, Music, SFX, Quit
    let base_y = 64;
    let line_height = 12;

    // Option 0: Continue
    draw_menu_option(0, selected, b"Continue", base_y);

    // Option 1: Music toggle
    if music_enabled {
        draw_menu_option(1, selected, b"Music: ON", base_y + line_height);
    } else {
        draw_menu_option(1, selected, b"Music: OFF", base_y + line_height);
    }

    // Option 2: SFX toggle
    if sfx_enabled {
        draw_menu_option(2, selected, b"SFX: ON", base_y + line_height * 2);
    } else {
        draw_menu_option(2, selected, b"SFX: OFF", base_y + line_height * 2);
    }

    // Option 3: Quit
    draw_menu_option(3, selected, b"Quit", base_y + line_height * 3);
}

/// Helper to draw a menu option
fn draw_menu_option(index: u8, selected: u8, label: &[u8], y: i32) {
    if index == selected {
        unsafe { *DRAW_COLORS = 0x04 }; // Yellow highlight
        text(b">", 32, y);
    }

    if index == selected {
        unsafe { *DRAW_COLORS = 0x04 };
    } else {
        unsafe { *DRAW_COLORS = 0x03 };
    }
    text(label, 44, y);
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
