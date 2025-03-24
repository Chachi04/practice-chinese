extern crate rand;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use rand::seq::SliceRandom;
use serde_json::{Result, Value};
use std::fmt::{Display, Formatter};

use std::fs;
use std::io::{stdout, Write};
use std::process::Command;

#[derive(Debug, Clone)]
enum MenuState {
    HskLevel,
    Mission(String),
}

#[derive(Debug, Clone)]
enum MenuOption {
    Back,
    Exit,
    Level(String),
    Mission(String, String),
}

#[derive(Debug, Clone)]
enum PracticeMode {
    Pinyin,
    Hanzi,
}

impl Display for MenuState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuState::HskLevel => write!(f, "HSK Level Selection"),
            MenuState::Mission(mission) => write!(f, "Missions for HSK Level: {}", mission),
        }
    }
}

impl Display for MenuOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuOption::Back => write!(f, "<< Back"),
            MenuOption::Exit => write!(f, "Exit"),
            MenuOption::Level(level) => write!(f, "{}", level),
            MenuOption::Mission(mission, _) => write!(f, "{}", mission),
        }
    }
}

// impl MenuOption {
//     fn is_navigation(&self) -> bool {
//         matches!(self, MenuOption::Back | MenuOption::Exit)
//     }
// }

// fn validate_transition(current: &MenuState, next: &MenuOption) -> Result<()> {
//     match (current, next) {
//         (MenuState::Mission(_), MenuOption::Level(_)) => {
//             Err("Invalid transition from mission to level").into()
//         }
//         _ => Ok(()),
//     }
// }

fn main() {
    let mut history: Vec<MenuState> = vec![MenuState::HskLevel];
    loop {
        let file: String = fs::read_to_string("practice_sheet.json").expect("Unable to read file");
        let v: Value = serde_json::from_str(&file).expect("JSON was not  well-formatted");
        let v = v.as_object().unwrap();
        let levels = v.keys().collect::<Vec<&String>>();
        let current_state = history.last().unwrap().clone();
        let mut items = match &current_state {
            MenuState::HskLevel => levels
                .iter()
                .map(|l| MenuOption::Level(l.to_string()))
                .collect(), //&levels.to_vec(),
            MenuState::Mission(hsk_level) => {
                let mut missions = v[hsk_level]
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(|m| MenuOption::Mission(m.to_string(), hsk_level.clone()))
                    .collect::<Vec<MenuOption>>();
                missions.push(MenuOption::Back);
                missions
            }
        };

        items.push(MenuOption::Exit);

        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{}", current_state))
            .items(&items)
            .default(0)
            .interact()
            .unwrap();

        match items[selection] {
            MenuOption::Back => {
                history.pop();
                continue;
            }
            MenuOption::Exit => break,
            MenuOption::Level(ref lvl) => {
                history.push(MenuState::Mission(lvl.clone()));
            }
            MenuOption::Mission(ref name, ref level) => {
                let mut terms = v[level][name].as_array().unwrap().to_vec();
                if terms.is_empty() {
                    println!("No terms found for this mission");
                    break;
                }
                start_practice(&mut terms).unwrap();
                break;
            } // choice => match current_state {
              //     MenuState::HskLevel => history.push(MenuState::Mission(choice.to_string())),
              //     MenuState::Mission(hsk_level) => {
              //         let mut terms = v[&hsk_level][choice].as_array().unwrap().to_vec();
              //         if terms.is_empty() {
              //             println!("No terms found for this mission");
              //             break;
              //         }
              //         start_practice(&mut terms).unwrap();
              //         break;
              //     }
              // },
        }

        println!("END LOOP");
    }

    fn start_practice(terms: &mut Vec<serde_json::Value>) -> Result<()> {
        let mut mode = PracticeMode::Pinyin;
        let mut rng = rand::rng();
        terms.shuffle(&mut rng);

        let mut current_term_index = 0;
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        stdout.execute(Hide).unwrap();
        loop {
            let current_term = match mode {
                PracticeMode::Pinyin => terms[current_term_index]["pinyin"].as_str().unwrap(),
                PracticeMode::Hanzi => terms[current_term_index]["hanzi"].as_str().unwrap(),
            };
            stdout.execute(Clear(ClearType::All)).unwrap();
            stdout.execute(MoveTo(0, 0)).unwrap();
            print_centered(
                format!(
                    "Term ({}/{}) {}",
                    current_term_index + 1,
                    terms.len(),
                    current_term
                )
                .as_str(),
            )
            .unwrap();
            if let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Right | KeyCode::Char('j') => {
                        current_term_index = (current_term_index + 1) % terms.len()
                    }
                    KeyCode::Left | KeyCode::Char('k') => {
                        current_term_index = (current_term_index + terms.len() - 1) % terms.len()
                    }
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => mode = PracticeMode::Hanzi,
                    KeyCode::Char('p') => mode = PracticeMode::Pinyin,
                    KeyCode::Char('!') => {
                        let search_term = terms[current_term_index]["hanzi"].as_str().unwrap();
                        disable_raw_mode().unwrap();
                        stdout.execute(Show).unwrap();

                        let output = Command::new("hskindex")
                            .arg(search_term)
                            .output()
                            .expect("Failed to execute hskindex");
                        stdout.execute(MoveTo(0, 1)).unwrap();
                        print_centered(
                            format!(
                                "{}\nPress enter to continue...",
                                String::from_utf8_lossy(&output.stdout)
                            )
                            .as_str(),
                        )
                        .unwrap();
                        stdout.flush().unwrap();
                        event::read().unwrap();

                        enable_raw_mode().unwrap();
                        stdout.execute(Hide).unwrap();
                    }
                    _ => {}
                }
            }
        }
        stdout.execute(Show).unwrap();
        disable_raw_mode().unwrap();
        Ok(())
    }

    fn print_centered(text: &str) -> Result<()> {
        let (width, height) = terminal::size().unwrap();
        let lines: Vec<&str> = text.split('\n').collect();
        let line_count = lines.len() as u16;
        let start_y = (height - line_count) / 2;

        execute!(stdout(), Clear(ClearType::All)).unwrap();

        for (i, line) in lines.iter().enumerate() {
            let x = (width - line.len() as u16) / 2;
            let y = start_y + i as u16;

            execute!(stdout(), MoveTo(x, y), crossterm::style::Print(line)).unwrap();
        }
        Ok(())
    }
}
