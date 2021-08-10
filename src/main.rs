use std::error::Error;
use std::path::Path;
use std::fs;
use clap::{Arg, App, SubCommand};

fn main() {
    // https://github.com/env-logger-rs/env_logger/issues/47#issuecomment-607475404
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));

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
        .arg(Arg::with_name("emote_size")
            .short("s")
            .long("size")
            .takes_value(true)
            .help("Resolution to resize emotes to")
            .default_value("48"))
        .get_matches();

    let source_dir = match matches.value_of("source_dir") {
        Some(string) => String::from(string),
        None => panic!("Source directory cannot be null!"),
    };
    let output_dir = match matches.value_of("output_dir") {
        Some(string) => String::from(string),
        None => panic!("Output directory cannot be null!"),
    };
    let emote_size = match matches.value_of("emote_size") {
        Some(size) => {
            match size.parse::<u32>() {
                Ok(s) => s,
                Err(err) => panic!("Could not convert emote_size to a number!"),
            }
        },
        None => panic!("Emote size cannot be null!"),
    };

    log::debug!("Input directory is {}", &source_dir);
    log::debug!("Output directory is {}", &output_dir);

    let emotes = match mojiman::find_emotes(&source_dir) {
        Ok(emotes) => emotes,
        Err(err) => panic!("Error finding emotes: {:?}", err),
    };

    log::debug!("emotes = {:?}", &emotes);
    log::info!("Found {} emotes", emotes.len());

    if !std::path::Path::new(&output_dir).exists() {
        match std::fs::create_dir(&output_dir) {
            Ok(_) => log::info!("Created output_dir"),
            Err(e) => panic!("Error creating output_dir: {:?}", e),
        };
    }

    let mut do_resize = Vec::new();

    for emote in emotes {
        let source_path = Path::new(&source_dir).join(&emote.file_name);
        let output_path = Path::new(&output_dir).join(&emote.file_name);

        if output_path.is_file() {
            let should_resize = match mojiman::is_newer_than(&source_path, &output_path) {
                Ok(b) => b,
                Err(err) => panic!("Error comparing modification date of {}: {:?}", emote.file_name, err),
            };

            if should_resize {
                log::debug!("Keep {}", emote.file_name);
                do_resize.push(emote);
            } else {
                log::debug!("Resize {}", emote.file_name);
            }
        } else {
            log::debug!("Resize {}", emote.file_name);
            do_resize.push(emote);
        }
    }

    log::debug!("do_resize = {:?}", do_resize);
    log::info!("Resizing {} emotes", do_resize.len());

    for emote in do_resize {
        let source_path = Path::new(&source_dir).join(&emote.file_name);
        let output_path = Path::new(&output_dir).join(&emote.file_name);
        match mojiman::resize(&source_path, &output_path, emote_size) {
            Ok(_) => log::info!("Resized {}", emote.file_name),
            Err(err) => panic!("Error resizing {}: {:?}", emote.file_name, err),
        };
    }
}
