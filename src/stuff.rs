use std::cmp::Ordering;
use std::cmp;

use std::io;
use std::{thread, time};

use rand::Rng;

pub fn _guessing_game() {
    println!("number guessing game! guess the number!");

    let secret_number = rand::thread_rng().gen_range(1..=100);
    
    loop {
        println!("please input your guess");
        let mut guess = String::new();
        
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: i32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };
        println!("you guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("too small!"),
            Ordering::Greater => println!("too big!"),
            Ordering::Equal => {
                println!("you win!");
                break;
            }
        }
    }
}


pub fn _boss_battle() {
    println!("mischievous pup blocks the way!");
    thread::sleep(time::Duration::from_secs(1));

    let mut boss_hp = 300;
    let mut boss_mercy = 0;
    let mut player_hp = 50;
    let mut boss_dmg_range = 10..=20;
    let mut player_dmg_range = 10..=30;
    let mut block_buff = 0;

    const SPARE_THREATHSHOLD: i32 = 81;

    let mut tried_mercy = false;
    let mut blocked = false;

    let mut turn = 0;

    loop {
        print!("\npuppy is at {} hp, you are {} health points till loss\n", boss_hp, player_hp);
        thread::sleep(time::Duration::from_secs(1));
        
        match boss_mercy {
            5..=30 => {
                println!("your mortal enemy seems to start feeling a little pinch of grace to you");
                thread::sleep(time::Duration::from_secs(1));
            },
            31..=80 => {
                println!("seemed as if dog confronting you starts to like you");
                thread::sleep(time::Duration::from_secs(1));
            },
            SPARE_THREATHSHOLD..=500 => {
                println!("it SPARES YOU");
                thread::sleep(time::Duration::from_secs(1));
            },
            _ => (),
        }

        print!("\nchoose to:\n- (f)ight\n- (b)lock\n- (m)ercy\n");
        if boss_mercy >= SPARE_THREATHSHOLD {
            print!("\n- (s)pare\n");
        }

        let mut boss_dmg = rand::thread_rng().gen_range(boss_dmg_range.clone());
        let player_dmg = rand::thread_rng().gen_range(player_dmg_range.clone());
        let player_heal = rand::thread_rng().gen_range(10..=30);

        loop {
            let mut response = String::new();

            io::stdin()
                .read_line(&mut response)
                .expect("failed");
            response = response.trim().to_string();

            match response.as_str() {
                "f" | "fight" => {
                    print!("\nyou deal exactly {} damage to the dog\n", player_dmg);
                    boss_hp = cmp::max(0, boss_hp - player_dmg);
                    thread::sleep(time::Duration::from_secs(1));

                    if boss_mercy <= 0 {
                        println!("it felt like your power grew...");

                        let (start, end) = (*player_dmg_range.start(), *player_dmg_range.end());
                        player_dmg_range = (start + 3)..=(end + 6);
                    } else {
                        println!("any kind of mercy vanished from inside of it");
                        boss_mercy = 0;
                    }

                    break;
                },

                "b" | "block" => {
                    blocked = true;
                    println!("you blocked! enemy's damage decreased!");
                    block_buff = 10;
                    thread::sleep(time::Duration::from_secs(1));
                    
                    println!("pup feels sorry for annoying you");
                    boss_mercy += 5;

                    if player_hp < 50 {
                        println!("you regained some of your powers");
                        player_hp = cmp::min(50, player_hp + player_heal);
                    }
                    
                    break;
                },

                "m" | "mercy" => {
                    tried_mercy = true;
                    match boss_mercy {
                        0..=30 => println!("you tried to pet it. it seemed to not like this"),
                        31..=60 => println!("you tried to pet it. little version of dog even closed its eyes for a moment"),
                        61..100 => println!("you tried to pet it. pup seems excited!"),
                        _ => println!("how did we get there"),
                    }
                    thread::sleep(time::Duration::from_secs(1));

                    boss_mercy += 40;
                    boss_dmg_range = cmp::max(5, boss_dmg_range.start() - 5)..=cmp::max(10, boss_dmg_range.end() - 5);

                    break;
                },

                "s" | "spare" => {
                    if boss_mercy >= SPARE_THREATHSHOLD {
                        println!("little puppy does accept your sparing request! while jumping around and waving it tail!");
                        thread::sleep(time::Duration::from_secs(1));
                        println!("you won by sparing!");
                        thread::sleep(time::Duration::from_secs(3));
                        break;
                    } else {
                        println!("but it refused.");
                    }
                    thread::sleep(time::Duration::from_secs(2));

                    break;
                },

                "exit" => return (),

                _ => continue,
            }
        }

        if boss_mercy >= SPARE_THREATHSHOLD {
            break;
        }

        thread::sleep(time::Duration::from_secs(1));

        boss_dmg -= block_buff;

        match turn % 3 {
            0 => print!("\npup swings its tail, dealing blasting {} damage!\n", boss_dmg),
            1 => print!("\nin a short swirly spin dog confronting you spined so fast, it created hurricane which lowered your health points by {}!\n", boss_dmg),
            2 => print!("\nlittle chomp out of your leg hurt a bit, dealing {} damage!\n", boss_dmg),
            _ => (),
        }

        thread::sleep(time::Duration::from_secs(2));

        player_hp = cmp::max(0, player_hp - boss_dmg);

        if player_hp <= 0 {
            println!("you lost. to a puppy. what a humiliating defeat");
            if !blocked {
                println!("you didn't even block");
            }
            thread::sleep(time::Duration::from_secs(5));
            break;
        }

        if boss_hp <= 0 {
            println!("you killed a puppy. what a monster if you, have you even trying to be MERCYful to it?");
            if tried_mercy {
                thread::sleep(time::Duration::from_secs(1));
                println!("actually. you tried.");
            }
            thread::sleep(time::Duration::from_secs(5));
            break;
        }

        block_buff = 0;


        println!("-------------------------------------------------------");
        turn += 1;
    }
}


pub fn _fib_nomemo(n: i32) -> i32 {
    match n {
        0 => 0,
        1 | 2 => 1,
        _ => {
            _fib_nomemo(n - 1) + _fib_nomemo(n - 2)
        },
    }
}