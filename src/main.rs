use std::path::Path;
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

    let source_dir = String::from(matches.value_of("source_dir")
        .expect("source_dir shouldn't be None"));
    let output_dir = String::from(matches.value_of("output_dir")
        .expect("output_dir shouldn't be None"));
    let emote_size = matches.value_of("emote_size")
        .expect("emote_size shouldn't be None");
    let emote_size = emote_size.parse::<u32>()
        .expect("emote_size couldn't be converted to a number");

    log::debug!("Input directory is {}", &source_dir);
    log::debug!("Output directory is {}", &output_dir);

    let emotes = mojiman::find_emotes(&source_dir).expect("Error finding emotes");

    log::debug!("emotes = {:?}", &emotes);
    log::info!("Found {} emotes", emotes.len());

    if !std::path::Path::new(&output_dir).exists() {
        std::fs::create_dir(&output_dir).expect("Error creating output_dir");
        log::info!("Created output_dir");
    }

    let mut do_resize = Vec::new();

    for emote in emotes {
        let source_path = Path::new(&source_dir).join(&emote.file_name);
        let output_path = Path::new(&output_dir).join(&emote.file_name);

        if output_path.is_file() {
            let should_resize = mojiman::is_newer_than(&source_path, &output_path)
                .expect(&format!("Error comparing modification date of {}", emote.file_name)[..]);

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
        mojiman::resize(&source_path, &output_path, emote_size)
            .expect(&format!("Error resizing {}", emote.file_name)[..]);
        log::info!("Resized {}", emote.file_name);
    }
}
