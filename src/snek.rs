use crossterm::{
    QueueableCommand, cursor, 
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll, read}, 
    style::{self, Print, StyledContent, Stylize}, terminal::{self, Clear, ClearType},
};
use std::{
    io::{Result, Write, stdout, BufReader}, 
    time::{self, Duration, SystemTime}, 
    cmp,
    fs::File,
    thread,
};
use rand::Rng;
use rodio::{Decoder, OutputStream, mixer, source::Source};


const MUSIC_MAIN: &'static str = "worm-shaped_snake.mp3"; // todo un-hardcode
const MUSIC_MAIN_DUR: Duration = Duration::from_secs(36); // todo un-hardcode

const INITIAL_SNEK_LEN: u16 = 3;
const INPUT_WAIT_TIME: u64 = 600;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    BorderCell,
    SnekCell(u16),
    AppleCell,
    EmptyCell,
}

impl Cell {
    fn decrease_thouself(&mut self) {
        match self {
            Cell::SnekCell(v) if *v > 0 => {
                *self = Cell::SnekCell(*v - 1);
            },
            Cell::SnekCell(0) => *self = Cell::EmptyCell,
            _ => {},
        }
    }
}


#[derive(Debug, Clone)]
pub struct MapTemplate {
    map: Vec<Vec<Cell>>,
    headx: usize,
    heady: usize,
}



pub fn crossrender(info: &mut MapTemplate) -> Result<()> {
    play_music();
    let MapTemplate { map, headx, heady } = info;
    let (h, w) : (usize, usize) = (map.len(), map[0].len());
    let mut sneklen = INITIAL_SNEK_LEN;

    let mut prev_key: KeyCode = KeyCode::Up;
    let mut just_eaten: bool = false;
    let mut now: SystemTime;
    let mut wait: Duration;

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.queue(terminal::Clear(ClearType::All))?
          .queue(crossterm::cursor::Hide)?;


    'main_loop: loop {

        for row in 0..h {
            for cell in 0..w {
                stdout
                    .queue(cursor::MoveTo((row * 2) as u16, (cell * 2) as u16))?
                    .queue(style::PrintStyledContent(get_cell_styled(&map[row][cell], &sneklen)))?;

                if !just_eaten {
                    map[row][cell].decrease_thouself();
                }
            }
        }
        just_eaten = false;

        stdout.flush()?;
        
        loop {
            now = SystemTime::now();

            if poll(time::Duration::from_millis(INPUT_WAIT_TIME))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Release {
                        continue;
                    }
                    match key_event.code {
                        KeyCode::Char('q') => break 'main_loop,

                        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down 
                            => { handlekey(key_event.code, map, headx, heady, &mut sneklen, &mut just_eaten); prev_key = key_event.code; },

                        _ => handlekey(prev_key, map, headx, heady, &mut sneklen, &mut just_eaten),
                    }
                } 
                
                else {
                    handlekey(prev_key, map, headx, heady, &mut sneklen, &mut just_eaten);
                }
            } 
            
            else {
                handlekey(prev_key, map, headx, heady, &mut sneklen, &mut just_eaten);
            }
            
            wait = Duration::from_millis(INPUT_WAIT_TIME) - cmp::min(SystemTime::now().duration_since(now).expect("timer handling err"), Duration::from_millis(INPUT_WAIT_TIME));
            /* dbg!(wait.as_millis()); */
            thread::sleep(wait);
            break;
        }
    }


    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.flush()?;
    
    terminal::disable_raw_mode()?;
    Ok(())
}



fn get_cell_styled(c: &Cell, sneklen: &u16) -> StyledContent<char> {
    match c {
        Cell::BorderCell => '█'.black(),
        Cell::SnekCell(len) => {
            if *len == *sneklen { '█'.white() }
            else { '█'.green() }
        },
        Cell::AppleCell => '█'.red(),
        _ => ' '.stylize(),
    }
}


fn handlekey(key: KeyCode, map: &mut Vec<Vec<Cell>>, headx: &mut usize, heady: &mut usize, sneklen: &mut u16, just_eaten: &mut bool) {
    match key {
        KeyCode::Left => {
            if *headx > 1 {
                *headx -= 1;
            } else {
                *headx = map.len() - 2;
            }
        },

        KeyCode::Right => {
            if *headx < map.len() - 2 {
                *headx += 1;
            } else {
                *headx = 1;
            }
        }

        KeyCode::Up => {
            if *heady > 1 {
                *heady -= 1;
            } else {
                *heady = map[0].len() - 2;
            }
        }

        KeyCode::Down => {
            if *heady < map[0].len() - 2 {
                *heady += 1;
            } else {
                *heady = 1;
            }
        }

        _ => unreachable!(),
    }

    // assume _ arm is really unreachable()
    if map[*headx][*heady] == Cell::AppleCell {
        *just_eaten = true;
        *sneklen += 1;
        insert_apple(map);
    }

    map[*headx][*heady] = Cell::SnekCell(*sneklen);
}

fn insert_apple(map: &mut Vec<Vec<Cell>>) {
    let (h, w) : (usize, usize) = (map.len(), map[0].len());
    let (mut ax, mut ay) : (usize, usize) = (rand::thread_rng().gen_range(2..((h))), 
                                                 rand::thread_rng().gen_range(2..((w))));

    while matches!(map[ax][ay], Cell::BorderCell | Cell::SnekCell(_) | Cell::AppleCell) {
        (ax, ay) = (rand::thread_rng().gen_range(2..((h))), rand::thread_rng().gen_range(2..((w))));
    }

    map[ax][ay] = Cell::AppleCell;
}


pub fn genmap(h: u16, w: u16) -> MapTemplate {
    let h = h as usize;
    let w = w as usize;
    let mut map: Vec<Vec<Cell>> = vec![vec![Cell::EmptyCell; w + 2]; h + 2];

    for row in 0..(h + 2) {
        if row == 0 || row == h + 1 {
            for cell in 0..(w + 2) {
                map[row][cell] = Cell::BorderCell;
            }
        } else {
            map[row][0] = Cell::BorderCell;
            map[row][w + 1] = Cell::BorderCell;
        }
    }

    let (headx, heady) : (usize, usize) = (rand::thread_rng().gen_range(2..((h))), 
                                                 rand::thread_rng().gen_range(2..((w))));
    map[headx][heady] = Cell::SnekCell(INITIAL_SNEK_LEN);

    insert_apple(&mut map);

    let res = MapTemplate {
        map: map,
        headx: headx,
        heady: heady,
    };

    res
}

fn play_music() {
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
    let file = BufReader::new(File::open(MUSIC_MAIN).unwrap());
    thread::spawn(move || {
        let sink = rodio::play(&stream_handle.mixer(), file).unwrap();
        loop {
            thread::sleep(MUSIC_MAIN_DUR);
            sink.play();
        }
    });
}