use clap::Parser;
use colored::Colorize;
use glob::glob;
use regex::Regex;
use std::path::PathBuf;
use std::vec;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    //If the program fix
    #[arg(short, long, default_value_t = false)]
    fix: bool,

    /// The paths to check
    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    paths: Vec<String>,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn find_unused_images(directory: &str, verbose: bool) -> Result<Vec<PathBuf>, anyhow::Error> {
    println!("Directory: {}", directory);
    let mut list_to_delete = Vec::new();
    let asset_folder = format!("assets{}", std::path::MAIN_SEPARATOR_STR);
    let asset_folder_posix = "assets/".to_string();

    let regex: Regex = Regex::new(r#"\[.*?\]\(([^ \)]*assets\/.*?)( "(.+)")?\)"#)?;
    let mut files_map: HashMap<String, HashSet<PathBuf>> = HashMap::new();
    let mut images_used_set: HashSet<String> = HashSet::new();
    let mut number_images = 0;
    for entry in glob(format!("{}**/assets/**/*.png", directory).as_str())?
        .chain(glob(format!("{}**/assets/**/*.PNG", directory).as_str())?)
    {
        match entry {
            Ok(path) => {
                if let Some((_, name)) = path
                    .as_path()
                    .to_str()
                    .map(|str| str.rsplit_once(asset_folder.as_str()))
                    .flatten()
                {
                    let name = name.replace(std::path::MAIN_SEPARATOR_STR, "/");
                    files_map
                        .entry(name.to_string())
                        .or_insert(HashSet::from_iter(vec![path.clone()]))
                        .insert(path.clone());
                    if verbose {
                        println!("Image found {} {}", &name, path.display());
                    }
                    number_images += 1;
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    for entry in glob(format!("{}**/*.md", directory).as_str())?
        .chain(glob(format!("{}**/*.mdx", directory).as_str())?)
    {
        match entry {
            Ok(path) => {
                let mut content = String::new();
                let _ = File::open(path)?.read_to_string(&mut content);
                let mut start = 0;
                while let Some(caps) = regex.captures(&content[start..]) {
                    let full_match = caps.get(1).unwrap();

                    let link = caps.get(1).map(|m| m.as_str()).unwrap();
                    if let Some((_, image)) = link.split_once(&asset_folder_posix) {
                        if verbose {
                            println!("Link found {}", &image);
                        }
                        images_used_set.insert(image.to_string());
                    }
                    start += full_match.end();
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    let mut total_size = 0;
    let mut counter = 0;
    for (name, paths) in files_map {
        if images_used_set.contains(&name) {
        } else {
            counter += paths.len();
            let sizes = paths
                .iter()
                .fold(0, |acc, path| acc + std::fs::metadata(path).unwrap().len());

            for path in &paths {
                println!("{} To Delete: {}", name.red(), path.display());
                list_to_delete.push(path.clone());
            }

            total_size += sizes;
        }
    }
    if counter > 0 {
        println!(
            "{} images not used {} {}",
            counter,
            counter as f64 / number_images as f64,
            (total_size / (1024 * 1024)) as f64
        );
    }

    Ok(list_to_delete)
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let directories: Vec<String> = args.paths;

    for directory in &directories {
        for entry in glob(directory)? {
            if let Some(mut path) = entry?
                .to_str()
                .map(|str| str.replace(std::path::MAIN_SEPARATOR_STR, "/"))
            {
                path.push('/');
                let vec = find_unused_images(&path, args.verbose)?;
                if args.fix {
                    for path in vec {
                        std::fs::remove_file(path)?;
                    }
                }
            }
        }
    }

    Ok(())
}
