use glob::glob;
use oxipng::{optimize, Options};
use std::{env, path};

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    let directory = &args[1];

    for entry in glob(directory)? {
        let path = entry?.to_path_buf();

        // Create default options for Oxipng
        let options = Options::default();
        println!("{}", path.clone().display());
        // Optimize the image
        let in_file = oxipng::InFile::from(&path);
        let out_file = oxipng::OutFile::from_path(path.clone());

        match optimize(&in_file, &out_file, &options) {
            Ok(_) => println!("Optimization successful!"),
            Err(err) => eprintln!("Optimization failed: {}", err),
        }
    }

    Ok(())
}
