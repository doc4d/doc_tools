use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_name: String,

    #[arg(short, long)]
    doc_folder: String,
}

fn create_regex(file_name: &str, extension: &str) -> Result<Regex, anyhow::Error> {
    Regex::new(
        format!(
            r#"\[.*?\]\(([^ \)]*{}\.{}?)( "(.+)")?\)"#,
            file_name, extension
        )
        .as_str(),
    )
    .map_err(|err| anyhow::Error::msg(err.to_string()))
}

fn process_files<F>(pattern: &str, regex: &Regex, modify_content: F) -> Result<(), anyhow::Error>
where
    F: Fn(&str, &Regex) -> Option<String>,
{
    for entry in glob::glob(pattern)? {
        let path = entry?;
        let content = fs::read_to_string(&path)?;
        if let Some(new_content) = modify_content(&content, regex) {
            fs::write(&path, new_content)?;
            println!("Updated: {}", path.display());
        }
    }
    Ok(())
}

fn replace_links(
    content: &str,
    regex: &Regex,
    link_filter: impl Fn(&str) -> bool,
    link_modifier: impl Fn(&str) -> String,
) -> Option<String> {
    let mut new_content = content.to_string();
    let mut has_changed = false;
    let mut replacements = Vec::new();

    let mut start = 0;
    while let Some(caps) = regex.captures(&new_content[start..]) {
        let full_match = caps.get(1).unwrap();
        let link = caps.get(1).map(|m| m.as_str()).unwrap();

        if link_filter(link) {
            let new_link = link_modifier(link);
            replacements.push((
                start + full_match.start(),
                start + full_match.end(),
                new_link,
            ));
            has_changed = true;
        }

        start += full_match.end();
    }

    for (start, end, replacement) in replacements.into_iter().rev() {
        new_content.replace_range(start..end, &replacement);
    }

    if has_changed {
        Some(new_content)
    } else {
        None
    }
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let mut doc_folder = args.doc_folder;
    if doc_folder.ends_with('/') {
        doc_folder.pop();
    }

    let mut split = args.file_name.split('.');
    let file_name_without_extension = split.next().unwrap_or(&args.file_name);
    let extension = split.next().unwrap_or("md");

    let regex_link = create_regex(file_name_without_extension, extension)?;

    println!("{}/**/commands-legacy/*/*.md", doc_folder);
    println!("Add '../commands/' to the links in commands-legacy");

    process_files(
        &format!("{}/**/commands-legacy/*.md", doc_folder),
        &regex_link,
        |content, regex| {
            replace_links(
                content,
                regex,
                |link| !link.starts_with("../commands"),
                |link| format!("../commands/{}", link),
            )
        },
    )?;

    println!("Remove '../commands-legacy/' from the links in commands folder");

    process_files(
        &format!("{}/docs/**/commands/*.{}", doc_folder, extension),
        &regex_link,
        |content, regex| {
            replace_links(
                content,
                regex,
                |link| link.contains("../commands-legacy/"),
                |link| link.replace("../commands-legacy/", ""),
            )
        },
    )?;

    println!("Replace '/commands-legacy/' to '/commands/' in the other files");

    process_files(
        &format!("{}/docs/**/*.{}", doc_folder, extension),
        &regex_link,
        |content, regex| {
            replace_links(
                content,
                regex,
                |link| !link.contains("/commands-legacy/"),
                |link| link.replace("/commands-legacy/", "/commands/"),
            )
        },
    )?;

    println!("Remove specific files in commands-legacy");

    for entry in glob::glob(&format!(
        "{}/**/commands-legacy/{}",
        doc_folder, args.file_name
    ))? {
        let path = entry?;
        fs::remove_file(&path)?;
        println!("Removed: {}", path.display());
    }

    Ok(())
}
