use clap::Parser;
use colored::Colorize;
use glob::glob;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::{collections::HashSet, fs::File, io::Read};

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

    let regex: Regex = Regex::new(r#"\[.*?\]\(([^ \)]*assets\/.*?)( "(.+)")?\)"#)?;
    let mut files_map: HashSet<PathBuf> = HashSet::new();
    let mut images_used_set: HashSet<PathBuf> = HashSet::new();
    let mut has_invalid_links = false;

    for entry in glob(format!("{}**/assets/**/*.png", directory).as_str())?
        .chain(glob(format!("{}**/assets/**/*.PNG", directory).as_str())?)
    {
        match entry {
            Ok(path) => {
                files_map.insert(path.canonicalize()?);
                if verbose {
                    println!("Image found {}", path.display());
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
                let _ = File::open(path.as_path())?.read_to_string(&mut content);
                let mut start = 0;
                let temp = path.as_path().parent();
                if let Some(temp) = temp {
                    while let Some(caps) = regex.captures(&content[start..]) {
                        if let Some(full_match) = caps.get(1) {
                            let link = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                            if !link.starts_with("http") {
                                let final_path = temp.join(std::path::Path::new(link));
                                match fs::canonicalize(final_path) {
                                    Ok(final_path) => {
                                        if verbose {
                                            println!(
                                                "Link found {}",
                                                &final_path.as_path().display()
                                            );
                                        }
                                        images_used_set.insert(final_path);
                                    }
                                    Err(_) => {
                                        has_invalid_links = true;
                                        println!(
                                            "Error with image path {} {}",
                                            link.red(),
                                            path.as_path().display()
                                        )
                                    }
                                }
                            }

                            start += full_match.end();
                        }
                    }
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    for path in images_used_set {
        if files_map.contains(&path) {
            files_map.remove(&path);
        }
    }
    if !files_map.is_empty() {
        if verbose {
            println!("{}", "To DELETE:".red());
        }
        for image in files_map {
            if verbose {
                println!("{} image not used", image.as_path().display());
            }
            if !has_invalid_links {
                list_to_delete.push(image.clone());
            }
        }
    }

    Ok(list_to_delete)
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let directories: Vec<String> = args.paths;
    let mut counter = 0;
    for directory in &directories {
        for entry in glob(directory)? {
            if let Some(mut path) = entry?
                .to_str()
                .map(|str| str.replace(std::path::MAIN_SEPARATOR_STR, "/"))
            {
                path.push('/');
                let vec = find_unused_images(&path, args.verbose)?;
                counter += vec.len();
                if args.fix {
                    for path in vec {
                        std::fs::remove_file(path)?;
                    }
                }
            }
        }
    }
    println!("Number images not used: {}", counter);

    Ok(())
}
