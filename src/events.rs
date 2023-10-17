use std::{collections::HashSet, time::Instant};

use ggez::{
    event::EventHandler,
    glam::Vec2,
    graphics::{self, Color, DrawMode, StrokeOptions, Text, TextFragment},
    input::keyboard::KeyMods,
    winit::event::VirtualKeyCode,
    Context, GameResult,
};

use crate::{
    network::Packet,
    render::{
        center_text_in_rect, cut_bottom, cut_left, cut_right, cut_top,
        render_words_in_rect, shrink, LIGHT_TEXT_COLOR, TEXT_COLOR, WINDOW_BG,
    },
    word_game::{WordGame, GameState, GameOutcome, OngoingGame, StateTransition},
};

pub const MARGIN: f32 = 10.0;

impl EventHandler for WordGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.process_network()?;

        match self.state {
            GameState::Ongoing(ref mut ongoing) => {
                if ((Instant::now() - ongoing.last_new_word).as_secs_f32() > 0.3
                    || ongoing.current_words.len() < 5)
                    && ongoing.current_words.len() < 20
                {
                    ongoing.add_new_word(&self.word_list);
                    ongoing.last_new_word = Instant::now();
                }

                if ongoing.received_words.len() > ongoing.limit() {
                    ongoing.conn.send_packet(Packet::ILost {})?;
                    self.queue_transition(StateTransition::LoseGame);
                }
            },
            GameState::Ended { waiting_to_restart: true, opponent_waiting_to_restart: true, ..} => {
                self.queue_transition(StateTransition::RestartGame)
            }
            _ => {}
        }

        self.flush_transitions();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, WINDOW_BG);
        let draw_region = canvas.scissor_rect();
        let draw_region = shrink(draw_region, MARGIN);
        self.draw_rect = draw_region;
        
        match &self.state {
            GameState::Ongoing(ongoing) => {
                let (word_region, write_region) = cut_bottom(draw_region, 75.0);

                center_text_in_rect(
                    ctx,
                    &mut canvas,
                    &Text::new(match (&ongoing.current_text).as_str() {
                        "" => TextFragment::new("Start typing...")
                            .color(LIGHT_TEXT_COLOR)
                            .scale(70.0)
                            .font("courier_new"),
                        _ => TextFragment::new(ongoing.current_text.clone()+"|")
                            .color(TEXT_COLOR)
                            .scale(70.0)
                            .font("courier_new"),
                    }),
                    write_region,
                );

                center_text_in_rect(
                    ctx,
                    &mut canvas,
                    &Text::new(
                        TextFragment::new(format!("{:.1}wpm", ongoing.wpm()))
                            .color(TEXT_COLOR)
                            .scale(50.0)
                            .font("courier_new"),
                    ),
                    cut_right(write_region, 200.0).1,
                );

                canvas.draw(
                    &graphics::Mesh::new_rounded_rectangle(
                        ctx,
                        DrawMode::Stroke(StrokeOptions::default().with_line_width(3.0)),
                        write_region,
                        10.0,
                        Color::BLACK,
                    )
                    .unwrap(),
                    Vec2::new(0.0, 0.0),
                );

                let (current_word_region, received_word_region) =
                    cut_left(word_region, word_region.w * 0.5);
                let current_word_region = shrink(current_word_region, MARGIN);
                let received_word_region = shrink(received_word_region, MARGIN);

                let (current_word_region_header, current_word_region) =
                    cut_top(current_word_region, 80.0);
                let (received_word_region_header, received_word_region) =
                    cut_top(received_word_region, 80.0);

                center_text_in_rect(
                    ctx,
                    &mut canvas,
                    &Text::new(
                        TextFragment::new("Your Words")
                            .color(Color::BLACK)
                            .scale(80.0)
                            .font("courier_new"),
                    ),
                    current_word_region_header,
                );

                let limit = ongoing.limit();
                let exclamation_mark_count = if ongoing.received_words.len() <= limit - 5 {
                    0
                } else {
                    (ongoing.received_words.len() + 5 - limit).min(5)
                };

                center_text_in_rect(
                    ctx,
                    &mut canvas,
                    &Text::new(
                        TextFragment::new(&format!(
                            "{}/{}{}",
                            ongoing.received_words.len(),
                            limit,
                            "!".repeat(exclamation_mark_count)
                        ))
                        .color(Color::BLACK)
                        .scale(80.0)
                        .font("courier_new"),
                    ),
                    received_word_region_header,
                );

                render_words_in_rect(
                    ctx,
                    &mut canvas,
                    &ongoing.current_words,
                    current_word_region,
                    "courier_new",
                    50.0,
                    &ongoing.current_text,
                    Color::BLACK,
                );
                render_words_in_rect(
                    ctx,
                    &mut canvas,
                    &ongoing.received_words,
                    received_word_region,
                    "courier_new",
                    50.0,
                    &ongoing.current_text,
                    Color::RED,
                );
            },
            GameState::Ended { outcome, waiting_to_restart, opponent_waiting_to_restart, wpm,  .. } => {
                //TODO: Improve this screen lol

                let text = match outcome {
                    GameOutcome::Win => "You won!",
                    GameOutcome::Loss => "You lost!"
                };

                center_text_in_rect(
                    ctx,
                    &mut canvas,
                    &Text::new(
                        TextFragment::new(text)
                            .color(Color::BLACK)
                            .scale(150.0)
                            .font("courier_new"),
                    ),
                    draw_region,
                );

                center_text_in_rect(
                    ctx,
                    &mut canvas,
                    &Text::new(
                        TextFragment::new(format!(
                            "press n to change ip\n{:.2}wpm\n{}",
                            wpm,
                            match (opponent_waiting_to_restart, waiting_to_restart) {
                                (false, false) => "press r to restart",
                                (true, false) => "opponent wants to play again, press r to restart",
                                (false, true) => "waiting for opponent...",
                                (true, true) => "restarting...",
                            }
                        ))
                        .color(TEXT_COLOR)
                        .scale(50.0)
                        .font("courier_new"),
                    ),
                    cut_top(draw_region, draw_region.h / 2.0).1,
                );
            },
            GameState::ConnectionConfig { input_y, host, ip, port } => {
                let height = draw_region.h / 4.0;
                let cursor1 = if (self.create_time.elapsed().as_secs_f32() * 2.0).round() % 2.0
                    == 0.0
                    && *input_y == 1
                {
                    "|"
                } else {
                    " "
                };
                let cursor2 = if (self.create_time.elapsed().as_secs_f32() * 2.0).round() % 2.0
                    == 0.0
                    && *input_y == 2
                {
                    "|"
                } else {
                    " "
                };
                for i in [
                    (format!("host: {:?}", host), cut_top(draw_region, height).0),
                    (
                        format!("ip: {:}{}", ip, cursor1),
                        cut_top(cut_top(draw_region, height).1, height).0,
                    ),
                    (
                        format!("port: {:?}{}", port, cursor2),
                        cut_bottom(cut_bottom(draw_region, height).0, height).1,
                    ),
                    ("Start".to_owned(), cut_bottom(draw_region, height).1),
                ] {
                    center_text_in_rect(
                        ctx,
                        &mut canvas,
                        &Text::new(
                            TextFragment::new(i.0)
                                .color(TEXT_COLOR)
                                .scale(50.0)
                                .font("courier_new"),
                        ),
                        i.1,
                    )
                }
            },

            GameState::InvalidState => panic!("Leaked InvalidState!")
        }
        canvas.finish(ctx)
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: ggez::event::MouseButton,
        _x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        match self.state {
            GameState::ConnectionConfig {
                ref mut input_y,
                ref mut host,
                ..
            } => {
                let mut new_input_y = (y * 4.0 / shrink(self.draw_rect, -MARGIN).h).floor() as u32;

                if new_input_y == 0 {
                    *host = !*host;
                    new_input_y = *input_y
                }

                *input_y = new_input_y;

                if new_input_y == 3 {
                    self.pair_up_ui();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn text_input_event(
        &mut self,
        _ctx: &mut Context,
        character: char,
    ) -> Result<(), ggez::GameError> {
        match self.state {
            GameState::Ongoing(ref mut ongoing) => {
                if character.is_alphabetic() || character == ' ' {
                    ongoing.current_text.push(character);
                }
            }
            GameState::Ended { ref mut waiting_to_restart, ref mut conn, .. } => match character {
                'r' | 'R' => {
                    *waiting_to_restart = true;
                    conn.send_packet(Packet::WaitingToRestart)?;
                }
                'n' | 'N' => {
                    self.state = GameState::ConnectionConfig { input_y: 1, host: false, ip: "localhost".to_owned(), port: 5555}
                }
                _ => {}
            },
            GameState::ConnectionConfig {
                ref mut input_y,
                ref mut ip,
                ref mut port,
                ..
            } => match input_y {
                1 => {
                    if character.is_alphabetic() || character == ' ' {
                        ip.push(character);
                    }
                }
                2 => {
                    if let Ok(n) = String::from(character).parse::<u32>() {
                        let res = (*port as u32) * 10 + n;
                        if res <= u16::MAX as u32 {
                            *port = res as u16;
                        }
                    }
                }
                3 => {
                    if character == '\n' {
                        self.pair_up_ui();
                    }
                }
                _ => {
                    *input_y = 0;
                }
            },
            GameState::InvalidState => panic!("Leaked InvalidState!")
        }

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        debug!("Key Pressed! {:?}", input);
        if input
            .mods
            .intersects(KeyMods::all() & KeyMods::SHIFT.complement())
        {
            return Ok(());
        }

        match input.keycode {
            Some(VirtualKeyCode::Back) => match self.state {
                GameState::Ongoing(ref mut ongoing) => {
                    ongoing.current_text.pop();
                },
                GameState::ConnectionConfig {
                    input_y,
                    ref mut ip,
                    ref mut port,
                    ..
                } => {
                    match input_y {
                        1 => {
                            ip.pop();
                        }
                        2 => *port = (*port as f32 / 10.0).floor() as u16,
                        _ => {}
                    }
                },
                _ => {}
            },
            Some(VirtualKeyCode::Return) => {
                if let GameState::Ongoing(OngoingGame{
                    ref mut current_words,
                    ref mut received_words,
                    ref mut current_text,

                    ref mut total_words,

                    ref mut conn,
                    
                    ..
                }) = self.state {
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

                    let len_change = start_len - (current_words.len() + received_words.len());

                    *total_words += len_change as u64;
                    for word in words_to_send.iter() {
                        debug!("Sending '{}'", word);
                        conn.send_packet(Packet::add_word(&word))?;
                    }

                    current_text.clear();
                }
            }
            _ => {}
        };

        Ok(())
    }
}
