mod renderer_context;
mod globals;
mod camera;
mod time_step;
mod game;
mod system;
mod inputs;
mod file_watcher;
mod utils;
mod voxel_world;
mod egui_renderer;
mod ray;
mod color;
mod transform;
mod app;
mod gui;

use app::App;

fn main() {
    pollster::block_on(App::run());
}