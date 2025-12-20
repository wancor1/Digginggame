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
    x: 56.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};

// ore blocks
pub const SPRITE_BLOCK_COAL: Rect = Rect {
    x: 56.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_COPPER: Rect = Rect {
    x: 56.0,
    y: 8.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_IRON: Rect = Rect {
    x: 56.0,
    y: 16.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_SILVER: Rect = Rect {
    x: 56.0,
    y: 24.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_GOLD: Rect = Rect {
    x: 56.0,
    y: 32.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_PLATINUM: Rect = Rect {
    x: 56.0,
    y: 40.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_URANIUM: Rect = Rect {
    x: 56.0,
    y: 48.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_PYRITE: Rect = Rect {
    x: 56.0,
    y: 56.0,
    w: 8.0,
    h: 8.0,
};

// gem blocks
pub const SPRITE_BLOCK_DIAMOND: Rect = Rect {
    x: 64.0,
    y: 0.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_EMERALD: Rect = Rect {
    x: 64.0,
    y: 8.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_RUBY: Rect = Rect {
    x: 64.0,
    y: 16.0,
    w: 8.0,
    h: 8.0,
};
pub const SPRITE_BLOCK_SAPPHIRE: Rect = Rect {
    x: 64.0,
    y: 24.0,
    w: 8.0,
    h: 8.0,
};

// continued ore,gem blocks
pub const SPRITE_BLOCK_AMETHYST: Rect = Rect {
    x: 72.0,
    y: 0.0,
    w: 32.0,
    h: 48.0,
};

// Animation
pub const SPRITE_BREAK_ANIM_U: f32 = 40.0;
pub const SPRITE_BREAK_ANIM_V_START: f32 = 0.0;
