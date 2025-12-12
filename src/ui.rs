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
        // Simple word wrap
        let words: Vec<&str> = self.message.split(' ').collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };
            if estimate_text_width(&test_line, font) > max_width {
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = word.to_string();
                } else {
                    // Word itself is too long, force break logic omitted for brevity
                    lines.push(word.to_string());
                }
            } else {
                current_line = test_line;
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
            .map(|l| estimate_text_width(l, font))
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
            self.current_x,
            self.current_y,
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
            self.current_x,
            self.current_y,
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
                self.current_x + NOTIFICATION_PADDING_X,
                y_off + FONT_SIZE * 0.8,
                TextParams {
                    font_size: FONT_SIZE as u16,
                    font: font.copied().unwrap_or(TextParams::default().font),
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
}

impl SelectBlock {
    pub fn new() -> Self {
        Self {
            selection_effect_start_time: 0.0,
            is_effect_active: false,
        }
    }

    pub fn is_effect_active(&self) -> bool {
        self.is_effect_active
    }

    pub fn update(&mut self, is_mouse_over: bool) {
        if is_mouse_over && !self.is_effect_active {
            self.is_effect_active = true;
            self.selection_effect_start_time = get_time();
        } else if !is_mouse_over {
            self.is_effect_active = false;
        }
    }

    pub fn draw(&mut self, mouse_x: f32, mouse_y: f32, atlas: &Texture2D) {
        if !self.is_effect_active {
            return;
        }

        let grid_x = (mouse_x / BLOCK_SIZE).floor() * BLOCK_SIZE;
        let grid_y = (mouse_y / BLOCK_SIZE).floor() * BLOCK_SIZE;
        let elapsed = get_time() - self.selection_effect_start_time;

        // Reset every 2s
        if elapsed > 2.0 {
            self.selection_effect_start_time = get_time();
        }

        if elapsed <= 1.0 {
            draw_texture_ex(
                *atlas,
                grid_x,
                grid_y,
                WHITE,
                DrawTextureParams {
                    source: Some(SPRITE_SELECT_NORMAL),
                    ..Default::default()
                },
            );
        } else if elapsed <= 2.0 {
            draw_texture_ex(
                *atlas,
                grid_x - 1.0,
                grid_y - 1.0,
                WHITE,
                DrawTextureParams {
                    source: Some(SPRITE_SELECT_LARGE),
                    ..Default::default()
                },
            );
        }
    }
}

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
        draw_rectangle(x, y, w, h, bg_col);
        draw_rectangle_lines(x, y, w, h, 1.0, COLOR_BUTTON_BORDER);

        if !is_pressed {
            draw_line(x + w - 1.0, y + 1.0, x + w - 1.0, y + h - 2.0, 1.0, BLACK);
            draw_line(x + 1.0, y + h - 1.0, x + w - 2.0, y + h - 1.0, 1.0, BLACK);
        }

        let key = if is_pressed { press_key } else { text_key };
        let label = lang.get_string(key);

        let (tx, ty) = calculate_text_center_position(w, h, &label, font);
        // Ensure color is correct using TextParams
        draw_text_ex(
            &label,
            x + tx,
            y + ty,
            TextParams {
                font_size: FONT_SIZE as u16,
                font: font.copied().unwrap_or(TextParams::default().font),
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
