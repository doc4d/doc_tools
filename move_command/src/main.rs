use clap::Parser;
use std::fs;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_name: String,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let extension = args.file_name.split('.').last().unwrap_or("md");
    let regex_link =
        regex::Regex::new(format!(r"\[[^\]]+\]\(([^)]+{}?)\)", args.file_name).as_str())?;

    for entry in glob::glob("./**/commands-legacy/**/*.md")? {
        let path = entry?;
        let content = fs::read_to_string(path.as_path())?;
        let mut new_content = content.clone();
        while let Some(caps) = regex_link.captures(&content) {
            if let Some(link) = caps.get(1).map(|m| m.as_str()) {
                if link.starts_with("../commands") {
                    continue;
                }
                let new_link = format!("../commands/{}", link);
                new_content = content.replace(link, &new_link);
            }
        }
        fs::write(path.as_path(), new_content)?;
    }

    for entry in glob::glob(format!("./**/commands/**/*.{}", extension).as_str())? {
        let path = entry?;
        let content = fs::read_to_string(path.as_path())?;
        let mut new_content = content.clone();
        while let Some(caps) = regex_link.captures(&content) {
            if let Some(link) = caps.get(1).map(|m| m.as_str()) {
                if !link.contains("../commands-legacy/") {
                    continue;
                }
                let new_link = link.replace("../commands-legacy/", "");
                new_content = content.replace(link, &new_link);
            }
        }
        fs::write(path.as_path(), new_content)?;
    }

    for entry in glob::glob(format!("./**/*.{}", extension).as_str())? {
        let path = entry?;
        let path_str = path.as_path().to_str().unwrap();
        if path_str.contains("/commands-legacy/") || path_str.contains("/commands/") {
            continue;
        }
        let content = fs::read_to_string(path.as_path())?;
        let mut new_content = content.clone();
        while let Some(caps) = regex_link.captures(&content) {
            if let Some(link) = caps.get(1).map(|m| m.as_str()) {
                if link.contains("/commands-legacy/") {
                    continue;
                }
                let new_link = link.replace("/commands-legacy/", "/commands/");
                new_content = content.replace(link, &new_link);
            }
        }
        fs::write(path.as_path(), new_content)?;
    }

    for entry in glob::glob(
        format!(
            "i18n/langue/docusaurus-plugin-content-docs/*/commands-legacy/**/{}",
            args.file_name
        )
        .as_str(),
    )? {
        let path = entry?;
        fs::remove_file(path.as_path())?;
    }

    Ok(())
}
