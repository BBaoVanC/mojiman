use std::fs;
use std::error::Error;
use std::path::Path;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Emote {
    pub name: String,

    #[serde(rename = "type")]
    pub extension: String,

    #[serde(skip_serializing)]
    pub file_name: String,
}

pub fn find_emotes(source_dir: &String) -> Result<Vec<Emote>, Box<dyn Error>> {
    let mut emotes = Vec::new();
    let dir = Path::new(source_dir);

    for f in fs::read_dir(dir)? {
        let path = f?.path();
        let file_name = match path.file_name() {
            Some(file_name) => String::from(file_name.to_str().unwrap()),
            None => continue,
        };
        let name = match path.file_stem() {
            Some(name) => String::from(name.to_str().unwrap()),
            None => continue,
        };
        let extension = match path.extension() {
            Some(extension) => String::from(extension.to_str().unwrap()),
            None => continue,
        };
        if ["png", "jpg", "gif"].contains(&&extension[..]) {
            emotes.push(Emote {
                name,
                extension,
                file_name,
            });
        }
    }
    //println!("source_dir = {:?}, dir = {:?}", source_dir, dir);

    Ok(emotes)
}

pub fn is_newer_than(one: &Path, two: &Path) -> Result<bool, Box<dyn Error>> {
    let one_modified = fs::metadata(one)?.modified()?;
    let two_modified = fs::metadata(two)?.modified()?;
    Ok(one_modified > two_modified)
}

pub fn resize(source_path: &Path, output_path: &Path, size: u32) -> Result<(), Box<dyn Error>> {
    let img = image::open(source_path)?;
    img.resize(size, size, image::imageops::Lanczos3).save(output_path)?;
    Ok(())
}

pub fn make_index_json(name: &String, emotes: &Vec<Emote>) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "path": "emotes",
        "emotes": emotes,
    })
}
