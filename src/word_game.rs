use std::{time::Instant, path::Path, str::Lines, net::{TcpListener, TcpStream}};

use ggez::{Context, graphics::{FontData, Rect}, GameResult, event::EventHandler};
use rand::Rng;

use crate::network::{Packet, Connection, connect_to_dummy};

pub const DEFAULT_WORD_LIST: &str = "5000_out";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    Win,
    Lost,
    InputData{input_y: u32, host: bool, ip: String, port: u32},
}

pub struct WordGame {
    pub start_time: Instant,
    pub total_words: u64,
    pub word_list: Vec<String>,
    pub opponent_waiting_to_restart: bool,
    pub waiting_to_restart: bool,
    pub current_words: Vec<String>,
    pub received_words: Vec<String>,
    pub last_new_word: Instant,
    pub current_text: String,
    pub words_per_min: f32,
    pub conn : Option<Connection>,
    pub draw_rect: Rect,
    pub status: GameStatus
}

impl WordGame {
    pub fn new(ctx: &mut Context, word_list: &str) -> WordGame {
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



        ctx.gfx.add_font("courier_new", FontData::from_path(ctx, "/font/cour.ttf").unwrap());

        WordGame {
            word_list: words,
            waiting_to_restart: false,
            current_words: vec![],
            received_words: vec![],
            draw_rect: Rect::one(),
            last_new_word: Instant::now(),

            current_text: String::new(),

            conn: None,
            #[cfg(not(debug_assertions))]
            status: GameStatus::InputData { input_y: 1, host: false, ip: "localhost".to_owned(), port: 5555 },
            
            #[cfg(debug_assertions)]
            status: GameStatus::InputData { input_y: 1, host: false, ip: "bot".to_owned(), port: 5555 },

            start_time: Instant::now(),
            total_words: 0,
            words_per_min: 0.0,

            opponent_waiting_to_restart: false,
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
            if self.conn.is_none() {break}
            let packet = self.conn.as_mut().unwrap().poll_next_packet()?;

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
                Packet::WaitingToRestart { } => {
                    self.opponent_waiting_to_restart = true;
                    if self.waiting_to_restart {
                        self.reset();
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) {
            self.current_words = vec![];
            self.received_words = vec![];
            self.last_new_word = Instant::now();
            self.current_text = String::new();
            self.status = GameStatus::Ongoing;
            self.start_time = Instant::now();
            self.total_words = 0;
            self.words_per_min = 0.0;
            self.opponent_waiting_to_restart = false;
            self.waiting_to_restart = false;
    }

    pub fn connect_game(&self) {

    }

    pub fn pair_up_ui(&mut self) {
        
    let (host, ip, port) = match &self.status {
        GameStatus::InputData { input_y: _, host, ip, port } => (
            host,
            ip,
            port,
        ),
        _ => {
            println!("failed to connect, status is {:?}", self.status);
            return}
    };

    
    let ip = ip.to_string();

    let mut res:Option<TcpStream> = None;


    if *host {
        println!("Waiting for connection on {}:{}", ip, port);

        if let Ok(listener) = TcpListener::bind((ip, *port as u16)) {
            if let Ok((stream, addr)) = listener.accept() {
                
                println!("Got connection from {}", addr);
        
                res = Some(stream)
            }
        }

    } else {
        println!("Attempting to connect to {}:{}", ip, port);

        res = TcpStream::connect((ip, *port as u16)).ok();

        println!("Connected!");

        
    }

    if let Some(stream) = res {
        self.conn = Connection::new(stream).ok();
        self.status = GameStatus::Ongoing;
        self.reset();
        println!("{:?}", self.status);
    }

    if matches!(self.status.clone(), GameStatus::InputData { input_y:_addr, host:_, ip, port:_ } if ip == "bot") {
        self.conn = connect_to_dummy().ok();
        self.status = GameStatus::Ongoing;
        self.reset();
    }


    }
}

