//! Simple program to run the marked library. By default the original text,
//! AST and HTML will all be written to the console. Various flags allow
//! turning bits off if desired.

#![deny(clippy::all, clippy::pedantic)]

use clap::{App, Arg};
use std::fs;

fn main() {
    let matches = App::new("mark")
        .version("0.1")
        .author("dan sinclair <dj2@everburning.com")
        .about("mark conversion")
        .arg(
            Arg::with_name("o")
                .short("o")
                .long("skip-original")
                .takes_value(false)
                .help("Skip printing original text"),
        )
        .arg(
            Arg::with_name("a")
                .short("a")
                .long("skip-ast")
                .takes_value(false)
                .help("Skip printing AST"),
        )
        .arg(
            Arg::with_name("s")
                .short("s")
                .long("skip-html")
                .takes_value(false)
                .help("Skip printing HTML"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let skip_original = matches.is_present("o");
    let skip_ast = matches.is_present("a");
    let skip_html = matches.is_present("s");
    let filename = &matches.value_of("INPUT").unwrap();

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    if !skip_original {
        println!("{}\n", contents);
    }
    if !skip_ast {
        println!("{:#?}\n", mark::to_ast(&contents));
    }
    if !skip_html {
        println!("{}\n", mark::to_html(&contents));
    }
}
