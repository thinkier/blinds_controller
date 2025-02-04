use std::collections::HashMap;
use std::{env, fs};
use toml::{self, Table};

fn main() {
    // Skip checks if it's not a release build
    if env::var("PROFILE").unwrap() != "release" {
        return;
    }

    let cargo = fs::read_to_string("Cargo.toml").unwrap();
    let conf = toml::from_str::<Table>(&cargo).unwrap();
    let boards = conf
        .get("package")
        .unwrap()
        .get("metadata")
        .unwrap()
        .get("supported_boards")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();

    let board_envar_names = boards
        .iter()
        .map(|board| {
            (
                format!("CARGO_FEATURE_{}", board.to_ascii_uppercase()),
                *board,
            )
        })
        .collect::<HashMap<_, _>>();

    let boards = env::vars()
        .map(|(k, _)| k)
        .map(|be| board_envar_names.get(&be))
        .filter(|b| b.is_some())
        .map(|f| *f.unwrap())
        .collect::<Vec<_>>();

    let boards_count = boards.len();
    if boards_count > 1 {
        println!("cargo::warning=Multiple boards selected: {:?}", boards);
    } else if boards_count == 0 {
        println!(
            "cargo::error=A board must be selected. Supported boards are: {:?}",
            boards
        );
    }
}
