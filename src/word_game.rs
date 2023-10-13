use std::{time::Instant, path::Path, str::Lines};

use ggez::{Context, graphics::FontData, GameResult};
use rand::Rng;

use crate::network::{Packet, Connection};

pub const DEFAULT_WORD_LIST: &str = "5000_out";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    Win,
    Lost
}

pub struct WordGame {
    pub start_time: Instant,
    pub total_words: u64,
    pub word_list: Vec<String>,

    pub current_words: Vec<String>,
    pub received_words: Vec<String>,
    pub last_new_word: Instant,
    pub current_text: String,
    pub words_per_min: f32,
    pub conn : Connection,

    pub status: GameStatus
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

            conn,

            status: GameStatus::Ongoing,
            start_time: Instant::now(),
            total_words: 0,
            words_per_min: 0.0,
        }
    }

    /// adds a word to currrent words
    pub fn add_new_word(&mut self) {

        let mut rng = rand::thread_rng();
        let mut idx = rng.gen_range(0..self.word_list.len());
        while self.current_words.contains(&self.word_list[idx]) {
            idx = rng.gen_range(0..self.word_list.len());
        }


        self.current_words.push(self.word_list[idx].clone());
    }

    /// detects if word has been sent, and if so adds it to list of received words
    pub fn process_network(&mut self) -> GameResult {
        loop {
            let packet = self.conn.poll_next_packet()?;

            if packet.is_none() {
                break;
            }

            match packet.unwrap() {
                Packet::AddWord { word } => {
                    self.received_words.push(word)
                },
                Packet::ILost {  } => {
                    self.status = GameStatus::Win;
                    self.current_words.clear();
                    self.received_words.clear();
                }
                _ => {}
            }
        }

        Ok(())
    }
}

