use std::{time::Instant, path::Path, str::Lines, net::{TcpListener, TcpStream}};

use ggez::{Context, graphics::{FontData, Rect}, GameResult};
use rand::Rng;

use crate::network::{Packet, Connection, connect_to_dummy};

pub const DEFAULT_WORD_LIST: &str = "5000_out";

#[derive(Debug, Clone, Copy)]
pub enum GameOutcome {
    Win, Loss
}

#[derive(Debug)]
pub struct OngoingGame {
    pub start_time: Instant,
    pub total_words: u64,
    
    pub current_words: Vec<String>,
    pub received_words: Vec<String>,

    pub last_new_word: Instant,
    pub current_text: String,

    pub conn: Connection
}

#[derive(Debug)]
pub enum GameState {
    //InterState is an invalid state. It is needed to be able to move values out of one state (with std::mem::take) to put them into the new state
    InvalidState,
    Ongoing(OngoingGame),
    Ended {
        outcome: GameOutcome,
        wpm: f32,

        waiting_to_restart: bool,
        opponent_waiting_to_restart: bool,

        conn: Connection
    },
    ConnectionConfig {
        input_y: u32,
        host: bool,
        ip: String,
        port: u16
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::InvalidState
    }
}

//For state transitions that require moving out of the current state
#[derive(Debug)]
pub enum StateTransition {
    WinGame, LoseGame,
    RestartGame
}

pub struct WordGame {
    pub create_time: Instant,
    pub word_list: Vec<String>,
    pub draw_rect: Rect,
    pub state: GameState,

    queued_transitions: Vec<StateTransition>
}

const WORD_LIMIT: usize = 20;

impl OngoingGame {
    pub fn add_new_word(&mut self, list: &Vec<String>) {
        let mut rng = rand::thread_rng();
        let mut idx = rng.gen_range(0..list.len());
        while self.current_words.contains(&list[idx]) {
            idx = rng.gen_range(0..list.len());
        }

        self.current_words.push(list[idx].clone());
    }

    pub fn wpm(&self) -> f32 {
        self.total_words as f32 / self.start_time.elapsed().as_secs_f32()
    }

    pub fn limit(&self) -> usize {
        (WORD_LIMIT*2 - (self.start_time.elapsed().as_secs() as usize / (120 / WORD_LIMIT))).min(WORD_LIMIT)
    }
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



        ctx.gfx.add_font(
            "courier_new", 
            match FontData::from_path(ctx, "C:/Windows/Fonts/cour.ttf") {
                Ok(a) => a,
                _ => FontData::from_path(ctx, "/font/FiraCode-VariableFont_wght.ttf").unwrap()
            }
        );

        WordGame {
            create_time: Instant::now(),
            word_list: words,
            draw_rect: Rect::one(),
            #[cfg(not(debug_assertions))]
            state: GameState::ConnectionConfig { input_y: 1, host: false, ip: "localhost".to_owned(), port: 5555 },
            
            #[cfg(debug_assertions)]
            state: GameState::ConnectionConfig { input_y: 1, host: false, ip: "bot".to_owned(), port: 5555 },

            queued_transitions: vec![]
        }
    }

    pub fn queue_transition(&mut self, transition: StateTransition) {
        self.queued_transitions.push(transition);
    }

    pub fn flush_transitions(&mut self) {
        let Self{state, queued_transitions, ..} = self;

        for transition in queued_transitions.iter() {
            let prev_state = std::mem::take(state);
            *state = match (transition, prev_state) {
                (StateTransition::WinGame, GameState::Ongoing(ongoing)) => {
                    GameState::Ended {
                        outcome: GameOutcome::Win, 
                        wpm: ongoing.wpm(), 
                        waiting_to_restart: false, 
                        opponent_waiting_to_restart: false, 
                        conn: ongoing.conn
                    }
                },
                (StateTransition::LoseGame, GameState::Ongoing(ongoing)) => {
                    GameState::Ended {
                        outcome: GameOutcome::Loss, 
                        wpm: ongoing.wpm(), 
                        waiting_to_restart: false, 
                        opponent_waiting_to_restart: false, 
                        conn: ongoing.conn
                    }
                },
                (StateTransition::RestartGame, GameState::Ended { conn, .. }) => {
                    GameState::Ongoing(OngoingGame {
                        start_time: Instant::now(), 
                        total_words: 0, 
                        current_words: vec![], 
                        received_words: vec![], 
                        last_new_word: Instant::now(), 
                        current_text: String::new(), 
                        conn
                    })
                }
                (t, s) => panic!("Invalid transition {:?} for state {:?}", t, s)
            };
        }

        queued_transitions.clear();
    }

    /// detects if word has been sent, and if so adds it to list of received words
    pub fn process_network(&mut self) -> GameResult {
        match self.state {
            GameState::Ongoing(ref mut ongoing) => {
                loop {
                    let packet = ongoing.conn.poll_next_packet()?;

                    match packet {
                        None => break,
                        Some(Packet::AddWord { word }) => {
                            ongoing.received_words.push(word);
                        },
                        Some(Packet::ILost {  }) => {
                            self.queue_transition(StateTransition::WinGame);
                            break;
                        },

                        Some(p) => {
                            warn!("Unexpected packed {:?} received in ongoing state!", p)
                        }
                    }
                }
            },
            GameState::Ended { ref mut opponent_waiting_to_restart, ref mut conn, .. } => {
                loop {
                    let packet = conn.poll_next_packet()?;

                    match packet {
                        None => break,
                        Some(Packet::WaitingToRestart) => {
                            *opponent_waiting_to_restart = true;
                        }

                        Some(p) => {
                            warn!("Unexpected packet {:?} received in ongoing state!", p)
                        }
                    }
                }
            },
            _ => {}
        }

        self.flush_transitions();

        Ok(())
    }

    pub fn pair_up_ui(&mut self) {
        let conn = match &self.state {
            GameState::ConnectionConfig { host: true, ip, .. } if ip == "bot" => {
                connect_to_dummy().ok()
            },
            GameState::ConnectionConfig { host: true, ip, port, .. } => {
                if let Ok(listener) = TcpListener::bind((ip.as_str(), *port)) {
                    if let Ok((stream, addr)) = listener.accept() {
                        info!("Got connection from {}", addr);
                        Connection::new(stream).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            GameState::ConnectionConfig { host: false, ip, port, .. } => {
                info!("Attempting to connect to {}:{}", ip, port);
                if let Ok(stream) = TcpStream::connect((ip.as_str(), *port)) {
                    Connection::new(stream).ok()
                } else {
                    None
                }
            },
            other => {
                error!("Invalid state for pairing up! {:?}", other);
                None
            }
        };
            
        if let Some(conn) = conn {
            info!("Connected!");
            self.state = GameState::Ongoing(OngoingGame {
                start_time: Instant::now(), 
                total_words: 0, 
                current_words: vec![], 
                received_words: vec![], 
                last_new_word: Instant::now(), 
                current_text: String::new(), 
                conn
            });
        } else {
            error!("Failed to connect!");
        }
    }
}

