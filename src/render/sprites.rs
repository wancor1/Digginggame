use macroquad::prelude::Rect;

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

/// ---------------------------------------------------
/// BLOCK SPRITES

// blocks
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
    x: 48.0,
    y: 16.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_INDESTRUCTIBLE: Rect = Rect {
    x: 48.0,
    y: 24.0,
    w: 8.0,
    h: 8.0,
};

// utils
pub const SPRITE_BLOCK_WARPGATE: Rect = Rect {
    x: 56.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};

/// ---------------------------------------------------
/// natural
// sedimentary_solid
pub const SPRITE_BLOCK_COAL: Rect = Rect {
    x: 128.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};
// subsurface_fluid
// surface_liquid
// metal_ore
// precious_metal_ore
// rare_metal_ore
// radioactive_ore
// aluminum_ore
// native_element
// gemstone
// crystal_cluster
// industrial_mineral
// volcanic
// hazardous
/// ---------------------------------------------------
/// processed
// refined_metal
// alloy
// refined_material

/// ---------------------------------------------------
// Animation
pub const SPRITE_BREAK_ANIM_U: f32 = 40.0;
pub const SPRITE_BREAK_ANIM_V_START: f32 = 0.0;
