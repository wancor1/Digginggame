use image::{ImageFormat, load_from_memory_with_format};
use macroquad::prelude::*;
use miniquad::conf::Icon;

mod components;
mod constants;
mod events;
mod game;
mod managers;
mod render;
mod ui;
mod utils;

use constants::*;
use events::GameEvent;
use game::Game;
use render::game_renderer::GameRenderer;

fn window_conf() -> Conf {
    let icon_bytes = include_bytes!("../src/icon.png");
    let dyn_image = load_from_memory_with_format(icon_bytes, ImageFormat::Png)
        .expect("Failed to load icon image");

    let to_rgba8 = |img: image::DynamicImage| -> Vec<u8> { img.to_rgba8().into_vec() };

    let small_icon_data: [u8; 16 * 16 * 4] = {
        let resized = dyn_image.resize_exact(16, 16, image::imageops::FilterType::Triangle);
        let rgba_data = to_rgba8(resized);
        let mut data_array = [0u8; 16 * 16 * 4];
        data_array.copy_from_slice(&rgba_data);
        data_array
    };

    let medium_icon_data: [u8; 32 * 32 * 4] = {
        let resized = dyn_image.resize_exact(32, 32, image::imageops::FilterType::Triangle);
        let rgba_data = to_rgba8(resized);
        let mut data_array = [0u8; 32 * 32 * 4];
        data_array.copy_from_slice(&rgba_data);
        data_array
    };

    let big_icon_data: [u8; 64 * 64 * 4] = {
        let resized = dyn_image.resize_exact(64, 64, image::imageops::FilterType::Triangle);
        let rgba_data = to_rgba8(resized);
        let mut data_array = [0u8; 64 * 64 * 4];
        data_array.copy_from_slice(&rgba_data);
        data_array
    };

    let icon = Icon {
        small: small_icon_data,
        medium: medium_icon_data,
        big: big_icon_data,
    };

    Conf {
        window_title: "Digging Game".to_owned(),
        window_width: SCREEN_WIDTH as i32 * 4,
        window_height: SCREEN_HEIGHT as i32 * 4,
        icon: Some(icon),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;
    let mut game_renderer = GameRenderer::new().await;

    show_mouse(false);

    let render_target = render_target(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    let mut accumulator = 0.0;

    loop {
        accumulator += get_frame_time();

        // Update logic at a fixed rate of 60 FPS
        while accumulator >= FRAME_TIME {
            game.update(&game_renderer);
            accumulator -= FRAME_TIME;
        }

        let mut camera_to_render_target =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT));
        camera_to_render_target.render_target = Some(render_target.clone());
        set_camera(&camera_to_render_target);
        clear_background(SKYBLUE);
        game_renderer.draw_world(&mut game);
        set_default_camera();

        let target_aspect = SCREEN_WIDTH / SCREEN_HEIGHT;
        let screen_aspect = screen_width() / screen_height();

        let (render_width, render_height, offset_x, offset_y);
        if screen_aspect > target_aspect {
            render_height = screen_height();
            render_width = SCREEN_WIDTH * (render_height / SCREEN_HEIGHT);
            offset_x = (screen_width() - render_width) / 2.0;
            offset_y = 0.0;
        } else {
            render_width = screen_width();
            render_height = SCREEN_HEIGHT * (render_width / SCREEN_WIDTH);
            offset_x = 0.0;
            offset_y = (screen_height() - render_height) / 2.0;
        }

        clear_background(BLACK);
        draw_texture_ex(
            &render_target.texture,
            offset_x.floor(),
            offset_y.floor() + render_height.floor(),
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(render_width.floor(), -render_height.floor())),
                ..Default::default()
            },
        );

        let ui_events = game_renderer.draw_ui(&mut game);
        let mut additional_ui_events = Vec::new();

        if game.on_new_game_input_screen || game.on_warp_place_screen {
            while let Some(c) = get_char_pressed() {
                if (game.on_new_game_input_screen && (c.is_alphanumeric() || c == '_' || c == '-'))
                    || (game.on_warp_place_screen && (c as u32 >= 32 && c as u32 <= 126))
                {
                    game.input_buffer.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                game.input_buffer.pop();
            }
            if is_key_pressed(KeyCode::Enter) {
                if game.on_new_game_input_screen {
                    additional_ui_events.push(GameEvent::ConfirmNewGame(game.input_buffer.clone()));
                } else {
                    additional_ui_events
                        .push(GameEvent::ConfirmWarpGateName(game.input_buffer.clone()));
                }
            }
        }

        for event in ui_events
            .into_iter()
            .chain(additional_ui_events.into_iter())
        {
            game.handle_event(event, &game_renderer);
        }

        next_frame().await
    }
}
