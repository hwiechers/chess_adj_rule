use std::io::Write;
use std::process::exit;

use clap::ArgMatches;

use super::{GameData, read_games};

#[derive(Debug, Clone)]
struct GameStats {
    length: u32,
    time: u32,
    score10: u32,
}

fn format_time(milliseconds: u32) -> String {
    let mut value = milliseconds;

    let ms = value % 1000;
    value /= 1000;

    let s = value % 60;
    value /= 60;

    let m = value % 60;
    value /= 60;

    let h = value;

    format!("{}:{:02}:{:02}.{:03}", h, m, s, ms)
}

// An engine resigns if his eval is worse than `-eval`
// for `count` of his moves with this rule
pub struct ResignRule {
    eval: i32,
    count: u32,
}

impl ResignRule {
    fn new(eval: i32, count: u32) -> Result<ResignRule, &'static str> {
        if eval <= 0 {
            return Err("eval is out of range.");
        }

        if count == 0 {
            return Err("count is out of range.");
        }

        Ok(ResignRule {
            eval: eval,
            count: count,
        })
    }
}

// The game is a draw the score is within [-eval, eval]
// for `count` of his moves with this rule. The rule
// may only be applied on or after `from_move`.
pub struct DrawRule {
    from_move: u32,
    eval: i32,
    count: u32,
}

impl DrawRule {
    fn new(from_move: u32, eval: i32, count: u32) -> Result<DrawRule, &'static str> {
        if from_move == 0 {
            return Err("from_move is out of range.");
        }

        if eval < 0 {
            return Err("eval is out of range.");
        }

        if count == 0 {
            return Err("count is out of range.");
        }

        Ok(DrawRule {
            from_move: from_move,
            eval: eval,
            count: count,
        })
    }
}

enum ResignRuleParsingError {
    BadFormat,
    NonPositiveEval,
    NonPositiveCount
}

fn parse_resign_rule(input: &str) -> Result<ResignRule, ResignRuleParsingError> {
    if input == "none" {
        //Return a rule that will never be applied
        return Ok(ResignRule::new(10000, 10000).unwrap());
    }

    let args: Vec<&str> = input.split('/').collect();
    let eval = match args[0].parse::<i32>() {
        Ok(value) => value,
        Err(_) => { return Err(ResignRuleParsingError::BadFormat); }
    };

    if eval <= 0 {
        return Err(ResignRuleParsingError::NonPositiveEval);
    }

    let count = match args[1].parse::<u32>() {
        Ok(value) => value,
        Err(_) => { return Err(ResignRuleParsingError::BadFormat); }
    };

    if count == 0 {
        return Err(ResignRuleParsingError::NonPositiveCount);
    }

    Ok(ResignRule::new(eval, count).unwrap())
}

enum DrawRuleParsingError {
    BadFormat,
    NonPositiveFromMove,
    NegativeEval,
    NonPositiveCount
}

fn parse_draw_rule(input: &str) -> Result<DrawRule, DrawRuleParsingError> {
    if input == "none" {
        //Return a rule that will never be applied
        return Ok(DrawRule::new(10000, 0, 10000).unwrap());
    }

    let args1: Vec<&str> = input.split(':').collect();
    let from_move = match args1[0].parse::<u32>() {
        Ok(value) => value,
        Err(_) => { return Err(DrawRuleParsingError::BadFormat); }
    };

    if from_move == 0 {
        return Err(DrawRuleParsingError::NonPositiveFromMove);
    }

    let args2: Vec<&str> = args1[1].split('/').collect();
    let eval = match args2[0].parse::<i32>() {
        Ok(value) => value,
        Err(_) => { return Err(DrawRuleParsingError::BadFormat); }
    };

    if eval <= 0 {
        return Err(DrawRuleParsingError::NegativeEval);
    }

    let count = match args2[1].parse::<u32>() {
        Ok(value) => value,
        Err(_) => { return Err(DrawRuleParsingError::BadFormat); }
    };

    if count == 0 {
        return Err(DrawRuleParsingError::NonPositiveCount);
    }

    Ok(DrawRule::new(from_move, eval, count).unwrap())
}

pub fn main(matches: &ArgMatches) {
    let resign_rule =
        match parse_resign_rule(matches.value_of("resign_rule").unwrap()) {
            Ok(rule) => rule,
            Err(ResignRuleParsingError::BadFormat) => {
                println_stderr!("error: Resign rule has bad format");
                exit(1);
            },
            Err(ResignRuleParsingError::NonPositiveEval) => {
                println_stderr!("error: Resign rule evaluation must be positive");
                exit(1);
            },
            Err(ResignRuleParsingError::NonPositiveCount) => {
                println_stderr!("error: Resign rule count must be positive");
                exit(1);
            },
        };

    let draw_rule =
        match parse_draw_rule(matches.value_of("draw_rule").unwrap()) {
            Ok(rule) => rule,
            Err(DrawRuleParsingError::BadFormat) => {
                println_stderr!("error: Draw rule has bad format");
                exit(1);
            },
            Err(DrawRuleParsingError::NonPositiveFromMove) => {
                println_stderr!("error: Draw rule move from must be positive");
                exit(1);
            },
            Err(DrawRuleParsingError::NegativeEval) => {
                println_stderr!("error: Draw rule evaluation must be positive");
                exit(1);
            },
            Err(DrawRuleParsingError::NonPositiveCount) => {
                println_stderr!("error: Draw rule count must be positive");
                exit(1);
            },
        };

    let game_data = read_games(&matches);

    test_rule(
        &game_data,
        &resign_rule,
        &draw_rule,
        matches.is_present("verbose"));
}

fn test_rule(games: &Vec<GameData>,
                 resign_rule: &ResignRule,
                 draw_rule: &DrawRule,
                 verbose: bool) {

    let mut actual_time = 0;
    let mut adjudicated_time = 0;

    let mut resign_num = 0;
    let mut resign_num_wrong = 0;
    let mut resign_time_saved = 0;
    let mut resign_squared_error10 = 0;

    let mut draw_num = 0;
    let mut draw_num_wrong = 0;
    let mut draw_time_saved = 0;
    let mut draw_squared_error10 = 0;


    if verbose {
        println!("game, actual_length, actual_time, actual_score, \
                  rule_applied, adjudicated_length, adjudicated_time, adjudicated_score");
    }

    for (index, game) in games.iter().enumerate() {

        let outcome = adjudicate_game(game, resign_rule, draw_rule);

        match outcome.rule_applied {
            Some(RuleType::Resign) => {
                resign_num += 1;
                resign_num_wrong += (!outcome.correctly_adjudicated()) as u32;
                resign_time_saved += outcome.time_saved();
                resign_squared_error10 += outcome.squared_error10();
            }
            Some(RuleType::Draw) => {
                draw_num += 1;
                draw_num_wrong += (!outcome.correctly_adjudicated()) as u32;
                draw_time_saved += outcome.time_saved();
                draw_squared_error10 += outcome.squared_error10();
            }
            None => { }
        }

        actual_time += outcome.actual.time;
        adjudicated_time += outcome.adjudicated.time;

        if verbose {
            println!("{}, {}, {}, {}, {}, {}, {}, {}",
                     index + 1,
                     outcome.actual.length,
                     outcome.actual.time,
                     outcome.actual.score10 as f32 / 10f32,
                     match outcome.rule_applied {
                         Some(RuleType::Resign) => "R",
                         Some(RuleType::Draw) => "D",
                         None => "-",
                     },
                     outcome.adjudicated.length,
                     outcome.adjudicated.time,
                     outcome.adjudicated.score10 as f32 / 10f32);
        }
    }

    if verbose {
        println!("");
    }

    let adjudicated_num = resign_num + draw_num;
    let adjudciated_num_wrong = resign_num_wrong + draw_num_wrong;

    println!("Games: {}", games.len());
    println!("Adjudicated: {} ({} wrong)", adjudicated_num, adjudciated_num_wrong);
    println!("  Resign: {} ({} wrong)", resign_num, resign_num_wrong);
    println!("  Draw: {} ({} wrong)", draw_num, draw_num_wrong);
    println!("");

    let time_saved = resign_time_saved + draw_time_saved;
    let time_saved_perc = time_saved as f64 / actual_time as f64 * 100f64;

    let resign_time_saved_perc = resign_time_saved as f64 / actual_time as f64 * 100f64;
    let draw_time_saved_perc = draw_time_saved as f64 / actual_time as f64 * 100f64;

    println!("Total Time: {}", format_time(actual_time));
    println!("After Adjudication: {}", format_time(adjudicated_time));
    println!("Time saved: {} ({:.2}%)", format_time(time_saved as u32), time_saved_perc);
    println!("  Resign: {} ({:.2}%)", format_time(resign_time_saved), resign_time_saved_perc);
    println!("  Draw: {} ({:.2}%)", format_time(draw_time_saved), draw_time_saved_perc);
    println!("Note: 'Time saved' excludes incorrectly adjudicated games");
    println!("");

    let mse =
        (resign_squared_error10 as f64 + draw_squared_error10 as f64)
        / 100f64
        / (games.len() as f64);

    let resign_mse = resign_squared_error10 as f64 / 100f64 / (games.len() as f64);
    let draw_mse = draw_squared_error10 as f64 / 100f64 / (games.len() as f64);

    println!("Mean Squared Error: {:.6}", mse);
    println!("  Resign: {:.6}", resign_mse);
    println!("  Draw: {:.6}", draw_mse);
    println!("Root MSE: {:.3}", mse.powf(0.5));
}

fn adjudicate_game(
    game: &GameData,
    resign_rule: &ResignRule,
    draw_rule: &DrawRule) -> AdjudicationOutcome {

    let mut resign_counts: [u32; 2] = [0, 0];
    let mut draw_count = 0;

    let score10 = game.score10;

    let mut total_time = 0;
    let mut rule_applied: Option<RuleType> = None;
    let mut adjudicated_outcome: Option<GameStats> = None;

    for (ply0, move_data) in game.move_data.iter().enumerate() {
        total_time += move_data.time;

        if adjudicated_outcome.is_none() {

            if move_data.eval.abs() <= draw_rule.eval {
                draw_count += 1;
            } else {
                draw_count = 0;
            }

            if (ply0 as u32 + 1) / 2 >= draw_rule.from_move &&
               draw_count >= 2 * draw_rule.count {

                rule_applied = Some(RuleType::Draw);
                adjudicated_outcome = Some(GameStats {
                    length: ply0 as u32 + 1,
                    time: total_time,
                    score10: 5,
                });
                continue;
           }

            if move_data.eval <= -(resign_rule.eval as i32) {
                resign_counts[ply0 % 2] += 1;
            } else {
                resign_counts[ply0 % 2] = 0;
            }

            if resign_counts[ply0 % 2] == resign_rule.count {

                rule_applied = Some(RuleType::Resign);
                adjudicated_outcome = Some(GameStats {
                    length: ply0 as u32 + 1,
                    time: total_time,
                    score10: [0, 10][ply0 % 2],
                });
            }
        }
    }

    let actual_outcome = GameStats {
        length: game.move_data.len() as u32,
        time: total_time,
        score10: score10,
    };

    AdjudicationOutcome {
        adjudicated: adjudicated_outcome.unwrap_or(actual_outcome.clone()),
        rule_applied: rule_applied,
        actual: actual_outcome,
    }
}

enum RuleType {
    Resign,
    Draw,
}

struct AdjudicationOutcome {
    actual: GameStats,
    rule_applied: Option<RuleType>,
    adjudicated: GameStats,
}

impl AdjudicationOutcome {
    fn correctly_adjudicated(&self) -> bool {
        self.actual.score10 == self.adjudicated.score10
    }

    fn time_saved(&self) -> u32 {
        if self.correctly_adjudicated() {
            (self.actual.time as i32 - self.adjudicated.time as i32) as u32
        } else {
            0
        }
    }

    fn squared_error10(&self) -> u32 {
       (self.actual.score10 as i32 - self.adjudicated.score10 as i32).pow(2) as u32
    }
}
