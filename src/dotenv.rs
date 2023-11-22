// Dotenv File Format
//
// The parsing of the .env files is conducted in accordance with the
// specified file format outlined in the following document.
//
// https://hexdocs.pm/dotenvy/dotenv-file-format.html

use anyhow::bail;
use anyhow::Error as E;
use anyhow::Result;
use lazy_static::lazy_static;
use path_absolutize::*;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "dotenv.pest"]
struct DotEnvParser;

enum Env {
    /// Env variable is ready.
    Ready(String),
    /// Env variable needs variable population.
    Populate(String),
}

/// Populates system variables with custom ones.
pub fn load_env_file<P: AsRef<Path>>(path: P) -> Result<()> {
    // Get .env content from the disk.
    let path = path.as_ref().absolutize()?;
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(_) => bail!("File not found \"{}\"", path.display()),
    };
    // Use custom parser for dotenv files.
    parse_dotenv(&source)
        .map_err(|e| E::msg(format!("Couldn't parse environment variables:\n{:?}", e)))
        .and_then(|variables| {
            for (key, value) in variables {
                env::set_var(key, value);
            }
            Ok(())
        })
}

/// Parse the .env file source.
fn parse_dotenv(source: &str) -> Result<HashMap<String, String>> {
    // Parse the input using the 'dotenv' rule.
    let mut variables = HashMap::new();
    let parse = DotEnvParser::parse(Rule::dotenv, source)?;

    // As we have parsed dotenv, which is a top level rule, there
    // cannot be anything else.
    for pair in parse.into_iter().next().unwrap().into_inner() {
        if let Some((key, value)) = parse_env_variable(pair) {
            // Handle variable substitution if necessary.
            let value = match value {
                Env::Ready(value) => value,
                Env::Populate(value) => populate(&value, &variables),
            };
            variables.insert(key, value);
        }
    }

    Ok(variables)
}

/// Parse a single environment variable.
fn parse_env_variable(pair: Pair<Rule>) -> Option<(String, Env)> {
    match pair.as_rule() {
        // The format of the rule is `key ~ "=" ~ value`.
        Rule::variable => {
            let mut rules = pair.into_inner();
            let key = rules.next().unwrap().as_str();
            let value = parse_value(rules.next().unwrap());
            Some((key.into(), value))
        }
        _ => None,
    }
}

/// Parse a value, which might be a string or a naked variable.
fn parse_value(pair: Pair<Rule>) -> Env {
    // Extract the value type from the pair.
    let inner = pair.clone().into_inner().next().unwrap();
    let value = inner.as_str();

    // Handle each type the correct way.
    match inner.as_rule() {
        Rule::quote => Env::Populate(substring(value, 1, 1)),
        Rule::literal => Env::Ready(substring(value, 1, 1)),
        Rule::multi_quote => Env::Populate(substring(value, 3, 3).trim().into()),
        Rule::multi_literal => Env::Ready(substring(value, 3, 3).trim().into()),
        Rule::chars => Env::Populate(value.into()),
        _ => unreachable!(),
    }
}

lazy_static! {
    // Regex to match "${x}" keywords.
    static ref VAR_REGEX: Regex = Regex::new(r"\$\{[^}]+\}").unwrap();
    // Stores all systems env variables.
    static ref SYS_VARIABLES: HashMap<String, String> = HashMap::from_iter(env::vars());
}

/// Handles variable substitutions for env variables.
fn populate(value: &str, vars: &HashMap<String, String>) -> String {
    // Iterate through variables and replace them.
    VAR_REGEX
        .find_iter(value)
        .fold(value.to_owned(), |result, variable| {
            // Extract the key and perform a search in envs table.
            let key = substring(variable.as_str(), 2, 1);
            let system = SYS_VARIABLES.iter();
            let replacement = vars
                .iter()
                .chain(system)
                .find(|(k, _)| k == &&key)
                .map(|(_, v)| v.to_owned())
                .unwrap_or_default();

            // Replace the variable in the string.
            result.replace(variable.as_str(), &replacement)
        })
}

/// Trims n chars from the start and m chars from the end of the string.
fn substring(value: &'_ str, n: usize, m: usize) -> String {
    let value = String::from(value);
    let value = &value[n..(value.len() - m)];
    value.into()
}

#[cfg(test)]
mod tests {
    use super::parse_dotenv;
    use std::collections::HashMap;

    #[test]
    fn empty() {
        assert_eq!(parse_dotenv("").unwrap(), HashMap::new())
    }

    #[test]
    fn single_variable() {
        let chars = parse_dotenv("FOO=BAR").unwrap();
        let quotes = parse_dotenv("FOO=\"BAR\"").unwrap();
        let literal = parse_dotenv("FOO='BAR'").unwrap();
        let expected = HashMap::from_iter(vec![("FOO".into(), "BAR".into())]);
        assert_eq!(chars, expected);
        assert_eq!(quotes, expected);
        assert_eq!(literal, expected);
    }

    #[test]
    fn single_variable_with_export() {
        let variables = parse_dotenv("export FOO = BAR").unwrap();
        let expected = HashMap::from_iter(vec![("FOO".into(), "BAR".into())]);
        assert_eq!(variables, expected);
    }

    #[test]
    fn variable_interpolation() {
        let source = r#"
            USER=admin
            EMAIL=${USER}@example.org
        "#;
        let expected = HashMap::from_iter(vec![
            ("USER".into(), "admin".into()),
            ("EMAIL".into(), "admin@example.org".into()),
        ]);
        assert_eq!(parse_dotenv(&source).unwrap(), expected);
    }

    #[test]
    fn multi_line_quote() {
        let source = r#"
            MESSAGE_TEMPLATE="""
                Hello,
                Nice to meet you!
            """
        "#;
        let expected = HashMap::from_iter(vec![(
            "MESSAGE_TEMPLATE".into(),
            "Hello,\n                Nice to meet you!".into(),
        )]);
        assert_eq!(parse_dotenv(&source).unwrap(), expected);
    }

    #[test]
    fn multi_line_literal() {
        let source = r#"
            MESSAGE_TEMPLATE='''
                Hello,
                Nice to meet you!
            '''
        "#;
        let expected = HashMap::from_iter(vec![(
            "MESSAGE_TEMPLATE".into(),
            "Hello,\n                Nice to meet you!".into(),
        )]);
        assert_eq!(parse_dotenv(&source).unwrap(), expected);
    }

    #[test]
    fn comments() {
        let source = r#"
            # This is a comment
            SECRET_KEY=YOURSECRETKEYGOESHERE # also a comment
            SECRET_HASH="--#-this-is-not-a-comment"
        "#;
        let expected = HashMap::from_iter(vec![
            ("SECRET_KEY".into(), "YOURSECRETKEYGOESHERE".into()),
            ("SECRET_HASH".into(), "--#-this-is-not-a-comment".into()),
        ]);
        assert_eq!(parse_dotenv(&source).unwrap(), expected);
    }
}
