use std::fs;
use std::error::Error;
use std::path::Path;

#[derive(Debug)]
pub struct Emote {
    pub name: String,
    pub extension: String,
}

pub fn find_emotes(source_dir: &String) -> Result<Vec<Emote>, Box<dyn Error>> {
    let mut emotes = Vec::new();
    let dir = Path::new(source_dir);

    for f in fs::read_dir(dir)? {
        let path = f?.path();
        let name = match path.file_name() {
            Some(name) => String::from(name.to_str().unwrap()),
            None => continue,
        };
        let extension = match path.extension() {
            Some(extension) => String::from(extension.to_str().unwrap()),
            None => continue,
        };
        emotes.push(Emote {
            name,
            extension,
        });
    }
    //println!("source_dir = {:?}, dir = {:?}", source_dir, dir);

    Ok(emotes)
}
