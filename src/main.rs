use std::fs;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("mojiman")
        .author("BBaoVanC <bbaovanc@bbaovanc.com")
        .arg(Arg::with_name("source_dir")
            .short("i")
            .long("source")
            .takes_value(true)
            .help("Directory containing the source emotes")
            .default_value("emotes"))
        .arg(Arg::with_name("output_dir")
            .short("o")
            .long("out")
            .takes_value(true)
            .help("Directory to output public files to")
            .default_value("public"))
        .get_matches();

    let source = match matches.value_of("source_dir") {
        Some(string) => string,
        None => panic!("Source directory cannot be null!"),
    };
    let output = match matches.value_of("output_dir") {
        Some(string) => string,
        None => panic!("Output directory cannot be null!"),
    };

    println!("Input directory is {}", source);
    println!("Output directory is {}", output);
}
