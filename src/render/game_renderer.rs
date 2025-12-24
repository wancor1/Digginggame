use super::ui::UIRenderer;
use super::world_renderer::WorldRenderer;
use crate::Game;
use crate::events::GameEvent;
use ::rand::Rng;
use macroquad::prelude::*;
use macroquad::text::load_ttf_font_from_bytes;

pub struct GameRenderer {
    atlas: Option<Texture2D>,
    atlas_image: Option<macroquad::texture::Image>,
    font: Option<Font>,
}

impl GameRenderer {
    pub async fn new() -> Self {
        // Load Assets
        let atlas_bytes = include_bytes!("../../src/atlas.png");
        let dynamic_image = image::load_from_memory(atlas_bytes).unwrap();
        let rgba_image = dynamic_image.to_rgba8();

        let mq_image = macroquad::texture::Image {
            width: rgba_image.width() as u16,
            height: rgba_image.height() as u16,
            bytes: rgba_image.into_raw(),
        };
        let atlas = Some(Texture2D::from_image(&mq_image));
        atlas.as_ref().unwrap().set_filter(FilterMode::Nearest);

        let font_bytes = include_bytes!("../../src/misaki_gothic.ttf");
        let font = Some(load_ttf_font_from_bytes(font_bytes).unwrap());

        Self {
            atlas,
            atlas_image: Some(mq_image),
            font,
        }
    }

    pub fn get_random_pixel_color(&self, rect: Rect) -> Color {
        if let Some(atlas_image) = &self.atlas_image {
            let img_width = atlas_image.width as usize;
            let img_height = atlas_image.height as usize;

            let rect_x_start = rect.x as usize;
            let rect_y_start = rect.y as usize;
            let rect_x_end = (rect.x + rect.w) as usize;
            let rect_y_end = (rect.y + rect.h) as usize;

            let mut rng = ::rand::rng();

            if rect_x_start >= img_width || rect_y_start >= img_height {
                return WHITE;
            }

            let valid_x_range_start = rect_x_start;
            let valid_x_range_end = (rect_x_end).min(img_width);
            let valid_y_range_start = rect_y_start;
            let valid_y_range_end = (rect_y_end).min(img_height);

            if valid_x_range_start >= valid_x_range_end || valid_y_range_start >= valid_y_range_end
            {
                return WHITE;
            }

            let rand_x = rng.random_range(valid_x_range_start..valid_x_range_end);
            let rand_y = rng.random_range(valid_y_range_start..valid_y_range_end);

            let index = (rand_y * img_width + rand_x) * 4;

            if index + 3 < atlas_image.bytes.len() {
                let r = atlas_image.bytes[index] as u32;
                let g = atlas_image.bytes[index + 1] as u32;
                let b = atlas_image.bytes[index + 2] as u32;
                let a = atlas_image.bytes[index + 3] as u32;
                Color::new(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                )
            } else {
                WHITE
            }
        } else {
            WHITE
        }
    }

    pub fn get_font(&self) -> Option<&Font> {
        self.font.as_ref()
    }

    pub fn draw_world(&mut self, game: &mut Game) {
        WorldRenderer::draw(game, self.atlas.as_ref());
    }

    pub fn draw_ui(&mut self, game: &mut Game) -> Vec<GameEvent> {
        UIRenderer::draw(game, self.font.as_ref(), self.atlas.as_ref())
    }
}
