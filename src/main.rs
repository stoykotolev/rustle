use std::io;

use strustle::game::Game;
use strustle::nyt::fetch_solution;
use strustle::persistence::{self, DailyRecord};
use strustle::render::render_board;

fn main() {
    // Use the same date source as the NYT client so the saved record lines up
    // with the puzzle that would otherwise be fetched.
    let today = chrono::Local::now().date_naive();

    // If today's game was already finished, replay the saved board and exit
    // without contacting NYT or starting a new game. A corrupt or missing save
    // simply yields `None`, falling through to a normal play session.
    if let Some(played) = persistence::load_today(today) {
        render_board(&played.rows, &mut io::stdout()).expect("write to stdout");
        println!("You already played today.");
        if played.won {
            println!("You won!");
        } else {
            let solution: String = played.solution.iter().collect();
            println!("You lost. The word was {solution}.");
            // Mirror the live loss path, which exits non-zero, so replaying a
            // lost game reports the same exit status to the shell.
            std::process::exit(1);
        }
        return;
    }

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

    // Persist the completed game (win or loss). Best-effort: a write failure
    // must never disrupt play, so the result is intentionally ignored.
    let record = DailyRecord::new(today, game.solution_string(), game.guessed_words(), won);
    let _ = persistence::save_result(&record);

    if !won {
        std::process::exit(1);
    }
}
