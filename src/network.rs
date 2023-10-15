use std::net::{TcpStream, TcpListener};
use std::io::{self, Read, ErrorKind, Write};
use std::{thread, fs};
use std::time::{Duration, Instant};

use rand::Rng;

use crate::word_game::DEFAULT_WORD_LIST;

type VersionType = u16;
const MAJOR_VERSION: VersionType = 0;
const MINOR_VERSION: VersionType = 2;

trait FriendlyRead {
    fn read_u8(&mut self) -> io::Result<u8>;
    fn read_u16(&mut self) -> io::Result<u16>;
    fn read_u32(&mut self) -> io::Result<u32>;
    fn read_u64(&mut self) -> io::Result<u64>;

    fn read_i8(&mut self) -> io::Result<i8>;
    fn read_i16(&mut self) -> io::Result<i16>;
    fn read_i32(&mut self) -> io::Result<i32>;
    fn read_i64(&mut self) -> io::Result<i64>;
    
    fn read_f32(&mut self) -> io::Result<f32>;
    fn read_f64(&mut self) -> io::Result<f64>;

    fn read_string(&mut self) -> io::Result<String>;
}

impl<T: Read> FriendlyRead for T {
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(u8::from_be_bytes(buf))
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }

    fn read_i8(&mut self) -> io::Result<i8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(i8::from_be_bytes(buf))
    }

    fn read_i16(&mut self) -> io::Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_i64(&mut self) -> io::Result<i64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }

    fn read_f64(&mut self) -> io::Result<f64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_be_bytes(buf))
    }

    fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_u32()? as usize;
        
        let mut bytes = vec![];
        bytes.resize(length, 0);
        self.read_exact(&mut bytes)?;

        match String::from_utf8(bytes) {
            Ok(x) => Ok(x),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))
        }
    }
}

trait FriendlyWrite {
    fn write_u8(&mut self, x: u8) -> io::Result<()>;
    fn write_u16(&mut self, x: u16) -> io::Result<()>;
    fn write_u32(&mut self, x: u32) -> io::Result<()>;
    fn write_u64(&mut self, x: u64) -> io::Result<()>;

    fn write_i8(&mut self, x: i8) -> io::Result<()>;
    fn write_i16(&mut self, x: i16) -> io::Result<()>;
    fn write_i32(&mut self, x: i32) -> io::Result<()>;
    fn write_i64(&mut self, x: i64) -> io::Result<()>;
    
    fn write_f32(&mut self, x: f32) -> io::Result<()>;
    fn write_f64(&mut self, x: f64) -> io::Result<()>;

    fn write_string(&mut self, x: &str) -> io::Result<()>;
}

impl<T: Write> FriendlyWrite for T {
    fn write_u8(&mut self, x: u8) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_u16(&mut self, x: u16) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_u32(&mut self, x: u32) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_u64(&mut self, x: u64) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_i8(&mut self, x: i8) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_i16(&mut self, x: i16) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_i32(&mut self, x: i32) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_i64(&mut self, x: i64) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_f32(&mut self, x: f32) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_f64(&mut self, x: f64) -> io::Result<()> {
        self.write_all(&x.to_be_bytes())
    }

    fn write_string(&mut self, x: &str) -> io::Result<()> {
        self.write_all(&(x.len() as u32).to_be_bytes())?;
        self.write_all(x.as_bytes())?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Packet {
    ClientInfo { // 0
        major: VersionType,
        minor: VersionType
    },

    AddWord {
        word: String
    },

    ILost {

    },

    WaitingToRestart
}

impl Packet {
    pub fn parse<T: Read>(mut data: T) -> io::Result<Self> {
        let packet_type = data.read_u16()?;

        let packet = match packet_type {
            0 => Self::ClientInfo { 
                major: data.read_u16()?, 
                minor: data.read_u16()?
            },

            1 => Self::AddWord { 
                word: data.read_string()?
            },

            2 => Self::ILost {

            },

            3 => Self::WaitingToRestart,

            x => {
                return Err(io::Error::new(io::ErrorKind::Other, format!("Unrecognised packet type {}", x)));
            }
        };

        Ok(packet)
    }

    fn packet_id(&self) -> u16 {
        match self {
            Self::ClientInfo {..}  => 0,
            Self::AddWord {..}     => 1,
            Self::ILost {..}       => 2,
            Self::WaitingToRestart => 3
        }
    }

    pub fn write<T: Write>(&self, mut out: T) -> io::Result<()> {
        out.write_u16(self.packet_id())?;

        match self {
            Self::ClientInfo { major, minor } => {
                out.write_u16(*major)?;
                out.write_u16(*minor)?;
            },

            Self::AddWord { word } => {
                out.write_string(word)?;
            },

            Self::ILost { } => {

            },

            Self::WaitingToRestart => {}
        }

        Ok(())
    }

    pub fn client_info() -> Packet {
        Self::ClientInfo { major: MAJOR_VERSION, minor: MINOR_VERSION }
    }

    pub fn add_word(word: &str) -> Packet {
        Self::AddWord { word: word.to_string() }
    }
}

#[derive(Debug)]
pub struct Connection {
    pub stream: TcpStream,
    pub buf: Vec<u8>,
    pub buf_pos: usize
}

impl Connection {
    const CHUNK_SIZE: usize = 8192;

    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Connection {
            stream,
            buf: vec![],
            buf_pos: 0
        })
    }

    fn try_read(&mut self, bytes: usize) -> io::Result<usize> {
        if self.buf_pos + bytes >= self.buf.len() {
            self.buf.resize(self.buf.len() + Self::CHUNK_SIZE, 0);
        }

        match self.stream.read(&mut self.buf[self.buf_pos..self.buf_pos + bytes]) {
            Ok(x) => {
                self.buf_pos += x;
                Ok(x)
            },
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(e)
        }
    }

    pub fn poll_next_packet(&mut self) -> io::Result<Option<Packet>> {
        while self.buf_pos < 4 {
            if self.try_read(4 - self.buf_pos)? == 0 {
                return Ok(None);
            }
        }

        let packet_size = u32::from_be_bytes([self.buf[0], self.buf[1], self.buf[2], self.buf[3]]);
        trace!("Attempting to get packet of {} bytes", packet_size);
        let mut required = packet_size as usize + 4 - self.buf_pos;

        while required > 0 {
            trace!("Missing {} bytes", required);

            let n_read = self.try_read(required)?;

            if n_read == 0 {
                return Ok(None);
            }

            required -= n_read;
        }

        let res = Packet::parse(&self.buf[4..])?;

        self.buf_pos = 0;
        debug!("Received packet! {:?}", res);

        Ok(Some(res))
    }

    pub fn next_packet(&mut self, timeout: Duration) -> io::Result<Packet> {
        let start = Instant::now();

        while (Instant::now() - start) < timeout {
            match self.poll_next_packet()? {
                Some(x) => return Ok(x),
                _ => {}
            }
        }

        return Err(io::Error::new(io::ErrorKind::TimedOut, "Timed out on Connection::next_packet"));
    }

    pub fn send_packet(&mut self, packet: Packet) -> io::Result<()> {
        let mut data = vec![];
        packet.write(&mut data)?;
        self.stream.write_u32(data.len() as _)?;
        self.stream.write_all(&data)
    }
}

fn get_yes_no(msg: &str) -> io::Result<bool> {
    let stdin = io::stdin();

    loop {
        println!("{} (Y/n)", msg);

        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;
        buffer = buffer.trim().to_string();

        if buffer.len() != 1 {
            println!("Only answer with a single character");
        } else {
            let c = buffer.chars().nth(0).unwrap();

            if c == 'y' || c == 'Y' {
                return Ok(true);
            } else if c == 'n' || c == 'N' {
                return Ok(false);
            } else {
                println!("Answer with 'y','Y','n', or 'N'");
            }
        }
    }
}

fn pair_up() -> io::Result<TcpStream> {
    println!("Finding Connection");
    let stdin = io::stdin();

    let host = get_yes_no("Do you want to host?")?;

    let mut ip = String::new();
    println!("Enter IP:");
    stdin.read_line(&mut ip)?;
    let ip = ip.trim().to_string();

    let mut port = String::new();
    println!("Enter Port:");
    stdin.read_line(&mut port)?;
    let port = match port.trim().parse() {
        Ok(x) => x,
        Err(e) => return io::Result::Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))
    };


    if host {
        info!("Waiting for connection on {}:{}", ip, port);

        let listener = TcpListener::bind((ip, port))?;
        let (stream, addr) = listener.accept()?;

        info!("Got connection from {}", addr);

        Ok(stream)
    } else {
        info!("Attempting to connect to {}:{}", ip, port);

        let res = TcpStream::connect((ip, port))?;

        info!("Connected!");

        Ok(res)
    }
}

pub fn connect() -> io::Result<Connection> {
    let mut conn = Connection::new(pair_up()?)?;
    
    conn.send_packet(Packet::client_info())?;

    match conn.next_packet(Duration::from_secs(10))? {
        Packet::ClientInfo { major, minor } => {
            if major != MAJOR_VERSION && minor != MINOR_VERSION {
                return Err(io::Error::new(io::ErrorKind::Other, format!("Trying to connect to client with mismatched version. Your version = {}.{}. Their version = {}.{}.", MAJOR_VERSION, MINOR_VERSION, major, minor)));
            }
        },

        packet => {
            return Err(io::Error::new(io::ErrorKind::Other, format!("First packet should be a ClientInfo packet. Received {:?}", packet)));
        }
    }

    println!("Verified versions!");

    Ok(conn)
}

const DUMMY_IP: &str = "localhost";
const DUMMY_PORT: u16 = 5555;
const DUMMY_WORD_LIST: &str = DEFAULT_WORD_LIST;

fn run_dummy() {
    let stream = TcpStream::connect((DUMMY_IP, DUMMY_PORT)).unwrap();
    let mut conn = Connection::new(stream).unwrap();

    let words = fs::read_to_string(format!("res/words/{DUMMY_WORD_LIST}.txt")).unwrap();
    let words = words.lines();
    let words: Vec<_> = words.filter_map(|s| {
        let trimmed = s.trim();
        if trimmed.len() == 0 { return None; }

        let first_char: String = trimmed.chars().nth(0).unwrap().to_uppercase().collect();
        let rest: String = trimmed.chars().skip(1).flat_map(|c| c.to_lowercase()).collect();

        Some(format!("{}{}", first_char, rest))
    }).collect();

    let secs_range = 2..=3;

    let mut rng = rand::thread_rng();
    let mut next_word_send = Instant::now() + Duration::from_secs(rng.gen_range(secs_range.clone()));

    loop {
        if next_word_send <= Instant::now() {
            next_word_send = Instant::now() + Duration::from_secs(rng.gen_range(secs_range.clone()));
            let word = &words[rng.gen_range(0..words.len())];
            conn.send_packet(Packet::add_word(word)).unwrap();
        }

        let packet = conn.poll_next_packet().unwrap();

        if packet.is_none() {
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        let packet = packet.unwrap();

        match packet {
            Packet::ClientInfo {..} => {},
            Packet::AddWord { word } => debug!("Dummy received {word}"),
            Packet::ILost {  } => {},
            Packet::WaitingToRestart => {
                conn.send_packet(Packet::WaitingToRestart).unwrap();
            }
        }
    }
}

pub fn connect_to_dummy() -> io::Result<Connection> {

    let listener = TcpListener::bind((DUMMY_IP, DUMMY_PORT))?;

    thread::spawn(run_dummy);

    let (stream, _addr) = listener.accept()?;

    Ok(Connection::new(stream)?)
}