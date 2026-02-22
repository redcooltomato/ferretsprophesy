use crossterm::{
    QueueableCommand, cursor, 
    event::{self, Event, KeyCode, KeyEventKind, poll}, 
    style::{self, Print, StyledContent, Stylize}, 
    terminal::{self, ClearType},
};
use std::{
    cmp, fs::File, thread,
    io::{BufReader, Result, Stdout, Write, stdout}, 
    sync::{Arc, atomic::{self, AtomicBool}},
    time::{Duration, SystemTime}
};
use rand::Rng;
use rodio::{OutputStream, Sink};
use mp3_duration;


const MUSIC_MAIN: &'static str = "src/ass/worm-shaped_snake.mp3"; // dont even try unhardcoding -- include_bytes veteran
                                                          // this veteran had sound turned off while doing that -- listening veteran

const INITIAL_SNEK_LEN: u16 = 3;
const INPUT_WAIT_TIME: u64 = 500;
const MUSIC_FLAG_CHECK_TIME: u64 = 100;


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

fn get_antag_key(key: KeyCode) -> KeyCode {
    match key {
        KeyCode::Up => KeyCode::Down,
        KeyCode::Down => KeyCode::Up,
        KeyCode::Left => KeyCode::Right,
        KeyCode::Right => KeyCode::Left,
        _ => key,
    }
}

pub fn crossrender(info: &mut MapTemplate) -> Result<()> {
    let MapTemplate { map, headx, heady } = info;
    let (h, w) : (usize, usize) = (map.len(), map[0].len());
    let mut sneklen = INITIAL_SNEK_LEN;

    let mut prev_key: KeyCode = KeyCode::Up;
    let (mut now, mut wait) : (SystemTime, Duration);

    let (music_thread, music_flag)
      : (thread::JoinHandle<()>, Arc<AtomicBool>) = play_music(MUSIC_MAIN.to_string());

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.queue(terminal::Clear(ClearType::All))?
          .queue(crossterm::cursor::Hide)?
          .queue(crossterm::cursor::DisableBlinking)?;


    'main_loop: loop {

        draw(h, w, map, &mut stdout, &mut sneklen)?;

        loop {
            now = SystemTime::now();

            if poll(Duration::from_millis(INPUT_WAIT_TIME))? {
                /* println!("{}", inp_wait_dur.as_millis()); */
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Release {
                        continue;
                    }

                    if key_event.code == prev_key || key_event.code == get_antag_key(prev_key) {
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


    music_flag.store(true, atomic::Ordering::Relaxed);
    music_thread.join().expect("music thread failed to join");
    lose(&mut stdout, h, w)?;

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

fn lose(stdout: &mut Stdout, h: usize, w: usize) -> Result<()> { // i will assume that map is large enough to fit this
    let (h, w) : (u16, u16) = (h as u16, w as u16);
    stdout.queue(terminal::Clear(ClearType::All))?;
    stdout.flush()?;
    

    stdout.queue(cursor::MoveTo(w / 2 - 8, 0))?;

    stdout.queue(Print("GAME OVER"))?;


    stdout.flush()?;
    thread::sleep(Duration::from_secs(5));

    Ok(())
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

fn play_music(audio_path: String) -> (thread::JoinHandle<()>, Arc<AtomicBool>) { // yeah its owned
    let stream_handle: OutputStream = rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");

    let music_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let music_flag_clone = music_flag.clone();

    let music_thread: thread::JoinHandle<()>= thread::spawn(move || {
        let (music_dur, mut elapsed, music_check_interval) : (Duration, Duration, Duration) = 
        (
            mp3_duration::from_path(&audio_path).expect("duration calc err"), 
            Duration::from_secs(0), 
            Duration::from_millis(MUSIC_FLAG_CHECK_TIME)
        );
        
        let (mut file, mut sink): (BufReader<File>, Sink);

        'music_loop: loop {
            file = BufReader::new(File::open(&audio_path).unwrap());
            sink = rodio::play(&stream_handle.mixer(), file).unwrap();

            elapsed = Duration::from_secs(0);
            
            while elapsed < music_dur {
                thread::sleep(music_check_interval);
                elapsed += music_check_interval;
                if music_flag_clone.load(atomic::Ordering::Relaxed) {
                    break 'music_loop;
                }
            }
        }
    });
    return (music_thread, music_flag);
}