use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{PathBuf, Path};
use std::str::Lines;
use std::time::Instant;

use ggez::conf::{WindowSetup, WindowMode, NumSamples};
use ggez::glam::Vec2;
use ggez::winit::event::VirtualKeyCode;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use network::{connect};
use rand::Rng;

pub mod network;
pub mod word_game;
pub mod render;
pub mod events;

use word_game::*;

fn main() {
    let mut conn = connect().unwrap();

    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .window_mode(
            WindowMode::default()
                .resizable(true)   
        )
        .window_setup(
            WindowSetup::default()
                .samples(NumSamples::Four)
        )
        .build()
        .expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = WordGame::new(&mut ctx, "basic", conn);

    // Run!
    event::run(ctx, event_loop, my_game);
}
