use clap::Parser;
use colored::Colorize;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;
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
#[derive(PartialEq, Debug)]
struct Param {
    pub name: Option<String>,
    pub param: Option<String>,
}

impl Param {
    #[cfg(test)]
    fn new_from(name: Option<&str>, param: Option<&str>) -> Self {
        Self {
            name: name.map(|s| s.to_string()),
            param: param.map(|s| s.to_string()),
        }
    }
}

static VALID_TYPES: &[&str] = &[
    "Integer",
    "Text",
    "Collection",
    "Object",
    "Boolean",
    "any",
    "Date",
    "Time",
    "Blob",
    "Variant",
    "Real",
    "Pointer",
    "Picture",
    "Null",
];

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
            r"<!--\s*REF #{}\.Params-->([\s\S]*)<!--\s*END REF\s*-->",
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

fn get_syntax_type_return_param(syntax: &str) -> Option<Param> {
    //ASCII
    let mut last_stop = syntax.len();
    let mut name: Option<String> = None;
    let mut type_name: Option<String> = None;
    for (index, c) in syntax.bytes().enumerate().rev() {
        if c == b':' {
            type_name = syntax
                .get(index + 1..last_stop)
                .map(|s| s.trim().to_string());
            last_stop = index;
        } else if c == b'>' {
            name = syntax
                .get(index + 1..last_stop)
                .map(|s| s.trim().to_string());
            break;
        } else if c == b')' || c == b'}' || c == b'*' {
            break;
        }
    }

    let param = Param {
        name,
        param: type_name,
    };

    if param.name.is_none() && param.param.is_none() {
        return None;
    }
    Some(param)
}

fn validate_type(type_to_validate: &str) -> bool {
    if type_to_validate.contains(".") {
        return true;
    }
    VALID_TYPES.contains(&type_to_validate)
}

fn get_type_return_param(
    params: &str,
    syntaxes: &str,
    logger: Arc<Logger>,
) -> Result<Option<String>, anyhow::Error> {
    let mut types: HashSet<String> = HashSet::new();
    for syntax in syntaxes.split("</br>") {
        let return_param = get_syntax_type_return_param(syntax);
        if let Some(ending) = return_param.as_ref().and_then(|p| p.name.clone()) {
            if let Some(new_type) = get_type(ending.as_str(), params, &logger)? {
                types.insert(new_type);
            }
        } else if let Some(type_) = return_param.and_then(|p| p.param) {
            types.insert(type_);
        }
    }

    let mut type_to_give: Option<String> = Some("any".to_string());
    if types.len() > 1 {
        logger.print_warning("Has different types");
        logger.print_complementary_info();
    } else {
        type_to_give = types.iter().next().map(|s| s.clone());
    }

    if let Some(type_to_give) = &type_to_give {
        if !validate_type(type_to_give) {
            logger.print_warning(format!("Invalid type {}", type_to_give).as_str());
            logger.print_complementary_info();
        }
    }

    Ok(type_to_give)
}

fn check_syntax(
    path: &Path,
    content: &str,
    find_command_regex: &Regex,
    args: &Args,
    conversion_map: Rc<HashMap<String, String>>,
) -> Result<String, anyhow::Error> {
    let mut new_content = content.to_string();

    for (_, [command, syntaxes]) in find_command_regex
        .captures_iter(content)
        .map(|c| c.extract())
    {
        let logger = std::sync::Arc::new(Logger {
            current_command: command.to_owned(),
            path: path.display().to_string(),
        });

        let mut params = get_params(content, command)?;
        let old_params = params.clone();
        if args.fix {
            for (key, value) in conversion_map.iter() {
                //fix return type only
                let regex_pattern = format!(r"( \|\s*)({})(\s*\|\s&(#8592|rarr);)", key);
                let replacement: String = String::from(format!("${{1}}{}${{3}}", value));
                let re = Regex::new(regex_pattern.as_str())?;
                params = re
                    .replace_all(params.as_str(), replacement.as_str())
                    .to_string();
            }
            new_content = new_content.replace(old_params.as_str(), params.to_string().as_str());
        }
        let type_to_give = get_type_return_param(params.as_str(), syntaxes, logger.clone())?;
        for syntax in syntaxes.split("</br>") {
            let param = get_syntax_type_return_param(syntax);
            if let Some(ending) = param.as_ref().and_then(|p| p.name.clone()) {
                if let Some(new_type) = &type_to_give {
                    if args.fix {
                        let replace_ending_regex =
                            Regex::new(format!(r"->\s?{}", ending).as_str())?;
                        let mut new_syntax = replace_ending_regex
                            .replace(syntax, format!(": {}", new_type).as_str())
                            .to_string();
                        if let Some(value) = conversion_map.get_key_value(new_type) {
                            let re = Regex::new(format!(r"(:\s)({})", new_type).as_str())?;
                            let replacement: String = String::from(format!("${{1}}{}", value.1));
                            new_syntax = re
                                .replace_all(new_syntax.as_str(), replacement.as_str())
                                .to_string();
                        }
                        new_content = new_content.replace(syntax, new_syntax.to_string().as_str());
                    }
                }
            } else if let Some(type_) = param.and_then(|p| p.param) {
                if !validate_type(type_.as_str()) && conversion_map.contains_key(type_.as_str()) {
                    if args.fix {
                        if let Some(value) = conversion_map.get_key_value(&type_) {
                            let re = Regex::new(format!(r"(:\s)({})", type_).as_str())?;
                            let replacement: String = String::from(format!("${{1}}{}", value.1));
                            let new_syntax =
                                re.replace_all(syntax, replacement.as_str()).to_string();

                            new_content =
                                new_content.replace(syntax, new_syntax.to_string().as_str());
                        }
                    }
                }
            }
        }
    }
    Ok(new_content)
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let mut types: Vec<(Regex, String)> = Vec::new();
    let conversion_map: Rc<HashMap<String, String>> = Rc::new(HashMap::from([
        ("Longint".to_string(), "Integer".to_string()),
        ("String".to_string(), "Text".to_string()),
        ("ListRef".to_string(), "Integer".to_string()),
        ("WinRef".to_string(), "Integer".to_string()),
        ("Expression".to_string(), "any".to_string()),
        ("Mixed".to_string(), "any".to_string()),
        ("DocRef".to_string(), "Time".to_string()),
        ("MenuRef".to_string(), "Text".to_string()),
        ("Number".to_string(), "Integer".to_string()),
        ("Inteiro longo".to_string(), "Integer".to_string()),
        ("Inteiro".to_string(), "Integer".to_string()),
        ("object".to_string(), "Object".to_string()),
        ("Entier long".to_string(), "Integer".to_string())
    ]));

    for (key, value) in conversion_map.clone().iter() {
        let regex_pattern = format!(r"(\|\s*)({})(\s*\|)", key);
        let replacement: String = String::from(format!("${{1}}{}${{3}}", value));
        types.push((Regex::new(regex_pattern.as_str())?, replacement));
    }

    let find_command_regex =
        Regex::new(r"<!--\s*REF #(.*?)\.Syntax\s*-->(.*?)<!--\s*END REF\s*-->")?;
    for path in &args.paths {
        for entry in glob::glob(path.as_str())? {
            let path = entry?;
            let content = std::fs::read_to_string(path.as_path())?;
            let mut new_content = content;

            new_content = check_syntax(
                path.as_path(),
                new_content.as_str(),
                &find_command_regex,
                &args,
                conversion_map.clone(),
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
        let result = get_syntax_type_return_param("**function()**");
        assert_eq!(result, None);

        let result = get_syntax_type_return_param("**function**()-> Function Result");
        assert_eq!(result, Some(Param::new_from(Some("Function Result"), None)));

        let result = get_syntax_type_return_param("**function**()-> Function Result : Collection");
        assert_eq!(
            result,
            Some(Param::new_from(Some("Function Result"), Some("Collection")))
        );

        let result = get_syntax_type_return_param("**function**($a : Text) : Collection");
        assert_eq!(result, Some(Param::new_from(None, Some("Collection"))));

        let result = get_syntax_type_return_param("**.original** : Collection");
        assert_eq!(result, Some(Param::new_from(None, Some("Collection"))));

        let result = get_syntax_type_return_param("**.original**");
        assert_eq!(result, None);
    }
}
