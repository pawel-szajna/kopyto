use clap::{Parser, Subcommand};

mod chess;

#[cfg(feature = "ui")]
mod ui;
mod uci;

#[derive(Parser, Debug)]
#[command(name = "kopyto")]
#[command(about = "An experimental chess engine")]
struct Args {
    #[command(subcommand)]
    mode: Option<Modes>,
}

#[derive(Subcommand, Debug)]
enum Modes {
    /// Universal Chess Interface mode
    UCI,

    /// Graphical UI, requires "ui" feature (default)
    UI,
}

#[cfg(feature = "ui")]
fn run_ui() {
    let mut ui = ui::UI::new();
    ui.run();
}

#[cfg(not(feature = "ui"))]
fn run_ui() {
    eprintln!("UI feature not enabled during compilation");
    std::process::exit(1);
}

fn run_uci() {
    let mut uci = uci::UCI::new();
    uci.run();
}

fn main() {
    let args = Args::parse();
    match args.mode {
        Some(Modes::UI) | None => run_ui(),
        Some(Modes::UCI) => run_uci(),
    }
}
