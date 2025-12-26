#![allow(
    clippy::similar_names,
    clippy::suboptimal_flops,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::multiple_crate_versions,
    clippy::cast_lossless,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::unnecessary_map_or,
    clippy::map_unwrap_or,
    clippy::new_without_default,
    clippy::items_after_statements,
    clippy::branches_sharing_code
)]

pub mod components;
pub mod constants;
pub mod events;
pub mod game;
pub mod managers;
pub mod render;
pub mod ui;
pub mod utils;

pub use crate::constants::*;
pub use crate::events::GameEvent;
pub use crate::game::Game;
pub use crate::render::game_renderer::GameRenderer;
