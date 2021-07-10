extern crate log;
extern crate simplelog;

use simplelog::*;

mod gamegen;
mod types;

fn main() {
    TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let dots = gamegen::generate_dots(10, 10);
    println!("Dots:");
    for dot in dots {
        println!("x: {}, y: {}", dot.0, dot.1);
    }
}
