use clap::Parser;
use colored::Colorize;
use regex::Regex;
use std::{collections::HashSet, fs, path::Path};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    //If the program fix
    #[arg(short, long, default_value_t = false)]
    fix: bool,

    /// The paths to check
    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    paths: Vec<String>,
}

struct Logger {
    pub current_command: String,
    pub path: String,
}

impl Logger {
    fn print_complementary_info(&self) {
        println!("{} {}", self.current_command, self.path);
    }

    fn print_warning(&self, message: &str) {
        println!("{} {} {}", "WARN".yellow(), self.current_command, message);
    }
}

fn get_params(content: &str, command: &str) -> Result<String, anyhow::Error> {
    let find_params_regex: Regex = Regex::new(
        format!(
            "<!--REF #_command_\\.{}\\.Params-->([\\s\\S]*)<!-- END REF-->",
            command
        )
        .as_str(),
    )?;
    let mut result = vec![];
    for (_, [params]) in find_params_regex
        .captures_iter(content)
        .map(|c| c.extract())
    {
        result.push(params);
    }
    Ok(result.join(""))
}

fn get_type(
    in_name: &str,
    in_params: &str,
    logger: &Logger,
) -> Result<Option<String>, anyhow::Error> {
    let mut function_result: Option<String> = None;

    let find_params_regex: Regex =
        Regex::new(format!(r"{}\s?\|\s?(.*?)\|", in_name.trim()).as_str())?;
    let mut return_types = vec![];
    for (_, [return_type]) in find_params_regex
        .captures_iter(in_params)
        .map(|c| c.extract())
    {
        return_types.push(return_type);
        if return_type.contains(",") {
            function_result = Some("any".to_owned());
        } else {
            function_result = Some(return_type.trim().to_string());
        }
    }

    if return_types.len() > 1 {
        logger.print_warning("Multiple return types");
        logger.print_complementary_info();
        return Ok(None);
    }
    Ok(function_result)
}

fn get_ending_param_name(syntax: &str) -> Option<String> {
    let end_result: Vec<&str> = syntax.split("->").collect();

    end_result
        .get(1)
        .filter(|str| !str.contains(":"))
        .map(|s| s.trim().to_string())
}

fn replace_types(content: String) -> Result<String, anyhow::Error> {
    let re = Regex::new(r"(\|\s*)(Longint)(\s*\|)")?;
    let mut new_content = re
        .replace_all(content.as_str(), "${1}Integer${3}")
        .to_string();

    let re = Regex::new(r"(\|\s*)(String)(\s*\|)")?;
    new_content = re
        .replace_all(new_content.as_str(), "${1}Text${3}")
        .to_string();
    Ok(new_content)
}

fn check_syntax(
    path: &Path,
    content: &str,
    find_command_regex: &Regex,
    args: &Args,
) -> Result<String, anyhow::Error> {
    let mut new_content = content.to_string();

    for (_, [command, syntaxes]) in find_command_regex
        .captures_iter(content)
        .map(|c| c.extract())
    {
        let logger = Logger {
            current_command: command.to_owned(),
            path: path.display().to_string(),
        };
        //println!("{}", command);
        //println!("{}", syntaxes);
        let params = get_params(content, command)?;
        let mut types: HashSet<String> = HashSet::new();
        for syntax in syntaxes.split("</br>") {
            if let Some(ending) = get_ending_param_name(syntax) {
                if let Some(new_type) = get_type(ending.as_str(), params.as_str(), &logger)? {
                    types.insert(new_type);
                }
            }
        }

        let mut type_to_give: Option<&str> = Some("any");
        if types.len() > 1 {
            logger.print_warning("Has different types");
            logger.print_complementary_info();
        } else {
            type_to_give = types.iter().next().map(|x| x.as_str());
        }

        for syntax in syntaxes.split("</br>") {
            if let Some(ending) = get_ending_param_name(syntax) {
                if let Some(new_type) = type_to_give {
                    if args.fix {
                        let replace_ending_regex =
                            Regex::new(format!(r"->\s?{}", ending).as_str())?;
                        let new_syntax = replace_ending_regex
                            .replace(syntax, format!(": {}", new_type).as_str());
                        new_content = new_content.replace(syntax, new_syntax.to_string().as_str());
                    }
                }
            }
        }
    }
    Ok(new_content)
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let find_command_regex =
        Regex::new(r"<!--\sREF #(.*?)\.Syntax\s*-->(.*?)<!--\s*END REF\s*-->")?;
    for path in &args.paths {
        for entry in glob::glob(path.as_str())? {
            let path = entry?;
            let content = std::fs::read_to_string(path.as_path())?;
            let mut new_content = content;
            if args.fix {
                new_content = replace_types(new_content)?;
            }
            new_content = check_syntax(
                path.as_path(),
                new_content.as_str(),
                &find_command_regex,
                &args,
            )?;
            fs::write(path.as_path(), new_content)?;
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ending_param() {
        let result = get_ending_param_name("**function()**");
        assert_eq!(result, None);

        let result = get_ending_param_name("**function**()-> Function Result");
        assert_eq!(result, Some("Function Result".to_string()));

        let result = get_ending_param_name("**function**()-> Function Result : Collection");
        assert_eq!(result, None);
    }
}