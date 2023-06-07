use bevy::prelude::*;

fn main() {
    App::new()
        // .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup() {
    // let music = asset_server.load("sounds/Windless Slopes.ogg");
    // audio.play(music);
    println!("Hello, world!");
}