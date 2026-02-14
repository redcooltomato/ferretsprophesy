use crossterm::{
    QueueableCommand, cursor, 
    event::{self, Event, KeyCode, KeyEventKind, poll}, 
    style::{self, StyledContent, Stylize}, terminal::{self, ClearType},
};
use std::{
    cmp, fs::File, io::{BufReader, Result, Stdout, Write, stdout}, thread, time::{self, Duration, SystemTime}
};
use rand::Rng;
use rodio::{OutputStream, Sink};
use mp3_duration;


const MUSIC_MAIN: &'static str = "src/ass/worm-shaped_snake.mp3"; // dont even try unhardcoding -- include_bytes veteran
                                                          // this veteran had sound turned off while doing that -- listening veteran

const INITIAL_SNEK_LEN: u16 = 3;
const INPUT_WAIT_TIME: u64 = 500;


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
    let (mut now, mut inp_wait_dur) : (SystemTime, Duration);
    let mut wait: Duration;

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.queue(terminal::Clear(ClearType::All))?
          .queue(crossterm::cursor::Hide)?
          .queue(crossterm::cursor::DisableBlinking)?;


    'main_loop: loop {

        draw(h, w, map, &mut stdout, &mut sneklen)?;

        inp_wait_dur = Duration::from_millis(INPUT_WAIT_TIME);

        loop {
            now = SystemTime::now();

            if poll(inp_wait_dur)? {
                /* println!("{}", inp_wait_dur.as_millis()); */
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Release {
                        continue;
                    }

                    if key_event.code == prev_key {
                        inp_wait_dur = Duration::from_millis(INPUT_WAIT_TIME) - SystemTime::now().duration_since(now).expect("clockshit in duplicate key check");
                        continue;
                    }

                    match key_event.code {
                        KeyCode::Char('q') => break 'main_loop,

                        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down
                            => { prev_key = key_event.code; },

                        _ => (),
                    }
                }
            }

            if handlekey(prev_key, map, headx, heady, &mut sneklen) == false { 
                break 'main_loop;
            }
            
            wait = Duration::from_millis(INPUT_WAIT_TIME) - cmp::min(SystemTime::now().duration_since(now).expect("timer handling err"), Duration::from_millis(INPUT_WAIT_TIME));
            /* dbg!(wait.as_millis()); */
            thread::sleep(wait);
            break;
        }
    }

    lose(&mut stdout, h, w);

    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.flush()?;
    
    terminal::disable_raw_mode()?;
    Ok(())
}


fn draw(h: usize, w: usize, map: &mut Vec<Vec<Cell>>, stdout: &mut Stdout, sneklen: &mut u16) -> Result<()> {
    for row in 0..h {
        for cell in 0..w {
            stdout
                .queue(cursor::MoveTo((row) as u16, (cell) as u16))?
                /* .queue(cursor::MoveTo((row * 2) as u16, (cell * 2) as u16))? */
                .queue(style::PrintStyledContent(get_cell_styled(&map[row][cell], &sneklen)))?;

            map[row][cell].decrease_thouself();
        }
    }

    stdout.flush()?;
    Ok(())
}

fn lose(stdout: &mut Stdout, h: usize, w: usize) { // i will assume that map is large enough to fit this
    ()
}

fn get_cell_styled(c: &Cell, sneklen: &u16) -> StyledContent<char> {
    match c {
        Cell::BorderCell => '█'.dark_blue(),
        Cell::SnekCell(len) => {
            if *len == *sneklen { '█'.white() }
            else { '█'.green() }
        },
        Cell::AppleCell => '█'.red(),
        _ => ' '.stylize(),
    }
}


fn handlekey(key: KeyCode, map: &mut Vec<Vec<Cell>>, headx: &mut usize, heady: &mut usize, sneklen: &mut u16) -> bool {
    match key {
        KeyCode::Left => {
            *headx -= 1;
        },

        KeyCode::Right => {
            *headx += 1;
        }

        KeyCode::Up => {
            *heady -= 1;
        }

        KeyCode::Down => {
            *heady += 1;
        }

        _ => unreachable!(),
    }

    // assume _ arm is really unreachable()
    let res: bool = match map[*headx][*heady] {
        Cell::AppleCell => {
            *sneklen += 1;
            insert_apple(map);
            true
        },
        Cell::BorderCell => false,
        Cell::SnekCell(_l) => false,
        _ => true,
    };

    map[*headx][*heady] = Cell::SnekCell(*sneklen);

    res
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
    let (w, h) = (h as usize, w as usize);
    // yeah i messed up the order somewhere so now they are reversed

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
    let stream_handle: OutputStream = rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
    thread::spawn(move || {
        let music_dur: Duration = mp3_duration::from_path(MUSIC_MAIN).expect("duration calc err");
        let mut file: BufReader<File>;
        let mut _sink: Sink;
        loop {
            file = BufReader::new(File::open(MUSIC_MAIN).unwrap());
            _sink = rodio::play(&stream_handle.mixer(), file).unwrap();
            thread::sleep(music_dur);
        }
    });
}