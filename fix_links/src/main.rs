use clap::Parser;
use glob::glob;
use regex::Regex;
use std::{fs, path};

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

fn link_modifier(in_link: &str) -> Result<Option<String>, anyhow::Error> {
    let mut link = in_link.to_string();
    let is_doc_link = link.starts_with("https://developer.4d.com/docs") || link.starts_with("../");
    if link.starts_with("https://developer.4d.com/docs")
    {
        link = urlencoding::decode(&link)?.to_string();
        let regex = Regex::new(r#"https://developer\.4d\.com/docs/(([0-9]{2}(R[0-9]+)?)|(en|fr|pt|ja|es))?/?(.*)"#)?;
        link = link.replace("/#", "#");
        link = regex.replace(&link, "../$5").to_string();
        if is_doc_link && link.ends_with("/") {
            link.pop();
        }
        let mut link_modified = link.clone();
        if !link.contains(".md")
        {
            link_modified = link.find("#").map_or_else(|| {
                let mut temp_link = link.clone();
                temp_link.push_str(".md");
                temp_link
            }, |i| {
                let mut temp_link = link.clone();
                temp_link.insert_str(i, ".md");
                temp_link
            });
        }

        link = link_modified;
    }

    if is_doc_link && link.ends_with("/") {
        link.pop();
    }

    if in_link != link {
        return Ok(Some(link));
    }

    Ok(None)
}

fn fix_links(new_content: &mut String, regex: &Regex) -> Result<bool, anyhow::Error> {
    let mut replacements = Vec::new();
    let mut has_changed = false;
    let mut start = 0;

    while let Some(caps) = regex.captures(&new_content[start..]) {
        let full_match = caps.get(1).unwrap();
        let link = caps.get(1).map(|m| m.as_str()).unwrap();
        if let Some(link_modified) = link_modifier(link)? {
            println!("Link: {} {}", link, link_modified);

            replacements.push((
                start + full_match.start(),
                start + full_match.end(),
                link_modified,
            ));
            has_changed = true;
        }

        start += full_match.end();
    }

    for (start, end, replacement) in replacements.into_iter().rev() {
        new_content.replace_range(start..end, &replacement);
    }
    Ok(has_changed)
}



fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    //regex to find markdown links
    let regex: Regex = Regex::new(r#"\[.*?\]\(([^ \)]*/.*?)( "(.+)")?\)"#)?;

    for directory in args.paths {
        for entry in glob(format!("{}/**/*.md", directory.as_str()).as_str())? {
            let path = entry?;
            let content = fs::read_to_string(path.as_path())?;
            let mut new_content = content.clone();
            let has_changed = fix_links(&mut new_content, &regex)?;

            if args.fix && has_changed {
                fs::write(path.as_path(), new_content)?;
            }
        }
    }
    Ok(())
}
