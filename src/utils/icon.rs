use image::{ImageFormat, load_from_memory_with_format};
use miniquad::conf::Icon;

/// Loads the game icon from an embedded PNG file.
///
/// # Panics
///
/// This function will panic if the embedded icon image cannot be loaded or decoded as a PNG.
#[must_use]
pub fn load_game_icon() -> Icon {
    let icon_bytes = include_bytes!("../icon.png");
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

    Icon {
        small: small_icon_data,
        medium: medium_icon_data,
        big: big_icon_data,
    }
}
