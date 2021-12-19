use dssim;
use std::collections::HashMap;
use std::fs;
use std::{error::Error, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_applicable_image(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| {
            s.ends_with(".jpeg")
                || s.ends_with(".JPEG")
                || s.ends_with(".jpg")
                || s.ends_with(".JPG")
                || s.ends_with(".png")
                || s.ends_with(".PNG")
        })
        .unwrap_or(false)
}

#[derive(Debug)]
struct PicuratorError(String);

impl std::fmt::Display for PicuratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for PicuratorError {}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        let err = "Supply a single directory for processing";
        return Err(Box::new(PicuratorError(err.to_owned())));
    }

    let image_dir = &args[1];
    let walker = WalkDir::new(image_dir).into_iter();
    let mut filegroups: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();
        if !entry.file_type().is_file() {
            continue;
        }
        if !is_applicable_image(&entry) {
            continue;
        }
        let metadata = fs::metadata(entry.path())?;
        if metadata.len() == 0 {
            continue;
        }
        match filegroups.get_mut(&metadata.len()) {
            Some(list) => {
                list.push(entry.into_path());
            }
            _ => {
                let mut new_list: Vec<std::path::PathBuf> = Vec::new();
                new_list.push(entry.into_path());
                filegroups.insert(metadata.len(), new_list);
            }
        }
    }

    for (size, paths) in filegroups.iter() {
        if paths.len() > 1 {
            for (i, original) in paths.iter().enumerate() {
                let context1 = dssim::new();
                let original_image: dssim::DssimImage<f32>;
                match dssim::load_image(&context1, original.as_path()) {
                    Ok(im) => {
                        original_image = im;
                    }
                    Err(e) => {
                        println!("error loading image: {:?}", e);
                        continue;
                    }
                }

                for (j, p) in paths.iter().enumerate() {
                    if i < j {
                        let modified_image: dssim::DssimImage<f32>;
                        match dssim::load_image(&context1, p.as_path()) {
                            Ok(im) => {
                                modified_image = im;
                            }
                            Err(e) => {
                                println!("error loading image: {:?}", e);
                                continue;
                            }
                        }

                        if original_image.height() != modified_image.height()
                            || original_image.width() != modified_image.width()
                        {
                            continue;
                        }

                        let val = context1.compare(&original_image, &modified_image);

                        println!(
                            "size={} sim={}:\n\toriginal: {}\n\tmodified: {}",
                            size,
                            val.0,
                            original.display(),
                            p.display(),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
