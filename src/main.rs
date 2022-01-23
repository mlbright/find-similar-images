use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::{error::Error, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
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
        write!(f, "error: {}", self.0)
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
            None => {
                let new_list: Vec<std::path::PathBuf> = vec![entry.into_path()];
                filegroups.insert(metadata.len(), new_list);
            }
        }
    }

    let attr = dssim::Dssim::new();

    for (size, paths) in filegroups.iter() {
        if paths.len() > 1 {
            let files = paths
                .par_iter()
                .map(|file| -> Result<_, String> {
                    let image = dssim::load_image(&attr, &file)
                        .map_err(|e| format!("Can't load {}, because: {}", file.display(), e))?;
                    Ok((file, image))
                })
                .collect::<Result<Vec<_>, _>>();

            let files = match files {
                Ok(f) => f,
                Err(err) => {
                    eprintln!("error: {}", err);
                    continue;
                }
            };

            for index in 0..files.len() - 1 {
                let (file1, original) = &files[index];
                for (file2, modified) in &files[index + 1..] {
                    if original.width() != modified.width()
                        || original.height() != modified.height()
                    {
                        println!(
                            "invalid: image {} has a different size ({}x{}) than {} ({}x{})\n",
                            file2.display(),
                            modified.width(),
                            modified.height(),
                            file1.display(),
                            original.width(),
                            original.height()
                        );
                        continue;
                    }

                    let (v, _) = attr.compare(original, modified);

                    if v < 0.00006 {
                        println!(
                            "similar: {:.8}\t{}\t{}\t{}",
                            v,
                            size,
                            file1.display(),
                            file2.display()
                        );
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
