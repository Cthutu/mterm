//
// Matt's Terminal
// Provides an interface for rendering ASCII text quickly on a window
//

mod app;
mod builder;
mod colour;
mod main_loop;
mod present;
mod render;
mod result;

pub use app::*;
pub use builder::*;
pub use colour::*;
pub use main_loop::*;
pub use present::*;
pub use render::*;
pub use result::*;
