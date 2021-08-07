use std::fs;
use std::path::Path;
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

    let source_dir = match matches.value_of("source_dir") {
        Some(string) => string,
        None => panic!("Source directory cannot be null!"),
    };
    let output_dir = match matches.value_of("output_dir") {
        Some(string) => string,
        None => panic!("Output directory cannot be null!"),
    };

    println!("Input directory name is {}", source_dir);
    println!("Output directory name is {}", output_dir);

    fs::create_dir(output_dir);

    println!("Input directory is {:?}", source_dir);
    println!("Output directory is {:?}", output_dir);

    let mut do_generate = Vec::new();

    for emote in fs::read_dir(source_dir).unwrap() {
        let emote = emote.unwrap();
        let path = emote.path();
        let extension = path.extension();
        if extension == None {
            continue
        }
        let extension = extension.unwrap().to_str().unwrap();

        match extension {
            "png" | "jpg" | "gif" => extension,
            _ => continue
        };
        let name = path.file_name().unwrap();

        let emote = Path::new(source_dir).join(name);
        let out = Path::new(output_dir).join(name);

        println!("out is {:?}", out);
        if out.is_file() {
            let in_last_modified = fs::metadata(&emote).unwrap().modified().unwrap();
            let out_last_modified = fs::metadata(&out).unwrap().modified().unwrap();
            if in_last_modified > out_last_modified {
                do_generate.push(&name);
            } else {
                println!("Skipping {:?}!", name);
            }
        } else {
            do_generate.push(&name);
        }
    }

    println!();
    println!("Files to generate: {:?}", do_generate);
}
