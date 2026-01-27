use crossterm::{
    QueueableCommand, cursor, event::{self, Event, KeyCode}, style::{self, Print, StyledContent, Stylize}, terminal::{self, Clear, ClearType}
};
use std::io::{Result, Write, stdout};
use rand::Rng;

const INITIAL_SNEK_LEN: u16 = 3;

#[derive(Debug, Clone, Copy)]
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

pub struct MapTemplate {
    map: Vec<Vec<Cell>>,
    headx: u16,
    heady: u16,
}


pub fn crossrender(info: &mut MapTemplate) -> Result<()> {
    let MapTemplate { map, headx, heady } = info;
    let (h, w) : (u16, u16) = (map.len() as u16, map[0].len() as u16);
    let mut sneklen = INITIAL_SNEK_LEN;

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();

    loop {
        stdout.queue(terminal::Clear(ClearType::All))?;

        for row in 0..h {
            for cell in 0..w {
                stdout
                    .queue(cursor::MoveTo(row * 2, cell * 2))?
                    .queue(style::PrintStyledContent(get_cell_styled(&map[row as usize][cell as usize], &sneklen)))?;
                map[row as usize][cell as usize].decrease_thouself();
            }
        }

        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('q') => break,

                KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down 
                    => handlekey(key_event.code, map, headx, heady, &mut sneklen),

                _ => continue,
            }
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


fn handlekey(key: KeyCode, map: &mut Vec<Vec<Cell>>, headx: &mut u16, heady: &mut u16, sneklen: &mut u16) {
    match key { // todo add apple handling
        KeyCode::Left => {
            if *headx > 1 {
                *headx -= 1;
            } else {
                *headx = (map.len() as u16) - 2;
            }
            map[*headx as usize][*heady as usize] = Cell::SnekCell(*sneklen);
        },
        KeyCode::Right => {
            if *headx < (map.len() as u16) - 2 {
                *headx += 1;
            } else {
                *headx = 1;
            }
            map[*headx as usize][*heady as usize] = Cell::SnekCell(*sneklen);
        }
        KeyCode::Up => {
            if *heady > 1 {
                *heady -= 1;
            } else {
                *heady = (map[0].len() as u16) - 2;
            }
            map[*headx as usize][*heady as usize] = Cell::SnekCell(*sneklen);
        }
        KeyCode::Down => {
            if *heady < (map[0].len() as u16) - 2 {
                *heady += 1;
            } else {
                *heady = 1;
            }
            map[*headx as usize][*heady as usize] = Cell::SnekCell(*sneklen);
        }
        _ => unreachable!(),
    }
}


pub fn genmap(h: u16, w: u16) -> MapTemplate {
    let mut map: Vec<Vec<Cell>> = vec![vec![Cell::EmptyCell; (w + 2) as usize]; (h + 2) as usize];

    for row in 0..(h + 2) {
        if row == 0 || row == h + 1 {
            for cell in 0..(w + 2) {
                map[row as usize][cell as usize] = Cell::BorderCell;
            }
        } else {
            map[row as usize][0] = Cell::BorderCell;
            map[row as usize][(w + 1) as usize] = Cell::BorderCell;
        }
    }

    let (headx, heady) : (u16, u16) = (rand::thread_rng().gen_range(2..((h))), 
                                                 rand::thread_rng().gen_range(2..((w))));
    map[headx as usize][heady as usize] = Cell::SnekCell(INITIAL_SNEK_LEN);

    let res = MapTemplate {
        map: map,
        headx: headx,
        heady: heady,
    };

    res
}