use strustle::game::Game;
use strustle::nyt::fetch_solution;

fn main() {
    let solution = match fetch_solution() {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    let mut game = match Game::new(solution.chars().collect::<Vec<char>>()) {
        Ok(game) => game,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };
    let won = game.start_game();
    if !won {
        std::process::exit(1);
    }
}
