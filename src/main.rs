use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{PathBuf, Path};
use std::str::Lines;
use std::time::Instant;

use ggez::conf::{WindowSetup, WindowMode, NumSamples};
use ggez::glam::Vec2;
use ggez::input::keyboard::KeyMods;
use ggez::winit::event::VirtualKeyCode;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, FontData, Text, Rect, StrokeOptions, Canvas, Drawable, TextFragment, DrawMode};
use ggez::event::{self, EventHandler};
use network::{connect, Connection, Packet};
use rand::Rng;

pub mod network;

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

struct WordGame {
    word_list: Vec<String>,

    current_words: Vec<String>,
    received_words: Vec<String>,

    last_new_word: Instant,

    current_text: String,

    conn : Connection
}

impl WordGame {
    pub fn new(ctx: &mut Context, word_list: &str, conn: Connection) -> WordGame {
        ctx.fs.mount(Path::new("./res"), true);

        let words = std::io::read_to_string(ctx.fs.open(format!("/words/{word_list}.txt")).unwrap()).unwrap();
        let words: Lines = words.lines();
        let words = words.filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.len() == 0 { return None; }

            let first_char: String = trimmed.chars().nth(0).unwrap().to_uppercase().collect();
            let rest: String = trimmed.chars().skip(1).flat_map(|c| c.to_lowercase()).collect();

            Some(format!("{}{}", first_char, rest))
        }).collect();

        for word in &words {
            println!("{}", word);
        }


        ctx.gfx.add_font("courier_new", FontData::from_path(ctx, "/font/cour.ttf").unwrap());

        WordGame {
            word_list: words,

            current_words: vec![],
            received_words: vec![],

            last_new_word: Instant::now(),

            current_text: String::new(),

            conn
        }
    }

    fn add_new_word(&mut self) {
        let mut rng = rand::thread_rng();

        let idx = rng.gen_range(0..self.word_list.len());
        self.current_words.push(self.word_list[idx].clone());
    }

    fn process_network(&mut self) -> GameResult {
        loop {
            let packet = self.conn.poll_next_packet()?;

            if packet.is_none() {
                break;
            }

            match packet.unwrap() {
                Packet::AddWord { word } => {
                    self.received_words.push(word)
                }
                _ => {}
            }
        }

        Ok(())
    }
}

fn shrink(rect: Rect, v: f32) -> Rect {
    Rect { x: rect.x + v, y: rect.y + v, w: rect.w - 2.0 * v, h: rect.h - 2.0 * v }
}

fn cut_bottom(rect: Rect, height: f32) -> (Rect, Rect) {
    (
        Rect {x: rect.x, y: rect.y, w: rect.w, h: rect.h - height},
        Rect {x: rect.x, y: rect.y + rect.h - height, w: rect.w, h: height}
    )
}

fn cut_top(rect: Rect, height: f32) -> (Rect, Rect) {
    (
        Rect {x: rect.x, y: rect.y, w: rect.w, h: height},
        Rect {x: rect.x, y: rect.y + height, w: rect.w, h: rect.h - height}
    )
}

fn cut_left(rect: Rect, width: f32) -> (Rect, Rect) {
    (
        Rect {x: rect.x, y: rect.y, w: width, h: rect.h},
        Rect {x: rect.x + width, y: rect.y, w: rect.w - width, h: rect.h}
    )
}

fn render_words_in_rect(ctx: &mut Context, canvas: &mut Canvas, words: &Vec<String>, rect: Rect, font: &str, font_size: f32, cross_out: &str, color: Color) {
    let prev = canvas.scissor_rect();
    canvas.set_scissor_rect(rect).unwrap();

    let word_width: f32 = font_size * 6.0;
    let word_height: f32 = font_size * 0.75;

    let num_columns = ((rect.w / word_width).floor() as i32).max(1);

    let texts: Vec<(Text, &String)> = words.iter().map(move |w| {
        let mut text = Text::new(TextFragment::new(w.clone()).color(color));
        text.set_font(font);
        text.set_scale(font_size);

        (text, w)
    }).collect();

    let column_width = rect.w / num_columns as f32;

    let mut x = rect.x;
    let mut y = rect.y;

    let cross_out_lower = cross_out.to_lowercase();

    for (text, word) in texts {
        let end_y = y + word_height;

        if end_y > rect.y + rect.h {
            y = rect.y;
            x += word_width;
        }

        canvas.draw(&text, Vec2::new(x, y));

        if word.to_lowercase().starts_with(&cross_out_lower) && cross_out_lower.len() > 0 {
            let positions = text.glyph_positions(ctx).unwrap();
            let dimensions = text.dimensions(ctx).unwrap();
            
            let cross_out_len = if cross_out_lower.len() == word.len() {
                dimensions.w
            } else {
                positions[cross_out_lower.len()].x
            };

            canvas.draw(
                &graphics::Mesh::new_line(
                    ctx,
                    &[
                        Vec2::new(0.0, dimensions.h * 0.5),
                        Vec2::new(cross_out_len, dimensions.h * 0.5)
                    ],
                    5.0,
                    Color::GREEN
                ).unwrap(),
                Vec2::new(x, y)
            );
        }

        y += word_height;
    }

    canvas.set_scissor_rect(prev).unwrap();
}

fn center_text_in_rect(ctx: &mut Context, canvas: &mut Canvas, text: &Text, rect: Rect) {
    let prev = canvas.scissor_rect();
    canvas.set_scissor_rect(rect).unwrap();

    let dimensions = text.dimensions(ctx).unwrap();

    let x = (rect.w - dimensions.w) / 2.0;
    let y = (rect.h - dimensions.h) / 2.0;

    canvas.draw(text, Vec2::new(rect.x + x, rect.y + y));

    canvas.set_scissor_rect(prev).unwrap();
}

impl EventHandler for WordGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if (Instant::now() - self.last_new_word).as_secs_f32() > 2.0 || self.current_words.len() < 5 {
            self.add_new_word();
            self.last_new_word = Instant::now();
        }

        self.process_network()?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(100, 100, 100));
        let draw_region = canvas.scissor_rect();

        const MARGIN: f32 = 10.0;

        let draw_region = shrink(draw_region, MARGIN);

        let (word_region, write_region) = cut_bottom(draw_region, 75.0);

        center_text_in_rect(ctx, &mut canvas, &Text::new(
            TextFragment::new(&self.current_text)
                .color(Color::GREEN)
                .scale(70.0)
                .font("courier_new")
        ), write_region);

        canvas.draw(
            &graphics::Mesh::new_rounded_rectangle(
                ctx, 
                DrawMode::Stroke(
                    StrokeOptions::default()
                        .with_line_width(3.0)
                ),
                write_region,
                10.0,
                Color::BLACK
            ).unwrap(),
            Vec2::new(0.0, 0.0)
        );

        let (current_word_region, received_word_region) = cut_left(word_region, word_region.w * 0.5);
        let current_word_region = shrink(current_word_region, MARGIN);
        let received_word_region = shrink(received_word_region, MARGIN);

        let (current_word_region_header, current_word_region) = cut_top(current_word_region, 80.0);
        let (received_word_region_header, received_word_region) = cut_top(received_word_region, 80.0);

        center_text_in_rect(ctx, &mut canvas, &Text::new(
            TextFragment::new("Ollie's Corn Farm")
                .color(Color::BLACK)
                .scale(80.0)
                .font("courier_new")
        ), current_word_region_header);

        center_text_in_rect(ctx, &mut canvas, &Text::new(
            TextFragment::new("Ollie's Corn Farm")
                .color(Color::BLACK)
                .scale(80.0)
                .font("courier_new")
        ), received_word_region_header);

        render_words_in_rect(ctx, &mut canvas, &self.current_words, current_word_region, "courier_new", 50.0, &self.current_text, Color::WHITE);
        render_words_in_rect(ctx, &mut canvas, &self.received_words, received_word_region, "courier_new", 50.0, &self.current_text, Color::RED);

        // Draw code here...
        canvas.finish(ctx)
    }

    fn text_input_event(&mut self, _ctx: &mut Context, character: char) -> Result<(), ggez::GameError> {
        if character.is_alphabetic() || character == ' ' {
            self.current_text.push(character);
        }

        Ok(())
    }

    fn key_down_event(
            &mut self,
            _ctx: &mut Context,
            input: ggez::input::keyboard::KeyInput,
            _repeated: bool,
        ) -> Result<(), ggez::GameError> {
        println!("Key Pressed! {:?}", input);
        if input.mods.intersects(KeyMods::all() & KeyMods::SHIFT.complement()) {
            return Ok(())
        }

        match input.keycode {
            Some(VirtualKeyCode::Back) => {self.current_text.pop();},
            Some(VirtualKeyCode::Return) => {
                let Self {ref mut current_words, ref mut received_words, current_text, ..} = self;
                let lower = current_text.to_lowercase();
                let lower = &lower;

                let mut words_to_send = HashSet::new();
                
                for word in current_words.iter() {
                    if word.to_lowercase() == *lower {
                        words_to_send.insert(word.clone());
                    }
                }

                current_words.retain(move |w| w.to_lowercase() != *lower);
                received_words.retain(move |w| w.to_lowercase() != *lower);

                for word in words_to_send.iter() {
                    println!("Sending '{}'", word);
                    self.conn.send_packet(Packet::add_word(&word))?;
                }
                
                self.current_text.clear();
            }
            _ => {}
        };

        Ok(())
    }
}