use std::{collections::HashSet, time::Instant};

use ggez::{Context, winit::event::VirtualKeyCode, input::keyboard::KeyMods, graphics::{TextFragment, Text, Color, StrokeOptions, self, DrawMode}, glam::Vec2, GameResult, event::EventHandler};

use crate::{render::{render_words_in_rect, center_text_in_rect, shrink, cut_top, cut_left, cut_bottom, WINDOW_BG, TEXT_COLOR, LIGHT_TEXT_COLOR, lerp_color, cut_right}, network::Packet, word_game::{WordGame, GameStatus}};

const WORD_LIMIT: usize = 20;

impl EventHandler for WordGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.status == GameStatus::Ongoing {
            if ((Instant::now() - self.last_new_word).as_secs_f32() > 0.3 || self.current_words.len() < 5)
            && self.current_words.len() < 20
            {
                self.add_new_word();
                self.last_new_word = Instant::now();
            }
        }

        self.process_network()?;

        if self.received_words.len() > WORD_LIMIT {
            self.status = GameStatus::Lost;
            self.conn.send_packet(Packet::ILost {})?;
            self.received_words.clear();
            self.current_words.clear();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, WINDOW_BG);
        let draw_region = canvas.scissor_rect();

        const MARGIN: f32 = 10.0;

        let draw_region = shrink(draw_region, MARGIN);


        if self.status == GameStatus::Ongoing {
            let (word_region, write_region) = cut_bottom(draw_region, 75.0);

            center_text_in_rect(ctx, &mut canvas, &Text::new(
                match (self.current_text).as_str() {
                "" => TextFragment::new("Start typing...")
                    .color(LIGHT_TEXT_COLOR)
                    .scale(70.0)
                    .font("courier_new"),
                _ => TextFragment::new(&self.current_text)
                    .color(TEXT_COLOR)
                    .scale(70.0)
                    .font("courier_new")}
                ),
            write_region);

            center_text_in_rect(ctx, &mut canvas, &Text::new(
                TextFragment::new(format!("{:.2}wpm",self.words_per_min))
                    .color(TEXT_COLOR)
                    .scale(50.0)
                    .font("courier_new")),
            cut_right(write_region,200.0).1);

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
                TextFragment::new("Your Words")
                    .color(Color::BLACK)
                    .scale(80.0)
                    .font("courier_new")
            ), current_word_region_header);

            let exclamation_mark_count = if self.received_words.len() <= WORD_LIMIT - 5 {
                0
            } else {
                (self.received_words.len() + 5 - WORD_LIMIT).max(5)
            };

            center_text_in_rect(ctx, &mut canvas, &Text::new(
                TextFragment::new(&format!("{}/{}{}", self.received_words.len(), WORD_LIMIT, "!".repeat(exclamation_mark_count)))
                    .color(Color::BLACK)
                    .scale(80.0)
                    .font("courier_new")
            ), received_word_region_header);

            render_words_in_rect(ctx, &mut canvas, &self.current_words, current_word_region, "courier_new", 50.0, &self.current_text, Color::BLACK);
            render_words_in_rect(ctx, &mut canvas, &self.received_words, received_word_region, "courier_new", 50.0, &self.current_text, Color::RED);
        } else {
            //TODO: Improve this screen lol

            let text = match self.status {
                GameStatus::Win => "You won!",
                GameStatus::Lost => "You lost!",
                _ => unreachable!()
            };

            center_text_in_rect(ctx, &mut canvas, &Text::new(
                TextFragment::new(text)
                    .color(Color::BLACK)
                    .scale(150.0)
                    .font("courier_new")
            ), draw_region);
            
            center_text_in_rect(ctx, &mut canvas, &Text::new(
                TextFragment::new(format!("{:.2}wpm",self.words_per_min))
                    .color(TEXT_COLOR)
                    .scale(50.0)
                    .font("courier_new"),
            ), cut_top(draw_region, draw_region.h/2.0).1);

        }

        canvas.finish(ctx)
    }

    fn text_input_event(&mut self, _ctx: &mut Context, character: char) -> Result<(), ggez::GameError> {
        if self.status != GameStatus::Ongoing { return Ok(()); }
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
        if self.status != GameStatus::Ongoing { return Ok(()); }

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

                let start_len = current_words.len() + received_words.len();

                current_words.retain(move |w| w.to_lowercase() != *lower);
                received_words.retain(move |w| w.to_lowercase() != *lower);


                let len_change = start_len -(current_words.len() + received_words.len());

                self.total_words += len_change as u64;

                self.words_per_min = self.total_words as f32/(self.start_time.elapsed().as_secs_f32()/60.0);


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