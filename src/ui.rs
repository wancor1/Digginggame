use crate::constants::*;
use crate::managers::LanguageManager;
use crate::utils::{calculate_text_center_position, estimate_text_width};
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

    pub fn draw(&self, font: Option<&Font>) {
        if !self.is_alive {
            return;
        }

        draw_rectangle(
            self.current_x.floor(),
            self.current_y.floor(),
            self.box_width,
            self.box_height,
            NOTIFICATION_BG_COLOR,
        );

        let text_col = if self.msg_type == "error" {
            NOTIFICATION_TEXT_COLOR_ERROR
        } else if self.msg_type == "success" {
            NOTIFICATION_TEXT_COLOR_SUCCESS
        } else {
            NOTIFICATION_TEXT_COLOR_INFO
        };

        draw_rectangle_lines(
            self.current_x.floor(),
            self.current_y.floor(),
            self.box_width,
            self.box_height,
            1.0,
            text_col,
        );

        let mut y_off = self.current_y + NOTIFICATION_PADDING_Y;
        for line in &self.wrapped_lines {
            // draw_text uses baseline, so add approx ascent. FONT_SIZE is usually roughly height.
            // We'll add FONT_SIZE * 0.8 as baseline offset.
            draw_text_ex(
                line,
                (self.current_x + NOTIFICATION_PADDING_X).floor(),
                (y_off + FONT_SIZE * 0.8).floor(),
                TextParams {
                    font_size: FONT_SIZE as u16,
                    font: font.or_else(|| TextParams::default().font),
                    color: text_col,
                    ..Default::default()
                },
            );
            y_off += FONT_SIZE + NOTIFICATION_LINE_SPACING;
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
    block_coords: Option<(f32, f32)>, // New field to store the hovered block's coordinates
}

impl SelectBlock {
    pub fn new() -> Self {
        Self {
            selection_effect_start_time: 0.0,
            is_effect_active: false,
            block_coords: None, // Initialize to None
        }
    }

    pub fn is_effect_active(&self) -> bool {
        self.is_effect_active
    }

    pub fn update(&mut self, hovered_block_coords: Option<(f32, f32)>) {
        if let Some(coords) = hovered_block_coords {
            if !self.is_effect_active {
                // Effect just became active
                self.selection_effect_start_time = get_time();
            } else if self.block_coords != Some(coords) {
                // Hovered block changed, reset timer for new pulse
                self.selection_effect_start_time = get_time();
            }
            self.is_effect_active = true;
            self.block_coords = Some(coords);
        } else {
            self.is_effect_active = false;
            self.block_coords = None;
        }
    }

    pub fn draw(
        &mut self,
        camera_x: f32, // camera_x and camera_y are still needed for world-to-screen conversion
        camera_y: f32,
        atlas: &Texture2D,
    ) {
        if !self.is_effect_active {
            return;
        }

        let (world_block_x, world_block_y) = self.block_coords.unwrap(); // We know it's Some if is_effect_active is true

        let screen_x = (world_block_x - camera_x).floor();
        let screen_y = (world_block_y - camera_y).floor();
        let elapsed = get_time() - self.selection_effect_start_time;

        // Reset every 2s
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
} // Added missing closing brace

pub struct ButtonBox;

impl ButtonBox {
    pub fn draw_button(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text_key: &str,
        press_key: &str,
        lang: &LanguageManager,
        font: Option<&Font>,
    ) -> bool {
        let mouse_pos = mouse_position();
        let mx = (mouse_pos.0 / screen_width()) * SCREEN_WIDTH;
        let my = (mouse_pos.1 / screen_height()) * SCREEN_HEIGHT;

        let is_hover = mx >= x && mx < x + w && my >= y && my < y + h;
        let is_pressed = is_hover && is_mouse_button_down(MouseButton::Left);
        let is_released = is_hover && is_mouse_button_released(MouseButton::Left);

        let bg_col = if is_pressed {
            COLOR_BUTTON_PRESSED_BG
        } else {
            COLOR_BUTTON_BG
        };
        draw_rectangle(x.floor(), y.floor(), w, h, bg_col);
        draw_rectangle_lines(x.floor(), y.floor(), w, h, 1.0, COLOR_BUTTON_BORDER);

        if !is_pressed {
            draw_line(
                (x + w - 1.0).floor(),
                (y + 1.0).floor(),
                (x + w - 1.0).floor(),
                (y + h - 1.0).floor(),
                1.0,
                BLACK,
            );
            draw_line(
                (x + 1.0).floor(),
                (y + h - 1.0).floor(),
                (x + w - 2.0).floor(),
                (y + h - 1.0).floor(),
                1.0,
                BLACK,
            );
        }

        let key = if is_pressed { press_key } else { text_key };
        let label = lang.get_string(key);

        let (tx, ty) = calculate_text_center_position(w, h, &label);

        let text_offset_x = if is_pressed { 1.0 } else { 0.0 };
        let text_offset_y = if is_pressed { 1.0 } else { 0.0 };

        // Ensure color is correct using TextParams
        draw_text_ex(
            &label,
            (x + tx + text_offset_x).floor(),
            (y + ty + text_offset_y + 1.0).floor(),
            TextParams {
                font_size: FONT_SIZE as u16,
                font: font.or_else(|| TextParams::default().font),
                color: COLOR_BUTTON_TEXT,
                ..Default::default()
            },
        );
        is_released
    }
}

pub struct GameMenu {
    pub is_open: bool, // is_menu_visible
    pub lang_dropdown_open: bool,
}

impl GameMenu {
    pub fn new() -> Self {
        Self {
            is_open: false,
            lang_dropdown_open: false,
        }
        // State is mostly managing interactions, which is handled in draw/update for IMGUI style here.
    }
}
