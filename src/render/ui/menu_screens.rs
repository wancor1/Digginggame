use super::common::{ButtonParams, draw_button};
use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use macroquad::prelude::*;

pub fn draw_title_screen(
    game: &Game,
    font: Option<&Font>,
    scale: f32,
    events: &mut Vec<GameEvent>,
) {
    let title = "Digging Game";
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    let params = TextParams {
        font_size: (FONT_SIZE * scale * 2.0) as u16,
        font,
        color: WHITE,
        ..Default::default()
    };
    let t_measure = measure_text(title, font, params.font_size, 1.0);
    draw_text_ex(
        title,
        (center_x - t_measure.width / 2.0).floor(),
        (center_y * 0.5).floor(),
        params,
    );

    let bw = 60.0 * scale;
    let bh = 10.0 * scale;
    let bx = (screen_width() - bw) / 2.0;
    let by = center_y;

    let s_font_size = (FONT_SIZE * scale).floor() as u16;

    if draw_button(
        ButtonParams {
            x: bx,
            y: by,
            w: bw,
            h: bh,
            text_key: "button.title_screen.start.default",
            press_key: "button.title_screen.start.pressed",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::OpenSaveSelection);
    }
    if draw_button(
        ButtonParams {
            x: bx,
            y: by + 15.0 * scale,
            w: bw,
            h: bh,
            text_key: "button.menu.quit.default",
            press_key: "button.menu.quit.pressed",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::QuitGame);
    }
}

pub fn draw_save_select_screen(
    game: &Game,
    font: Option<&Font>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    s_font_size: u16,
    events: &mut Vec<GameEvent>,
) {
    draw_text_ex(
        "Select Save File",
        offset_x + 10.0 * scale,
        offset_y + 20.0 * scale,
        TextParams {
            font_size: s_font_size,
            font,
            color: WHITE,
            ..Default::default()
        },
    );
    let mut cy = offset_y + 35.0 * scale;
    if draw_button(
        ButtonParams {
            x: offset_x + 10.0 * scale,
            y: cy,
            w: (SCREEN_WIDTH - 20.0) * scale,
            h: 10.0 * scale,
            text_key: "button.menu.new_game.default",
            press_key: "button.menu.new_game.pressed",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::StartNewGameSetup);
    }
    if draw_button(
        ButtonParams {
            x: offset_x + 2.0 * scale,
            y: offset_y + 2.0 * scale,
            w: 30.0 * scale,
            h: 10.0 * scale,
            text_key: "button.menu.return.default",
            press_key: "button.menu.return.pressed",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::ReturnToTitleFromSaveSelect);
    }
    cy += 15.0 * scale;
    for file in &game.save_files {
        if draw_button(
            ButtonParams {
                x: offset_x + 10.0 * scale,
                y: cy,
                w: (SCREEN_WIDTH - 20.0) * scale,
                h: 10.0 * scale,
                text_key: file,
                press_key: file,
                lang: &game.lang_manager,
                font_size: s_font_size,
            },
            font,
        ) {
            events.push(GameEvent::LoadSave(file.clone()));
        }
        cy += 12.0 * scale;
    }
}

pub fn draw_new_game_input_screen(
    game: &Game,
    font: Option<&Font>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    s_font_size: u16,
    events: &mut Vec<GameEvent>,
) {
    draw_text_ex(
        "Enter Filename:",
        offset_x + 10.0 * scale,
        offset_y + 30.0 * scale,
        TextParams {
            font_size: s_font_size,
            font,
            color: WHITE,
            ..Default::default()
        },
    );
    draw_rectangle(
        offset_x + 10.0 * scale,
        offset_y + 40.0 * scale,
        (SCREEN_WIDTH - 20.0) * scale,
        12.0 * scale,
        DARKGRAY,
    );
    let cur = if (get_time() * 2.0) as i32 % 2 == 0 {
        "|"
    } else {
        ""
    };
    draw_text_ex(
        &format!("{}{}", game.input_buffer, cur),
        offset_x + 12.0 * scale,
        offset_y + 49.0 * scale,
        TextParams {
            font_size: s_font_size,
            font,
            color: WHITE,
            ..Default::default()
        },
    );
    if draw_button(
        ButtonParams {
            x: offset_x + 10.0 * scale,
            y: offset_y + 60.0 * scale,
            w: 60.0 * scale,
            h: 10.0 * scale,
            text_key: "Confirm",
            press_key: "Confirm",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::ConfirmNewGame(game.input_buffer.clone()));
    }
}

pub fn draw_warp_place_screen(
    game: &Game,
    font: Option<&Font>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    s_font_size: u16,
    events: &mut Vec<GameEvent>,
) {
    draw_text_ex(
        &game.lang_manager.get_string("warp.name_prompt"),
        offset_x + 10.0 * scale,
        offset_y + 30.0 * scale,
        TextParams {
            font_size: s_font_size,
            font,
            color: WHITE,
            ..Default::default()
        },
    );
    draw_rectangle(
        offset_x + 10.0 * scale,
        offset_y + 40.0 * scale,
        (SCREEN_WIDTH - 20.0) * scale,
        12.0 * scale,
        DARKGRAY,
    );
    let cur = if (get_time() * 2.0) as i32 % 2 == 0 {
        "|"
    } else {
        ""
    };
    draw_text_ex(
        &format!("{}{}", game.input_buffer, cur),
        offset_x + 12.0 * scale,
        offset_y + 49.0 * scale,
        TextParams {
            font_size: s_font_size,
            font,
            color: WHITE,
            ..Default::default()
        },
    );
    if draw_button(
        ButtonParams {
            x: offset_x + 10.0 * scale,
            y: offset_y + 60.0 * scale,
            w: 60.0 * scale,
            h: 10.0 * scale,
            text_key: "Confirm",
            press_key: "Confirm",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::ConfirmWarpGateName(game.input_buffer.clone()));
    }
}

pub fn draw_warp_select_screen(
    game: &Game,
    font: Option<&Font>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    s_font_size: u16,
    events: &mut Vec<GameEvent>,
) {
    draw_text_ex(
        &game.lang_manager.get_string("warp.title"),
        offset_x + 10.0 * scale,
        offset_y + 20.0 * scale,
        TextParams {
            font_size: s_font_size,
            font,
            color: WHITE,
            ..Default::default()
        },
    );

    if draw_button(
        ButtonParams {
            x: offset_x + 2.0 * scale,
            y: offset_y + 2.0 * scale,
            w: 30.0 * scale,
            h: 10.0 * scale,
            text_key: "shop.back_to_game",
            press_key: "shop.back_to_game",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::CloseMenu);
    }

    let mut cy = offset_y + 35.0 * scale;
    for (i, gate) in game.player_manager.player.warp_gates.iter().enumerate() {
        let label = format!("> {}", gate.name);
        if draw_button(
            ButtonParams {
                x: offset_x + 10.0 * scale,
                y: cy,
                w: (SCREEN_WIDTH - 20.0) * scale,
                h: 10.0 * scale,
                text_key: &label,
                press_key: &label,
                lang: &game.lang_manager,
                font_size: s_font_size,
            },
            font,
        ) {
            events.push(GameEvent::TeleportToWarp(i));
        }
        cy += 12.0 * scale;
    }
}

pub fn draw_pause_menu(
    game: &Game,
    font: Option<&Font>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    s_font_size: u16,
    events: &mut Vec<GameEvent>,
) {
    let (mw, mh) = (80.0 * scale, 60.0 * scale);
    let (mx, my) = (
        offset_x + ((SCREEN_WIDTH - 80.0) / 2.0).floor() * scale,
        offset_y + ((SCREEN_HEIGHT - 60.0) / 2.0).floor() * scale,
    );
    draw_rectangle(mx, my, mw, mh, LIGHTGRAY);
    draw_rectangle_lines(mx, my, mw, mh, 1.0, BLACK);

    let mut cur_y = (my + 10.0 * scale).floor();
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * scale,
            y: cur_y,
            w: mw - 10.0 * scale,
            h: 10.0 * scale,
            text_key: "button.menu.return.default",
            press_key: "button.menu.return.pressed",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::CloseMenu);
    }
    cur_y += 15.0 * scale;
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * scale,
            y: cur_y,
            w: mw - 10.0 * scale,
            h: 10.0 * scale,
            text_key: "button.menu.save.default",
            press_key: "button.menu.save.pressed",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::SaveGame);
    }
    cur_y += 12.0 * scale;
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * scale,
            y: cur_y,
            w: mw - 10.0 * scale,
            h: 10.0 * scale,
            text_key: "button.menu.quit_to_title.default",
            press_key: "button.menu.quit_to_title.pressed",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::ReturnToTitle);
    }
}
