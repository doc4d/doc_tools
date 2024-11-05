use glob::{glob, glob_with};
use oxipng::{optimize, Options};
use std::env;

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    let directory = &args[1];

    let glob_options = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    //../build/**/*.png
    for entry in glob_with(directory, glob_options)? {
        match entry {
            Ok(path) => {
                // Create default options for Oxipng
                println!("{}", path.clone().display());
                // Optimize the image
                let in_file = oxipng::InFile::from(&path);
                let out_file = oxipng::OutFile::from_path(path.clone());
                let options = Options::default();

                match optimize(&in_file, &out_file, &options) {
                    Ok(_) => println!("Optimization successful!"),
                    Err(err) => println!("Optimization failed: {}", err),
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}
