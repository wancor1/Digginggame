use crate::constants::*;
use crate::render::sprites::*;
use macroquad::prelude::*;

#[derive(PartialEq)]
pub enum NotificationState {
    FadingIn,
    Visible,
    FadingOut,
}

pub struct Notification {
    pub message: String,
    pub duration: f64,
    pub msg_type: String, // info, error, success
    pub is_alive: bool,
    pub state: NotificationState,

    start_time: f64,
    current_x: f32,
    current_y: f32,
    target_x: f32,
    target_y: f32,
    vel_x: f32,

    // Computed layout
    box_width: f32,
    box_height: f32,
    wrapped_lines: Vec<String>,
}

impl Notification {
    pub fn new(
        message: String,
        duration: f64,
        msg_type: &str,
        max_width: f32,
        font: Option<&Font>,
    ) -> Self {
        let mut n = Self {
            message: message.clone(),
            duration,
            msg_type: msg_type.to_string(),
            is_alive: true,
            state: NotificationState::FadingIn,
            start_time: get_time(),
            current_x: 0.0,
            current_y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            vel_x: 0.0,
            box_width: 0.0,
            box_height: 0.0,
            wrapped_lines: Vec::new(),
        };
        n.calculate_dimensions(max_width, font);
        n
    }

    fn calculate_dimensions(&mut self, max_width: f32, font: Option<&Font>) {
        let font = match font {
            Some(f) => f,
            None => return, // Cannot calculate without a font
        };

        let measure = |s: &str| -> f32 { measure_text(s, Some(font), FONT_SIZE as u16, 1.0).width };

        let space_width = measure(" ");
        let words: Vec<&str> = self.message.split(' ').collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_line_width = 0.0;

        for word in words {
            let word_width = measure(word);

            if word_width > max_width {
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                    current_line_width = 0.0;
                }

                let mut temp_word_line = String::new();
                let mut temp_word_width = 0.0;
                for char in word.chars() {
                    let char_str = char.to_string();
                    let char_width = measure(&char_str);
                    if temp_word_width + char_width > max_width {
                        lines.push(temp_word_line);
                        temp_word_line = char_str;
                        temp_word_width = char_width;
                    } else {
                        temp_word_line.push(char);
                        temp_word_width += char_width;
                    }
                }
                if !temp_word_line.is_empty() {
                    current_line = temp_word_line;
                    current_line_width = temp_word_width;
                }
            } else {
                let width_if_added = if current_line.is_empty() {
                    word_width
                } else {
                    current_line_width + space_width + word_width
                };

                if width_if_added > max_width && !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = word.to_string();
                    current_line_width = word_width;
                } else {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(word);
                    current_line_width = measure(&current_line);
                }
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        self.wrapped_lines = lines;

        let line_count = self.wrapped_lines.len();
        self.box_height = (line_count as f32 * FONT_SIZE)
            + (line_count.saturating_sub(1) as f32 * NOTIFICATION_LINE_SPACING)
            + NOTIFICATION_PADDING_Y * 2.0;

        let max_line_w = self
            .wrapped_lines
            .iter()
            .map(|l| measure(l))
            .fold(0.0f32, f32::max);

        self.box_width = (max_line_w + NOTIFICATION_PADDING_X * 2.0).min(NOTIFICATION_MAX_WIDTH);
    }

    pub fn set_target_position(&mut self, x: f32, y: f32) {
        self.target_x = x;
        self.target_y = y;
        // Init position if new
        if self.current_x == 0.0 && self.current_y == 0.0 {
            self.current_x = x;
            self.current_y = y + NOTIFICATION_FADE_IN_OFFSET_Y;
        }
    }

    pub fn update(&mut self) {
        if !self.is_alive {
            return;
        }

        // Y Movement
        let dy = self.target_y - self.current_y;
        if dy.abs() > NOTIFICATION_TARGET_Y_TOLERANCE {
            let move_dist = NOTIFICATION_FADE_IN_AMOUNT_Y_PER_FRAME.min(dy.abs());
            self.current_y += if dy > 0.0 { move_dist } else { -move_dist };
        } else {
            self.current_y = self.target_y;
        }

        // State Machine
        match self.state {
            NotificationState::FadingIn => {
                if (self.current_y - self.target_y).abs() <= NOTIFICATION_TARGET_Y_TOLERANCE {
                    self.state = NotificationState::Visible;
                }
            }
            NotificationState::Visible => {
                if get_time() - self.start_time > self.duration {
                    self.state = NotificationState::FadingOut;
                    self.vel_x = NOTIFICATION_FADE_OUT_INITIAL_AMOUNT_X_PER_FRAME;
                }
            }
            NotificationState::FadingOut => {
                self.vel_x += NOTIFICATION_FADE_OUT_ACCELERATION_X_PER_FRAME;
                self.current_x += self.vel_x;
                if self.current_x > SCREEN_WIDTH {
                    self.is_alive = false;
                }
            }
        }
    }

    pub fn draw_high_res(&self, font: Option<&Font>, scale: f32, off_x: f32, off_y: f32) {
        if !self.is_alive {
            return;
        }

        let sx = off_x + self.current_x * scale;
        let sy = off_y + self.current_y * scale;
        let sw = self.box_width * scale;
        let sh = self.box_height * scale;

        draw_rectangle(sx.floor(), sy.floor(), sw, sh, NOTIFICATION_BG_COLOR);

        let text_col = if self.msg_type == "error" {
            NOTIFICATION_TEXT_COLOR_ERROR
        } else if self.msg_type == "success" {
            NOTIFICATION_TEXT_COLOR_SUCCESS
        } else {
            NOTIFICATION_TEXT_COLOR_INFO
        };

        draw_rectangle_lines(sx.floor(), sy.floor(), sw, sh, 1.0, text_col);

        let mut y_off = sy + NOTIFICATION_PADDING_Y * scale;
        let s_font_size = (FONT_SIZE * scale).floor() as u16;
        for line in &self.wrapped_lines {
            draw_text_ex(
                line,
                (sx + NOTIFICATION_PADDING_X * scale).floor(),
                (y_off + FONT_SIZE * scale * 0.8).floor(),
                TextParams {
                    font_size: s_font_size,
                    font: font.or_else(|| TextParams::default().font),
                    color: text_col,
                    ..Default::default()
                },
            );
            y_off += (FONT_SIZE + NOTIFICATION_LINE_SPACING) * scale;
        }
    }

    pub fn get_box_width(&self) -> f32 {
        self.box_width
    }
    pub fn get_box_height(&self) -> f32 {
        self.box_height
    }
}

pub struct SelectBlock {
    selection_effect_start_time: f64,
    is_effect_active: bool,
    block_coords: Option<(f32, f32)>,
    preview_sprite: Option<Rect>,
    is_valid: bool,
}

impl SelectBlock {
    pub fn new() -> Self {
        Self {
            selection_effect_start_time: 0.0,
            is_effect_active: false,
            block_coords: None,
            preview_sprite: None,
            is_valid: true,
        }
    }

    pub fn update(
        &mut self,
        hovered_block_coords: Option<(f32, f32)>,
        preview_sprite: Option<Rect>,
        is_valid: bool,
    ) {
        if let Some(coords) = hovered_block_coords {
            if !self.is_effect_active {
                self.selection_effect_start_time = get_time();
            } else if self.block_coords != Some(coords) {
                self.selection_effect_start_time = get_time();
            }
            self.is_effect_active = true;
            self.block_coords = Some(coords);
        } else {
            self.is_effect_active = false;
            self.block_coords = None;
        }
        self.preview_sprite = preview_sprite;
        self.is_valid = is_valid;
    }

    pub fn draw(
        &mut self,
        camera_x: f32,
        camera_y: f32,
        atlas: &Texture2D,
    ) {
        if !self.is_effect_active {
            return;
        }

        let (world_block_x, world_block_y) = self.block_coords.unwrap();

        let screen_x = (world_block_x - camera_x).round();
        let screen_y = (world_block_y - camera_y).round();
        let elapsed = get_time() - self.selection_effect_start_time;

        if let Some(sprite) = self.preview_sprite {
            let color = if self.is_valid {
                Color::new(1.0, 1.0, 1.0, 0.4)
            } else {
                Color::new(1.0, 0.3, 0.3, 0.5)
            };
            draw_texture_ex(
                atlas,
                screen_x,
                screen_y,
                color,
                DrawTextureParams {
                    source: Some(sprite),
                    ..Default::default()
                },
            );
        }

        if elapsed > SELECTION_PULSE_DURATION {
            self.selection_effect_start_time = get_time();
        }

        if elapsed <= (SELECTION_PULSE_DURATION / 2.0) {
            draw_texture_ex(
                atlas,
                screen_x,
                screen_y,
                WHITE,
                DrawTextureParams {
                    source: Some(SPRITE_SELECT_NORMAL),
                    ..Default::default()
                },
            );
        } else {
            draw_texture_ex(
                atlas,
                screen_x - SELECTION_ENLARGE_AMOUNT,
                screen_y - SELECTION_ENLARGE_AMOUNT,
                WHITE,
                DrawTextureParams {
                    source: Some(SPRITE_SELECT_LARGE),
                    dest_size: Some(vec2(
                        BLOCK_SIZE + SELECTION_ENLARGE_AMOUNT * 2.0,
                        BLOCK_SIZE + SELECTION_ENLARGE_AMOUNT * 2.0,
                    )),
                    ..Default::default()
                },
            );
        }
    }
}
