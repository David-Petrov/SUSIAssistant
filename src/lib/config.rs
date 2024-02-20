use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;

use crate::fmi_elective::*;
use crate::susi_course::Semester;

use std::{collections::HashMap, fs, str::FromStr};

use serde_json::Value;
use strum::IntoEnumIterator;

/** PARSING NA KATEGORIIT OT KONFIGA */
pub fn read_categories_config() -> Result<HashMap<Vec<ElectiveCategory>, u8>> {
    let config_file = fs::read_to_string("elective_categories_requirements.json")
        .context("Failed to read elective categories requirements config file.")?;

    let config: Value = serde_json::from_str(&config_file)
        .context("Failed to parse elective categories requirements config file.")?;

    let mut values: HashMap<Vec<ElectiveCategory>, u8> = HashMap::new();

    match config {
        Value::Object(obj) => {
            for (key, value) in obj {
                let categories: Vec<ElectiveCategory> = if key == "_" {
                    ElectiveCategory::iter().collect::<Vec<_>>()
                } else if let Ok(c) = parse_categories(&key) {
                    c
                } else {
                    bail!("Wrong format of category configuration file.")
                };

                if let Some(num) = value.as_u64() {
                    values.insert(categories, num as u8);
                } else {
                    bail!("Expected a number as a value for a category group in config file.");
                }
            }

            Ok(values
                .into_iter()
                .sorted_by_cached_key(|(cats, _)| cats.len())
                .collect())
        }
        _ => Err(anyhow!(
            "Expected a simple object with category -> count mapping, wrong format."
        )),
    }
}

fn parse_categories(key: &str) -> Result<Vec<ElectiveCategory>> {
    key.split('|')
        .map(str::trim)
        .map(ElectiveCategory::from_str)
        .collect()
}

/** Parse the year -> url mapping from the config file for elective tables from FMI site */
pub fn parse_elective_archive_urls_config() -> Result<HashMap<(u16, Semester), String>> {
    let config_file = fs::read_to_string("elective_archive_urls.json")
        .context("Failed to read elective archive urls config file.")?;

    let config: Value = serde_json::from_str(&config_file)
        .context("Failed to parse elective archive urls config file.")?;

    let mut result: HashMap<(u16, Semester), String> = HashMap::new();

    let Value::Object(config_obj) = config else {
        bail!("Wrong format; config object expected.");
    };

    for (year, semester_urls) in config_obj {
        let year: u16 = year.parse().context("Wrong format of year key.")?;

        let Value::Object(semester_urls_obj) = semester_urls else {
            bail!("Wrong format; semester key expected.");
        };

        for (semester, url) in semester_urls_obj {
            let semester = Semester::from_str(&semester)?;

            let Value::String(url) = url else {
                bail!("Wrong format; url string expected.");
            };

            result.insert((year, semester), url);
        }
    }

    Ok(result)
}
