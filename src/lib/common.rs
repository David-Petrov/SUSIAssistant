use std::{
    collections::{HashMap, HashSet},
    fs, str::FromStr,
};

use anyhow::{anyhow, Error, Result, bail, Context};
use futures::future::*;
use itertools::Itertools;
use regex::Regex;
use serde_json::Value;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Semester {
  Winter,
  Summer,
}

impl FromStr for Semester {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    use Semester::*;

    Ok(match s {
      "Summer" => Summer,
      "Winter" => Winter,
      _ => bail!("Unknown semester string."),
    })
  }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ExamSession {
  Regular(Semester),
  Elimination,
}

impl ExamSession {
  pub fn from_susi_index(index: u8) -> Result<Self> {
    use ExamSession::*;

    Ok(match index {
      0 => Regular(Semester::Winter),
      1 => Regular(Semester::Summer),
      2 => Elimination,
      _ => bail!("Unknown exam session index."),
    })
  }
}

//FOR ELECTIVES
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, EnumIter)]
pub enum ElectiveCategory {
  Informatics,
  Practicum,
  Maths,
  AppliedMaths,
  CompSciFundamentals,
  CompSciCore,
  Statistics,
  Seminar,
  Humanitarian,
  Other,
}

impl std::fmt::Display for ElectiveCategory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ElectiveCategory::*;

    let string = match self {
      Informatics => "Информатика",
      Practicum => "Практикум",
      Maths => "Математика",
      AppliedMaths => "Приложна математика",
      CompSciFundamentals => "ОКН",
      CompSciCore => "ЯКН",
      Statistics => "Статистика",
      Seminar => "Семинар",
      Humanitarian => "Хуманитарни",
      Other => "Други",
    };

    write!(f, "{}", string)
  }
}

impl FromStr for ElectiveCategory {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    use ElectiveCategory::*;

    Ok(match s.trim() {
      "Д" | "Др." => Other,
      "И" => Informatics,
      "КП" => Practicum,
      "М" | "M" => Maths,
      "ПМ" => AppliedMaths,
      "ОКН" | "OKН" => CompSciFundamentals,
      "ЯКН" => CompSciCore,
      "Ст" | "Стат" => Statistics,
      // TODO: Figure out why Ivan Gadjev's seminar in Calculus is posted without a category in all of FMI's pages...
      // That's why we have that empty string ugly mapped to Seminar
      "С" | "" => Seminar,
      "Х" => Humanitarian,

      _ => bail!("Unrecognized category '{}'", s),
    })
  }
}

#[derive(Debug, Copy, Clone)]
pub struct ElectiveHorarium {
  pub lecture: u8,
  pub seminar: u8,
  pub practicum: u8,
}

impl ElectiveHorarium {
  fn from_triple([lecture, seminar, practicum]: [u8; 3]) -> Self {
    ElectiveHorarium { lecture, seminar, practicum }
  }

  fn from_str(s: &str) -> Result<Self> {
    let triple: [u8; 3] = s
      .split('+')
      .map(str::parse)
      .collect::<Result<Vec<_>, _>>()
      .context("Incorrect format of hours given.")?
      .try_into()
      // .context("Incorrect number of hour entries.")?;
      .map_err(|err| anyhow!("Incorrect number of hour entries.\n {:?}", err))?;

    Ok(Self::from_triple(triple))
  }
}

#[derive(Debug, Clone)]
pub struct ElectiveCourse {
  pub department: String,
  pub name: String,
  pub horarium: Option<ElectiveHorarium>,
  pub ects: Option<f32>,
  pub categories: Vec<ElectiveCategory>,
}

use std::hash::{Hash, Hasher};

impl Hash for ElectiveCourse {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
  }
}

impl PartialEq for ElectiveCourse {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}

impl Eq for ElectiveCourse {}

impl ElectiveCourse {
  pub fn from_fmi_site_row(args: Vec<String>) -> Result<Self> {

    let department = args
      .get(0)
      .ok_or(anyhow!("Department not found."))?
      .clone();

    let name = args
      .get(1)
      .ok_or(anyhow!("Name not found."))?
      .clone();

    let horarium = {
      let horarium_str = args.get(2).ok_or(anyhow!("Horarium not found."))?;

      ElectiveHorarium::from_str(horarium_str.as_str()).ok()
    };

    let ects = args
      .get(3)
      .ok_or(anyhow!("ECTS not found."))?
      .replace(',', ".")
      .parse()
      .context("Incorrect format of ECTS.")
      .ok();

    let categories = {
      let re = Regex::new(r"\s*[/,]\s*").unwrap();

      re
        .split({
          args
            .get(4)
            .ok_or(anyhow!("Category not found."))?
            .as_str()
        })
        .map(ElectiveCategory::from_str)
        .collect::<Result<Vec<_>>>()?
    };

    Ok(ElectiveCourse {
      department,
      name,
      horarium,
      ects,
      categories,
    })
  }
}

/** PARSING NA SPISUKA S IZBIRAEMITE */
pub async fn fetch_elective_page_from_fmi_site(url: String) -> Result<Vec<ElectiveCourse>> {

  let client = reqwest::Client::builder()
      .build()
      .unwrap();

  let req = client.get(&url); // FIXME
  let res = req.send().await?;
  let text = res.text().await?;
  let text = text.as_str();
  let html = scraper::Html::parse_document(text);
  let selector = scraper::Selector::parse("table:first-of-type tr").unwrap();
  let courses =
    html.select(&selector)
      .skip(1)
      .map(|tr| {
        tr
          .select(&scraper::Selector::parse("td").unwrap())
          .map(|e| e.text().collect::<String>())
          .collect::<Vec<_>>()
      })
      .map(ElectiveCourse::from_fmi_site_row)
      // .map(|x| if x.is_err() {dbg!((&x, &url)); x} else {x}) // FIXME
      .collect::<Result<Vec<_>>>()?;

  Ok(courses)
}

pub async fn fetch_elective_pages_from_fmi_site(semesters: HashSet<(u16, Semester)>) -> Result<HashMap<(u16, Semester), Vec<ElectiveCourse>>> {
  join_all({
      parse_year_url_config()?
        .into_iter()
        .filter(|(sem, _)| semesters.contains(sem))
        .map(|(sem, url)| async move {
          (sem, fetch_elective_page_from_fmi_site(url).await)
        })
  })
    .await
    .into_iter()
    .map(|((year, sem), res_courses): (_, Result<_>)| {
      res_courses.map(|courses| {
        ((year, sem), courses)
      })
    })
    .collect::<Result<Vec<_>>>()
    .map(|res| {
      res
        .into_iter()
        .collect::<HashMap<(_, _), Vec<_>>>()
    })
}

pub async fn fetch_all_elective_pages_from_fmi_site() -> Result<HashSet<ElectiveCourse>> {
  let year_semesters = parse_year_url_config()?.into_keys().collect::<HashSet<_>>();

  let all_electives = fetch_elective_pages_from_fmi_site(year_semesters).await?
    .into_values()
    .flatten()
    .collect::<HashSet<_>>();

  Ok(all_electives)
}

/** PARSING NA KATEGORIIT OT KONFIGA */
pub fn read_categories_config() -> Result<HashMap<Vec<ElectiveCategory>, u8>> {
  let config_file =
    fs::read_to_string("elective_categories_requirements.json")
      .context("Failed to read elective categories requirements config file.")?;

  let config: Value =
    serde_json::from_str(&config_file)
      .context("Failed to parse elective categories requirements config file.")?;

  let mut values: HashMap<Vec<ElectiveCategory>, u8> = HashMap::new();

  match config {
    Value::Object(obj) => {
      for (key, value) in obj {
        let categories: Vec<ElectiveCategory> =
          if key == "_" {
            ElectiveCategory::iter().collect::<Vec<_>>()
          } else if let Ok(c) = parse_categories(&key) {
            c
          } else {
            bail!("Wrong format of category configuration file.")
          };

        if let Some(num) = value.as_u64() {
          values.insert(categories, num as u8);
        }
      }

      Ok(values.into_iter()
        .sorted_by_cached_key(|(cats, _)| cats.len())
        .collect())
    },
    _ => Err(anyhow!("Expected a simple object with category -> count mapping, wrong format.")),
  }
}

fn parse_categories(key: &str) -> Result<Vec<ElectiveCategory>> {
  key
    .split('|')
    .map(str::trim)
    .map(ElectiveCategory::from_str)
    .collect()
}

/** Parse the year -> url mapping from the config file for elective tables from FMI site */
pub fn parse_year_url_config() -> Result<HashMap<(u16, Semester), String>> {
  let config_file =
    fs::read_to_string("elective_archive_urls.json")
      .context("Failed to read elective archive urls config file.")?;

  let config: Value =
    serde_json::from_str(&config_file)
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
