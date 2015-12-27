extern crate chess_pgn_parser;
extern crate clap;
extern crate regex;

// http://stackoverflow.com/a/27590832
#[macro_use]
mod macros {

    #[macro_export]
    macro_rules! println_stderr(
        ($($arg:tt)*) => (
            match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
                Ok(_) => {},
                Err(x) => panic!("Unable to write to stderr: {}", x),
            }
        )
    );
}

mod game_data;
mod rule_test;

use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};
use chess_pgn_parser::Game;

use game_data::{GameMappingError, GameError, map_game_data};
use rule_test::main as test_rule_main;

// The evaluation, in centipawns, of the engine after the move
// and the time taken in milliseconds
#[derive(Debug, PartialEq)]
pub struct MoveData {
    eval: i32,
    time: u32,
}

pub struct GameData {
    // We store the score as 10x the usual values and
    // scale down when displaying results.
    // 1-0     => 10
    // 1/2-1/2 => 5
    // 0-1     => 0
    pub score10: u32,
    pub move_data: Vec<MoveData>,
}

fn build_app<'a, 'v, 'ab, 'u, 'h, 'ar>() -> App<'a, 'v, 'ab, 'u, 'h, 'ar> {

    App::new("Chess-Adjudication-Rule-Analyzer")
        .version("0.1")
        .author("Henri Wiechers <henri@wiechers.me>")
        .about("Tool for studying chess adjudication rules")
        .subcommand(SubCommand::with_name("resign")
                    .about("Recommends a resign rule")
                    .arg(Arg::with_name("file")
                             .help("The PGN file to analyze")
                             .index(1)
                             .required(true)))
        .subcommand(SubCommand::with_name("draw")
                    .about("Recommends a draw rule")
                    .arg(Arg::with_name("file")
                             .help("The PGN file to analyze")
                             .index(1)
                             .required(true)))
        .subcommand(SubCommand::with_name("test")
                    .about("Applies <resign_rule> and <draw_rule> on <file>")
                    .arg(Arg::with_name("file")
                             .help("The PGN file to analyze")
                             .index(1)
                             .required(true))
                    .arg(Arg::with_name("resign_rule")
                             .help("The resign rule in format <eval>/<count> or 'none'")
                             .index(2)
                             .required(true))
                    .arg(Arg::with_name("draw_rule")
                             .help("The draw rule in format <move_number>:<eval>/<count> or 'none'")
                             .index(3)
                             .required(true))
                    .arg(Arg::with_name("verbose")
                              .long("verbose")
                              .help("Turns on verbose output"))
                              )
        .subcommand_required_else_help(true)
}


fn main() {

    let matches = build_app().get_matches();

    if let Some(_) = matches.subcommand_matches("resign") {
        println_stderr!("This command isn't implemented yet! :O");
        exit(1);
    }

    if let Some(_) = matches.subcommand_matches("draw") {
        println_stderr!("This command isn't implemented yet! :O");
        exit(1);
    }

    if let Some(ref matches) = matches.subcommand_matches("test") {
        test_rule_main(matches);
    }
}

fn read_games(matches: &ArgMatches) ->  Vec<GameData> {

    let path = matches.value_of("file").unwrap();

    let mut pgn_file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            println_stderr!("error: Can't open file");
            exit(1);
        }
    };

    let mut pgn = String::new();
    match pgn_file.read_to_string(&mut pgn) {
        Ok(_) => { },
        Err(_) => {
            println_stderr!("error: Can't read file");
            exit(1);
        }
    }

    let games: Vec<Game> = match chess_pgn_parser::read_games(&pgn) {
        Ok(games) => games,
        Err(_) => {
            println_stderr!("error: Can't parse pgn file");
            exit(1);
        }
    };

    return match map_game_data(&games) {
        Ok(game_data) => game_data,
        Err(GameMappingError { game_number, error }) => {
            match error {
                GameError::UnknownGameTermination => {
                    println_stderr!("error: Game {} has unknown result",
                                    game_number);
                },
                GameError::MissingComment{ply} => {
                    println_stderr!("error: Game {}, Ply {} - Missing comment",
                                    game_number, ply);
                }
                GameError::BadComment{ply} => {
                    println_stderr!("error: Game {}, Ply {} - Bad comment format",
                                    game_number, ply);
                }
            }
            exit(1);
        }
    };
}
