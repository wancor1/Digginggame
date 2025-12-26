use super::ui::UIRenderer;
use super::world_renderer::WorldRenderer;
use crate::Game;
use crate::events::GameEvent;
use ::rand::Rng;
use macroquad::prelude::*;
use macroquad::text::load_ttf_font_from_bytes;
use num_traits::ToPrimitive;

pub struct GameRenderer {
    atlas: Option<Texture2D>,
    atlas_image: Option<macroquad::texture::Image>,
    font: Option<Font>,
}

impl GameRenderer {
    /// Creates a new `GameRenderer` and loads the necessary assets.
    ///
    /// # Panics
    ///
    /// Panics if the atlas image or font cannot be loaded.
    pub fn new() -> Self {
        // Load Assets
        let atlas_bytes = include_bytes!("../../src/atlas.png");
        let dynamic_image =
            image::load_from_memory(atlas_bytes).expect("Failed to load atlas image from memory");
        let rgba_image = dynamic_image.to_rgba8();

        let mq_image = macroquad::texture::Image {
            width: rgba_image.width().to_u16().unwrap_or(0),
            height: rgba_image.height().to_u16().unwrap_or(0),
            bytes: rgba_image.into_raw(),
        };
        let atlas = Texture2D::from_image(&mq_image);
        atlas.set_filter(FilterMode::Nearest);

        let font_bytes = include_bytes!("../../src/misaki_gothic.ttf");
        let font =
            Some(load_ttf_font_from_bytes(font_bytes).expect("Failed to load font from bytes"));

        Self {
            atlas: Some(atlas),
            atlas_image: Some(mq_image),
            font,
        }
    }

    pub fn get_random_pixel_color(&self, rect: Rect) -> Color {
        if let Some(atlas_image) = &self.atlas_image {
            let img_width = atlas_image.width.to_usize().unwrap_or(0);
            let img_height = atlas_image.height.to_usize().unwrap_or(0);

            let rect_x_start = rect.x.to_usize().unwrap_or(0);
            let rect_y_start = rect.y.to_usize().unwrap_or(0);
            let rect_x_end = (rect.x + rect.w).to_usize().unwrap_or(0);
            let rect_y_end = (rect.y + rect.h).to_usize().unwrap_or(0);

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
                let r = atlas_image.bytes[index].to_u32().unwrap_or(0);
                let g = atlas_image.bytes[index + 1].to_u32().unwrap_or(0);
                let b = atlas_image.bytes[index + 2].to_u32().unwrap_or(0);
                let a = atlas_image.bytes[index + 3].to_u32().unwrap_or(0);
                Color::new(
                    r.to_f32().unwrap_or(0.0) / 255.0,
                    g.to_f32().unwrap_or(0.0) / 255.0,
                    b.to_f32().unwrap_or(0.0) / 255.0,
                    a.to_f32().unwrap_or(0.0) / 255.0,
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
