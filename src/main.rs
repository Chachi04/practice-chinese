use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::{stdout, Write};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Term {
    hanzi: String,
    pinyin: String,
    #[serde(default)]
    saved: bool,
}

impl Term {
    fn toggle_save(&mut self) {
        self.saved = !self.saved;
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Mission {
    #[serde(rename = "Mission")]
    id: i8,
    #[serde(rename = "Terms")]
    terms: Vec<Term>,
}

#[derive(Serialize, Deserialize, Debug)]
struct HskLevel {
    #[serde(rename = "HSK Level")]
    level: i8,
    #[serde(rename = "Missions")]
    missions: Vec<Mission>,
}

impl Display for HskLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HSK Level {}", self.level)
    }
}

impl Display for Mission {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mission {}", self.id)
    }
}

#[derive(Debug, Clone)]
enum MenuState {
    HskLevel,
    Mission(usize),
}

#[derive(Debug, Clone)]
enum MenuOption {
    Back,
    Exit,
    HskLevel(String),
    Mission(String, String),
    SavedTerms,
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
            MenuOption::HskLevel(level) => write!(f, "HSK Level {}", level),
            MenuOption::Mission(mission, _) => write!(f, "Mission {}", mission),
            MenuOption::SavedTerms => write!(f, "Saved Terms"),
        }
    }
}

const PRACTICE_SHEET_PATH: &str =
    "/home/chachi/code/side-projects/practice-chinese/practice_sheet.json";

fn main() -> Result<(), Box<dyn Error>> {
    let mut history = vec![MenuState::HskLevel];

    let mut hsk_levels: Vec<HskLevel> = read_levels_from_file(PRACTICE_SHEET_PATH)?;

    loop {
        let current_state = history.last().unwrap().to_owned();
        let mut items: Vec<MenuOption> = match &current_state {
            MenuState::HskLevel => {
                let mut levels: Vec<MenuOption> = hsk_levels
                    .iter()
                    .map(|hsk| MenuOption::HskLevel(hsk.level.to_string()))
                    .collect();
                levels.push(MenuOption::SavedTerms);
                levels
            }
            MenuState::Mission(level) => {
                let mut missions: Vec<MenuOption> = hsk_levels[*level - 1]
                    .missions
                    .iter()
                    .map(|mission| MenuOption::Mission(mission.id.to_string(), level.to_string()))
                    .collect();
                missions.push(MenuOption::Back);
                missions
            }
        };
        items.push(MenuOption::Exit);

        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{}", current_state))
            .items(&items)
            .default(0)
            .interact()?;

        match items[selection] {
            MenuOption::Back => {
                history.pop();
                continue;
            }
            MenuOption::Exit => return Ok(()),
            MenuOption::HskLevel(ref level) => {
                println!("{}", level);
                history.push(MenuState::Mission(level.parse()?));
            }
            MenuOption::Mission(ref mission, ref level) => {
                let lvl_idx = level.parse::<usize>()? - 1;
                let mission_idx = mission.parse::<usize>()? - 1;
                let mut terms: Vec<&mut Term> = hsk_levels[lvl_idx].missions[mission_idx]
                    .terms
                    .iter_mut()
                    .collect();
                start_practice(
                    &(lvl_idx + 1).to_string(),
                    &(mission_idx + 1).to_string(),
                    &mut terms,
                )?;
                break;
            }
            MenuOption::SavedTerms => {
                let mut terms: Vec<&mut Term> = get_saved_terms(&mut hsk_levels);
                if terms.is_empty() {
                    println!("No saved terms");
                    break;
                }
                start_practice("Saved Terms", "", &mut terms)?;
                break;
            }
        }
    }
    save_to_file(&hsk_levels, PRACTICE_SHEET_PATH)?;
    Ok(())
}
fn start_practice(
    lvl_id: &str,
    mission_id: &str,
    terms: &mut Vec<&mut Term>,
) -> Result<(), Box<dyn Error>> {
    let mut current_mode = PracticeMode::Pinyin;
    let mut rng = rand::rng();
    terms.shuffle(&mut rng);

    let mut current_term_index = 0;
    enable_raw_mode()?;
    execute!(stdout(), Hide)?;
    loop {
        let saved = match terms[current_term_index].saved {
            true => " [Saved]",
            false => "",
        };
        let term = match current_mode {
            PracticeMode::Pinyin => terms[current_term_index].pinyin.as_str(),
            PracticeMode::Hanzi => terms[current_term_index].hanzi.as_str(),
        };
        print_card(
            lvl_id,
            mission_id,
            term,
            &current_term_index,
            &terms.len(),
            saved,
        )?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Right | KeyCode::Char('j') => {
                    current_term_index = (current_term_index + 1) % terms.len()
                }
                KeyCode::Left | KeyCode::Char('k') => {
                    current_term_index = (current_term_index + terms.len() - 1) % terms.len()
                }
                KeyCode::Char('q') => break,
                KeyCode::Char('h') => current_mode = PracticeMode::Hanzi,
                KeyCode::Char('p') => current_mode = PracticeMode::Pinyin,
                KeyCode::Char('!') => {
                    let search_term = terms[current_term_index].hanzi.as_str();
                    // disable_raw_mode()?;
                    execute!(stdout(), Show)?;

                    let output = Command::new("hskindex").arg(search_term).output()?;
                    execute!(stdout(), Clear(ClearType::All))?;
                    execute!(stdout(), MoveTo(0, 1))?;
                    print_centered(
                        format!(
                            "{}\nPress enter to continue...",
                            String::from_utf8_lossy(&output.stdout)
                        )
                        .as_str(),
                    )?;
                    stdout().flush()?;

                    event::read()?;
                    // enable_raw_mode()?;
                    execute!(stdout(), Hide)?;
                }
                KeyCode::Char('s') => terms[current_term_index].toggle_save(),
                _ => {}
            }
        }
    }
    stdout().execute(Show)?;
    disable_raw_mode()?;
    Ok(())
}

fn read_levels_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<HskLevel>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let u = serde_json::from_reader(reader)?;

    Ok(u)
}

fn save_to_file(hsks: &Vec<HskLevel>, path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, hsks)?;
    Ok(())
}
fn print_card(
    lvl_idx: &str,
    mission_idx: &str,
    term: &str,
    current_term_index: &usize,
    terms_count: &usize,
    term_saved: &str,
) -> Result<(), Box<dyn Error>> {
    let set = format!("HSK Level {lvl_idx} - Mission {mission_idx}",);
    execute!(stdout(), Clear(ClearType::All))?;
    execute!(
        stdout(),
        MoveTo(0, 0),
        crossterm::style::Print(set.as_str())
    )
    .unwrap();
    print_centered(
        format!(
            "Term ({}/{}) {} {}",
            current_term_index + 1,
            terms_count,
            term,
            term_saved
        )
        .as_str(),
    )?;
    Ok(())
}

fn print_centered(text: &str) -> Result<(), Box<dyn Error>> {
    let (width, height) = terminal::size()?;
    let lines: Vec<&str> = text.split('\n').collect();
    let line_count = lines.len() as u16;
    let start_y = (height - line_count) / 2;

    for (i, line) in lines.iter().enumerate() {
        let x = (width - line.len() as u16) / 2;
        let y = start_y + i as u16;

        execute!(stdout(), MoveTo(x, y), crossterm::style::Print(line))?;
    }
    Ok(())
}

fn get_saved_terms(hsk_levels: &mut Vec<HskLevel>) -> Vec<&mut Term> {
    hsk_levels
        .iter_mut()
        .flat_map(|hsk| hsk.missions.iter_mut())
        .flat_map(|mission| mission.terms.iter_mut())
        .filter(|term| term.saved)
        .collect()
}
