use macroquad::prelude::Rect;

pub const SCREEN_WIDTH: f32 = 160.0;
pub const SCREEN_HEIGHT: f32 = 120.0;
pub const BLOCK_SIZE: f32 = 8.0;

// Game Bank (Bank 0) Mappings
pub const SPRITE_SELECT_NORMAL: Rect = Rect {
    x: 8.0,
    y: 8.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_SELECT_LARGE: Rect = Rect {
    x: 8.0,
    y: 16.0,
    w: 10.0,
    h: 10.0,
};
pub const SPRITE_CURSOR: Rect = Rect {
    x: 8.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};

pub const SPRITE_BLOCK_DIRT: Rect = Rect {
    x: 48.0,
    y: 8.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_GRASS: Rect = Rect {
    x: 48.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_STONE: Rect = Rect {
    x: 56.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_COAL: Rect = Rect {
    x: 56.0,
    y: 8.0,
    w: 8.0,
    h: 8.0,
};

// Animation
pub const SPRITE_BREAK_ANIM_U: f32 = 40.0;
pub const SPRITE_BREAK_ANIM_V_START: f32 = 0.0;

use macroquad::color::Color;

pub const COLOR_BUTTON_BG: Color = macroquad::color::GRAY; // 13
pub const COLOR_BUTTON_BORDER: Color = macroquad::color::WHITE; // 7
pub const COLOR_BUTTON_TEXT: Color = macroquad::color::WHITE; // 7
pub const COLOR_BUTTON_PRESSED_BG: Color = macroquad::color::YELLOW; // 10

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

// World Generation Constants
pub const HARDNESS_MIN: i32 = 3;
pub const SURFACE_Y_LEVEL: i32 = 7;
pub const NOISE_SCALE_HARDNESS: f64 = 0.005;
pub const NOISE_SCALE_ORE: f64 = 0.04;
pub const ORE_THRESHOLD: f64 = 0.4;
pub const HARDNESS_INCREASE_PER_BLOCK: f64 = 0.1;
pub const NOISE_HARDNESS_RANGE: f64 = 20.0;

// Particle Constants
pub const GRAVITY: f32 = 0.19;
pub const MAX_LIFESPAN_ON_GROUND_SEC: f64 = 5.0;
pub const BOUNCE_DAMPENING_X: f32 = -0.4;
pub const FRICTION_ON_GROUND: f32 = 0.85;

// Component Constants
pub const PARTICLE_SPEED_MIN: f32 = 20.0 / 60.0;
pub const PARTICLE_SPEED_MAX: f32 = 1.0;

// Player Constants
pub const PLAYER_INITIAL_X: f32 = 80.0;
pub const PLAYER_INITIAL_Y: f32 = 48.0;
pub const PLAYER_INITIAL_FUEL: f32 = 100.0;
pub const PLAYER_INITIAL_CARGO: usize = 30;
pub const PLAYER_GRAVITY: f32 = 0.1;
pub const PLAYER_FRICTION_AIR: f32 = 0.95;
pub const PLAYER_FRICTION_GROUND: f32 = 0.8;
pub const PLAYER_TERMINAL_VELOCITY: f32 = 4.0;
