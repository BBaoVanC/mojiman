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
        .get_matches();

    let source_dir = match matches.value_of("source_dir") {
        Some(string) => String::from(string),
        None => panic!("Source directory cannot be null!"),
    };
    let output_dir = match matches.value_of("output_dir") {
        Some(string) => String::from(string),
        None => panic!("Output directory cannot be null!"),
    };

    log::debug!("Input directory is {}", &source_dir);
    log::debug!("Output directory is {}", &output_dir);

    let emotes = match mojiman::find_emotes(&source_dir) {
        Ok(emotes) => emotes,
        Err(err) => panic!("Error finding emotes: {:?}", err),
    };

    log::debug!("emotes = {:?}", &emotes);

    if !std::path::Path::new(&output_dir).exists() {
        match std::fs::create_dir(&output_dir) {
            Ok(_) => log::info!("Created output_dir"),
            Err(e) => panic!("Error creating output_dir: {:?}", e),
        };
    }
}
