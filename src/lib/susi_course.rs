use crate::consts;
use anyhow::{anyhow, bail, Result, Context, Error};
use std::{hash::{Hash, Hasher}, cmp::Ordering};

use std::collections::HashMap;
use std::env;
use itertools::Itertools;

use dotenvy::dotenv;
use scraper::{Element, Selector, Html};
use std::str::FromStr;



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

#[derive(Debug, Clone)]
pub struct Course {
  pub name: String,
  pub tutor: Option<String>,
  pub year: u16,
  pub exam_session: ExamSession,
  pub is_elective: bool,
  pub is_passed: bool,
  pub grade: Option<f32>,
  pub ects: f32,
}

impl Ord for Course {
  fn cmp(&self, other: &Self) -> Ordering {
    (self.year, self.exam_session, &self.name).cmp(&(other.year, other.exam_session, &other.name))
  }
}

impl PartialOrd for Course {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for Course {
  fn eq(&self, other: &Self) -> bool {
    (self.year, self.exam_session, &self.name) == (other.year, other.exam_session, &other.name)
  }
}

impl Eq for Course { }

impl Hash for Course {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
  }
}

impl Course {
  pub fn from_susi_row(year: u16, semester: ExamSession, args: &Vec<String>) -> Result<Self> {
    let name = args
      .get(0)
      .ok_or(anyhow!("Name not found."))?
      .clone();

    let tutor = {
      let tutor_str = args
        .get(1)
        .ok_or(anyhow!("Tutor not found."))?
        .clone();
      
      if tutor_str.is_empty() {
        None
      } else {
        Some(tutor_str)
      }
    };

    let is_elective = {
      let is_elective_str = args
        .get(2)
        .ok_or(anyhow!("Elective flag not found."))?;

      is_elective_str == "Избираеми"
    };

    let is_passed = {
      let is_passed_str = args
        .get(3)
        .ok_or(anyhow!("Passed flag not found."))?;

      is_passed_str == "да"
    };

    let grade = {
      let grade_str = args
        .get(4)
        .ok_or(anyhow!("Grade not found."))?;

      if grade_str.is_empty() {
        None
      } else {
        Some(grade_str
          .parse::<f32>()
          .context("Incorrect format of grade.")?)
      }
    };

    let ects = args
      .get(5)
      .ok_or(anyhow!("ECTS not found."))?
      .replace(',', ".")
      .parse()
      .context("Incorrect format of ECTS.")?;

    Ok(Course {
      name,
      tutor,
      year,
      exam_session: semester,
      is_elective,
      is_passed,
      grade,
      ects,
    })
  }
}


pub async fn fetch_susi_courses_raw_html() -> Result<String> {
  dotenv()?;

  let client = reqwest::Client::builder()
    .cookie_store(true)
    .build()
    .unwrap();

  let username = env::var("SUSI_USERNAME").context("No `SUSI_USERNAME` in .env")?;
  let password = env::var("SUSI_PASSWORD").context("No `SUSI_PASSWORD` in .env")?;

  let form = HashMap::from([
    ("txtUserName", username.as_str()),
    ("txtPassword", password.as_str()),
    ("__EVENTVALIDATION", "/wEdAASvVXD1oYELeveMr0vHCmYPY3plgk0YBAefRz3MyBlTcHY2+Mc6SrnAqio3oCKbxYY85pbWlDO2hADfoPXD/5tdTuv7w4ACnajvAfo6U9t/biWbGiT2XZmQEcBPUoPMug0="),
    ("__VSTATE", "eJz7z8ifws%2fKZWhsamBhYWBgYsmfIsaUhkKIMDHyizHJsYdlFmcm5aRmpDAxA%2fnyDEAGKz%2b%2fGIscv0d%2bUWZVfl5JYo5jTmZ6HreWZnBlcUlqrl54apJeqCeIcgZKF%2bXnFOuhqWWSY4lXDHZiamgQAFoHtAlkFUtIakVJakoKEzvIfHlGbm0WJnkmFDXyzCB5TgLy3ATkeWEe4SegUBCmUJgfykoBAMHgO5k%3d"),
    ("__EVENTTARGET", ""),
    ("btnSubmit", "Влез")
  ]);

  // Try to login
  let req = client.post(consts::FMI_LOGIN_URL).form(&form);
  let res = req.send().await?;
  let text = res.text().await?;

  // Bail out if we fail to login
  if text.contains("Вход за студенти в студентска информационна система") {
    let html = Html::parse_document(text.as_str());
    let msg_selector = Selector::parse("#PageError1_lblError").unwrap();
    let msg = html.select(&msg_selector).next().unwrap().text().collect::<String>();

    println!("Failed to log in. Message from susi: {}", msg);
    bail!("Exiting.")
  }

  let req = client.get(consts::FMI_EXAMS_REPORT_URL);
  let res = req.send().await?;
  let text = res.text().await?;
  let html = Html::parse_document(text.as_str());

  let [v_state, event_validation] = ["#__VSTATE", "#__EVENTVALIDATION"].map(|field| {
    html.select(&Selector::parse(field).unwrap())
      .next()
      .unwrap()
      .value()
      .attr("value")
      .unwrap()
  });

  // Send request for the grade reports
  let form = HashMap::from([
    ("__EVENTTARGET", "Report_Exams1$btnReportExams"),
    ("Report_Exams1:chkTaken", "on"),
    ("Report_Exams1:chkNotTaken", "on"),
    ("__VSTATE", v_state),
    ("__EVENTVALIDATION", event_validation),
  ]);

  let req = client
    .post(consts::FMI_EXAMS_REPORT_URL)
    .form(&form);
  let res = req.send().await?;
  let raw_html = res.text().await?;

  // Return HTML for grade reports
  Ok(raw_html)
}

pub fn parse_susi_course_html(raw: &String) -> Result<Vec<Course>> {
  let html = Html::parse_document(raw.as_str());

  let selector = Selector::parse("tbody").unwrap();
  let my_table = html.select(&selector).max_by_key(|tbody| tbody.children().count()).unwrap();
  let year_trs_selector = Selector::parse(":scope > tr:nth-child(6n+1)").unwrap();
  let year_trs = my_table.select(&year_trs_selector);

  let mut courses = Vec::<Course>::new();

  for tr in year_trs {
    let year_string = tr.text().collect::<String>().replace(char::is_whitespace, "");

    if !year_string.starts_with("Година:") {
      continue;
    }

    let year = year_string.split(&[':', '/'][..]).map(String::from).collect::<Vec<_>>().get(1).unwrap().clone().parse::<u16>().unwrap();

    let tbl = tr.next_sibling_element().unwrap();//tr.select(&adj_tbl_sel).next().unwrap();
    let exam_session_tables_sel = Selector::parse(":scope table table").unwrap();
    let exam_session_tables = tbl.select(&exam_session_tables_sel);

    for (exam_session_index, exam_session_table) in exam_session_tables.enumerate() {
      let exam_session = ExamSession::from_susi_index(exam_session_index as u8)?;

      let tr_sel = Selector::parse(":scope tr:not(.greyType2)").unwrap(); // TVA ZAKVO ISKA :SCOPE?????

      let td_sel = Selector::parse("td").unwrap();
      let trs_fields = exam_session_table
        .select(&tr_sel)
        .skip(1)
        .filter(|tr| tr.text().any(|td| !td.chars().all(char::is_whitespace)))
        .map(|tr| {
          tr
            .select(&td_sel)
            .map(|td|
              td.text()
              .collect::<String>()
              .trim_matches(char::is_whitespace)
              .to_string()
            ).collect::<Vec<_>>()
        });

      let mut semester_courses = trs_fields
        .map(|fields|
            Course::from_susi_row(year, exam_session, &fields)
              .context(format!("When parsing course from following fields: [{}]", fields.iter().join(", "))))
        .collect::<Result<Vec<_>, _>>()?;

      courses.append(&mut semester_courses);
    }
  }

  let electives_susi = courses
    .into_iter()
    .filter(|c| c.is_elective && c.is_passed)
    .collect::<Vec<_>>();

  Ok(electives_susi)
}