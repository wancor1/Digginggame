use macroquad::prelude::Rect;

pub const SCREEN_WIDTH: f32 = 160.0;
pub const SCREEN_HEIGHT: f32 = 120.0;
pub const BLOCK_SIZE: f32 = 8.0;

// Sprite Coordinates (U, V, W, H) - Assuming 256x256 texture atlas or similar, but
// logic uses pixel coordinates. UV in Macroquad draw_texture_ex usually requires pixel source rect.

// Pyxel bank 0 (Game) -> Atlas Region 1 (e.g., top-left)
// Pyxel bank 1 (UI)   -> Atlas Region 2
// For simplicity in this port, we'll assume `atlas.png` combines everything.
// We need to map the Python tuples: (BANK, U, V, W, H, COLKEY) to Rects.
// Since we have a single atlas, we ignore the bank index and just map UVs.
// If the user provided `atlas.png` is just the `sprite_sheet.pyxres` exports stitched,
// we might need to know the offset.
// For now, I'll assume standard layout or direct mapping.
// Based on `_create_dummy_sprites_if_needed` in Python:
// Bank 0 (Game): 32x16
// Bank 1 (UI): 32x64
// Let's assume they are side-by-side or vertical in atlas.
// Wait, the user said "pyxelのタイルについてはsrc/atlas.pngを作ったのでどこを使用するか指定できるようにして対応してください"
// (I made src/atlas.png for pyxel tiles, so please support specifying where to use it.)
// I'll define constants for Rects.

// Game Bank (Bank 0) Mappings
pub const SPRITE_SELECT_NORMAL: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_SELECT_LARGE: Rect = Rect {
    x: 0.0,
    y: 8.0,
    w: 10.0,
    h: 10.0,
};
pub const SPRITE_CURSOR: Rect = Rect {
    x: 16.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};

// UI Bank (Bank 1) Mappings - Python uses bank 1.
// We need to know where Bank 1 starts in `atlas.png`.
// Let's assume Bank 1 is below Bank 0. Bank 0 height is 16.
// So Bank 1 V start = 16.
const BANK_1_OFFSET_Y: f32 = 24.0;

pub const SPRITE_BLOCK_DIRT: Rect = Rect {
    x: 8.0,
    y: BANK_1_OFFSET_Y + 0.0,
    w: 8.0,
    h: 8.0,
}; // Same as error in python code? Check lines 13-14 of constants.py
// Line 13: SPRITE_BLOCK_ERROR = (SPRITE_BANK_UI, 8, 0, 8, 8, 1)
// Line 14: SPRITE_BLOCK_DIRT = (SPRITE_BANK_UI, 8, 0, 8, 8, 0)
// They share the same UV in Python, just different colkey.

pub const SPRITE_BLOCK_GRASS: Rect = Rect {
    x: 16.0,
    y: BANK_1_OFFSET_Y + 0.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_STONE: Rect = Rect {
    x: 8.0,
    y: BANK_1_OFFSET_Y + 8.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_COAL: Rect = Rect {
    x: 16.0,
    y: BANK_1_OFFSET_Y + 8.0,
    w: 8.0,
    h: 8.0,
};

// Animation
pub const SPRITE_BREAK_ANIM_U: f32 = 0.0;
pub const SPRITE_BREAK_ANIM_V_START: f32 = BANK_1_OFFSET_Y + 0.0;

// Colors (Pyxel Palette approximation for UI mainly)
// Macroquad colors are flloats 0.0-1.0
// We can use a helper to map 0-15 to Color if needed, or just use semantic names.
// Pyxel colors:
// 0: Black, 7: White, 13: Light Gray/Blue-ish?, 10: Yellow?
// Let's define them as hex or macroquad Colors.
use macroquad::color::Color;

pub const COLOR_BUTTON_BG: Color = macroquad::color::GRAY; // 13
pub const COLOR_BUTTON_BORDER: Color = macroquad::color::WHITE; // 7
pub const COLOR_BUTTON_TEXT: Color = macroquad::color::WHITE; // 7
pub const COLOR_BUTTON_PRESSED_BG: Color = macroquad::color::YELLOW; // 10

pub const SAVE_FILE_NAME: &str = "savegame.json";
pub const LANG_FOLDER: &str = "lang";
pub const DEFAULT_LANGUAGE: &str = "en_us";
pub const FONT_SIZE: f32 = 8.0;

pub const NOTIFICATION_PADDING_X: f32 = 5.0;
pub const NOTIFICATION_PADDING_Y: f32 = 3.0;
pub const NOTIFICATION_LINE_SPACING: f32 = 2.0;
pub const NOTIFICATION_INTER_ITEM_SPACING: f32 = 2.0;
pub const NOTIFICATION_MAX_WIDTH: f32 = 115.0;
pub const NOTIFICATION_MAX_DISPLAY_TIME: f32 = 2.0;
pub const MAX_NOTIFICATIONS: usize = 3;

pub const NOTIFICATION_FADE_IN_AMOUNT_Y_PER_FRAME: f32 = 5.0;
pub const NOTIFICATION_FADE_OUT_INITIAL_AMOUNT_X_PER_FRAME: f32 = 2.0;
pub const NOTIFICATION_FADE_OUT_ACCELERATION_X_PER_FRAME: f32 = 0.5;
pub const NOTIFICATION_FADE_IN_OFFSET_Y: f32 = -30.0;
pub const NOTIFICATION_TARGET_Y_TOLERANCE: f32 = 1.0;

pub const NOTIFICATION_BG_COLOR: Color = macroquad::color::GRAY;
pub const NOTIFICATION_TEXT_COLOR_INFO: Color = macroquad::color::BLACK;
pub const NOTIFICATION_TEXT_COLOR_ERROR: Color = macroquad::color::RED;
pub const NOTIFICATION_TEXT_COLOR_SUCCESS: Color = macroquad::color::GREEN; // 5 is usually dark blue/purple in pyxel, but let's use green for success.

pub const CHUNK_SIZE_X_BLOCKS: usize = 16;
pub const CHUNK_SIZE_Y_BLOCKS: usize = 16;

// Selection Effect Constants
pub const SELECTION_PULSE_DURATION: f64 = 2.0; // Duration of one full pulse cycle in seconds
pub const SELECTION_ENLARGE_AMOUNT: f32 = 1.0; // How much the selection sprite enlarges
