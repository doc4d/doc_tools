use glob::glob;
use regex::Regex;
use std::path::PathBuf;
use std::env;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
};
fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    let directory = &args[1];
    let mut files_map: HashMap<String, HashSet<PathBuf>> = HashMap::new();
    let mut images_used_set: HashSet<String> = HashSet::new();
    let mut number_images = 0;

    for entry in glob(format!("{}**/assets/**/*.png", directory).as_str())?
    .chain(glob(format!("{}**/assets/**/*.PNG", directory).as_str())?)
    {
        match entry {
            Ok(path) => {
                let path_string = path.clone()
                    .into_os_string()
                    .into_string()
                    .map_err(|e| anyhow::Error::msg(e.into_string().unwrap()))?;
                let (_, name) = path_string.rsplit_once("assets/").unwrap();
                files_map
                    .entry(name.to_string())
                    .or_insert(HashSet::from_iter(vec![path.clone()]))
                    .insert(path.clone());
                number_images+=1;
            }
            Err(e) => println!("{:?}", e),
        }
    }
    let re = Regex::new(r"!\[.*?\]\(.*?/assets/(.*?\.\w*)")?;
    for entry in glob(format!("{}**/*.md", directory).as_str())?
        .chain(glob(format!("{}**/*.mdx", directory).as_str())?)
    {
        match entry {
            Ok(path) => {
                dbg!(&path);
                let mut content = String::new();
                let _ = File::open(path)?.read_to_string(&mut content);
                for (_, [lineno]) in re.captures_iter(&content).map(|c| c.extract()) {
                    images_used_set.insert(lineno.to_string());
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
            let sizes = paths.iter().fold(0, |acc, path| {acc + std::fs::metadata(path).unwrap().len()});
            //println!("Image {} is not used", name);
            for path in &paths {
                std::fs::remove_file(path)?;
            }


            total_size += sizes;
        }

    }
    println!("{} images not used {} {}", counter, counter as f64/number_images as f64, (total_size/(1024*1024)) as f64);

    Ok(())
}
