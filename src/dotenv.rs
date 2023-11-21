// The .env files are parsed based on the file format defined here:
// https://hexdocs.pm/dotenvy/dotenv-file-format.html

use anyhow::Result;
use lazy_static::lazy_static;
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
    // Get .env content from the disk and parse it.
    let source = fs::read_to_string(path)?;
    for (key, value) in parse_dotenv(&source)? {
        env::set_var(key, value);
    }
    Ok(())
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
        Rule::quote => Env::Populate(trim(value, 1, 1)),
        Rule::literal => Env::Ready(trim(value, 1, 1)),
        Rule::multi_quote => Env::Populate(trim(value, 3, 3)),
        Rule::multi_literal => Env::Ready(trim(value, 3, 3)),
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
            let key = trim(variable.as_str(), 2, 1);
            let replacement = SYS_VARIABLES
                .iter()
                .chain(vars.iter())
                .find(|(k, _)| k == &&key)
                .map(|(_, v)| v.to_owned())
                .unwrap_or_default();

            // Replace the variable in the string.
            result.replace(variable.as_str(), &replacement)
        })
}

/// Trims n chars from the start and m chars from the end of the string.
fn trim(value: &'_ str, n: usize, m: usize) -> String {
    let value = String::from(value);
    let value = &value[n..(value.len() - m)];
    value.into()
}
