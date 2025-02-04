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

    let mut board_selected = false;

    for board in &boards {
        let current_board_selected =
            env::var(format!("CARGO_FEATURE_{}", board.to_ascii_uppercase())).is_ok();

        if current_board_selected && board_selected {
            println!("cargo::warning=Multiple boards selected, please make up your mind...");
            return;
        }
        board_selected = true;
    }

    if !board_selected {
        println!(
            "cargo::error=A board must be selected. Supported boards are: {:?}",
            boards
        );
    }
}
