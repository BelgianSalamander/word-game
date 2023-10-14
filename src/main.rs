#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ggez::conf::{WindowSetup, WindowMode, NumSamples};
use ggez::ContextBuilder;
use ggez::event;

pub mod network;
pub mod word_game;
pub mod render;
pub mod events;

use word_game::*;

fn main() {
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

    let my_game = WordGame::new(&mut ctx, DEFAULT_WORD_LIST);

    // Run!
    event::run(ctx, event_loop, my_game);
}
