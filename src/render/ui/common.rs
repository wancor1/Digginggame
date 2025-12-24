use crate::constants::*;
use macroquad::prelude::*;
use macroquad::text::Font;
use macroquad::texture::Texture2D;

pub struct ButtonParams<'a> {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub text_key: &'a str,
    pub press_key: &'a str,
    pub lang: &'a crate::managers::LanguageManager,
    pub font_size: u16,
}

pub fn draw_button(params: ButtonParams, font: Option<&Font>) -> bool {
    let mouse_pos = mouse_position();
    let is_hover = mouse_pos.0 >= params.x
        && mouse_pos.0 < params.x + params.w
        && mouse_pos.1 >= params.y
        && mouse_pos.1 < params.y + params.h;
    let is_pressed = is_hover && is_mouse_button_down(MouseButton::Left);
    let is_released = is_hover && is_mouse_button_released(MouseButton::Left);

    let bg_col = if is_pressed {
        COLOR_BUTTON_PRESSED_BG
    } else {
        COLOR_BUTTON_BG
    };
    draw_rectangle(
        params.x.floor(),
        params.y.floor(),
        params.w,
        params.h,
        bg_col,
    );
    draw_rectangle_lines(
        params.x.floor(),
        params.y.floor(),
        params.w,
        params.h,
        1.0,
        COLOR_BUTTON_BORDER,
    );

    let key = if is_pressed {
        params.press_key
    } else {
        params.text_key
    };
    let label = params.lang.get_string(key);

    let t_measure = measure_text(&label, font, params.font_size, 1.0);
    let tx = params.x + (params.w - t_measure.width) / 2.0;
    let ty = params.y + (params.h + t_measure.height) / 2.0;

    draw_text_ex(
        &label,
        tx.floor(),
        ty.floor(),
        TextParams {
            font_size: params.font_size,
            font,
            color: COLOR_BUTTON_TEXT,
            ..Default::default()
        },
    );

    is_released
}

pub struct MenuRenderContext<'a> {
    pub font: Option<&'a Font>,
    pub atlas: Option<&'a Texture2D>,
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub font_size: u16,
    pub events: &'a mut Vec<crate::events::GameEvent>,
}
