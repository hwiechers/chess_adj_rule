use chess_pgn_parser::{Game, GameTermination};
use regex::{Captures,Regex};
use super::{GameData, MoveData};

pub struct GameMappingError {
    pub game_number: u32,
    pub error: GameError,
}

pub enum GameError {
    UnknownGameTermination,
    MissingComment { ply: u32 },
    BadComment { ply: u32 },
}

pub fn map_game_data(games: &Vec<Game>)
    -> Result<Vec<GameData>, GameMappingError> {

    let mut result: Vec<GameData> = Vec::with_capacity(games.len());

    let comment_parser = CommentParser::new();

    for (index, game) in games.iter().enumerate() {
        match map_single_game_data(game, &comment_parser) {
            Ok(game_data) => result.push(game_data),
            Err(error) => {
                return Err(GameMappingError {
                    game_number: (index + 1) as u32,
                    error: error });
            }
        }
    }

    Ok(result)
}

fn map_single_game_data(game: &Game, comment_parser: &CommentParser) ->
    Result<GameData, GameError> {

    let score10 = match game.termination {
        GameTermination::WhiteWins => 10,
        GameTermination::DrawnGame => 5,
        GameTermination::BlackWins => 0,
        GameTermination::Unknown => {
            return Err(GameError::UnknownGameTermination);
        }
    };

    let mut move_data_vec : Vec<MoveData> =
        Vec::with_capacity(game.moves.len());

    for (ply, move_) in game.moves.iter().enumerate() {

        let comment_opt = move_.comment.as_ref();
        if comment_opt.is_none() {
            return Err(GameError::MissingComment { ply: ply as u32 });
        }

        let comment = comment_opt.unwrap();
        let result = comment_parser.parse(comment);
        match result {
            Ok(move_data) => move_data_vec.push(move_data),
            Err(()) => {
                return Err(GameError::BadComment {
                    ply: (ply + 1) as u32
                });
            }
        }
    }

    Ok(GameData {
        score10: score10,
        move_data: move_data_vec
    })
}

struct CommentParser {
    re: Regex
}

impl CommentParser {
    pub fn new() -> CommentParser {
        let re = Regex::new(r"(?x)
                ^(?P<sign>(-|\+)?)
                ((?P<mate>M\d+)|((?P<eval>\d+)(\.(?P<eval_dec>\d{2}))))
                /\d+\s
                ((?P<time>\d+)(\.(?P<time_dec>\d{1,3}))?s)
            ").unwrap();

        CommentParser { re: re }
    }

    pub fn parse(&self, comment: &str) -> Result<MoveData, ()> {

        let captures_opt = self.re.captures(comment);
        if captures_opt.is_none() {
            return Err(());
        }

        let captures = captures_opt.unwrap();
        let eval = CommentParser::get_eval(&captures);
        let time = CommentParser::get_time(&captures);

        Ok(MoveData { eval: eval, time: time })
    }

    fn get_eval(captures: &Captures) -> i32 {
        let mut result = 0;

        result += match captures.name("mate") {
            None | Some("") => 0,
            Some(_) => 10000,
        };

        result += match captures.name("eval") {
            None | Some("") => 0,
            Some(value) => 100 * value.parse::<i32>().unwrap(),
        };

        result += match captures.name("eval_dec") {
            None | Some("") => 0,
            Some(value) => value.parse::<i32>().unwrap(),
        };

        result *= match captures.name("sign") {
            None | Some("") | Some("+") => 1,
            Some("-") => -1,
            _ => unreachable!(),
        };

        result
    }

    fn get_time(captures: &Captures) -> u32 {
        let mut result = 0;

        result +=
        match captures.name("time") {
            Some(value) => 1000 * value.parse::<u32>().unwrap(),
            _ => unreachable!(),
        };

        result +=
        match captures.name("time_dec") {
            None | Some("") => 0,
            Some(value) => 10u32.pow((3 - value.len() as i32) as u32) *
                           value.parse::<u32>().unwrap(),
        };

        result
    }
}

#[cfg(test)]
mod tests {
    use super::CommentParser;
    use MoveData;

    #[test]
    fn comment_parsing() {
       let comment_parser =  CommentParser::new();

       assert_eq!(comment_parser.parse("-1.91/13 0.031s"), Ok(MoveData{ eval: -191, time: 31 }));
       assert_eq!(comment_parser.parse("+0.18/15 0.45s"), Ok(MoveData{ eval: 18, time: 450 }));
       assert_eq!(comment_parser.parse("+M17/21 0.020s"), Ok(MoveData{ eval: 10000, time: 20 }));
       assert_eq!(comment_parser.parse("-M26/18 0.022s"), Ok(MoveData{ eval: -10000, time: 22 }));
    }
}
