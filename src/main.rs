use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};
use std::env;
use std::fs::OpenOptions;
use itertools::Itertools;
use ngrammatic::{CorpusBuilder, Pad, SearchResult};

use dotenvy::dotenv;
use scraper::{self, Element};
use susi::common::*;
use susi::*;

use anyhow::{anyhow, Result, bail, Context};

use std::hash::{Hash, Hasher};

async fn fetch() -> Result<String> {
  dotenv()?;

  let client = 
    reqwest::Client::builder()
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

  let req = client
    .post("https://susi.uni-sofia.bg/ISSU/forms/Login.aspx")
    .form(&form);

  let res = req.send().await?;
  let text = res.text().await?;
  
  // Assert login was successful
  if text.contains("Вход за студенти в студентска информационна система") {
    let html = scraper::Html::parse_document(text.as_str());
    let msg_selector = scraper::Selector::parse("#PageError1_lblError").unwrap();
    let msg = html.select(&msg_selector).next().unwrap().text().collect::<String>();

    println!("Failed to log in. Message from susi: {}", msg);
    bail!("Exiting.")
  }

  let grade_report_url = "https://susi.uni-sofia.bg/ISSU/forms/students/ReportExams.aspx";

  let req = client.get(grade_report_url);
  let res = req.send().await?;
  let text = res.text().await?;
  let html = scraper::Html::parse_document(text.as_str());

  let [v_state, event_validation] = ["#__VSTATE", "#__EVENTVALIDATION"].map(|field| html.select(&scraper::Selector::parse(field).unwrap()).next().unwrap().value().attr("value").unwrap());
  
  let form = HashMap::from([
    ("__EVENTTARGET", "Report_Exams1$btnReportExams"),
    ("Report_Exams1:chkTaken", "on"),
    ("Report_Exams1:chkNotTaken", "on"),
    ("__VSTATE", v_state),
    ("__EVENTVALIDATION", event_validation),
  ]);

  let req = client
    .post(grade_report_url)
    .form(&form);

  let res = req.send().await?;
  let text = res.text().await?;
  Ok(text)
}


use std::fs::File;
use std::io::prelude::*;

#[tokio::main]
async fn main() -> Result<()>{
  let text = fetch().await?;//include_str!("../thing.html");
  let text = text.as_str();
  
  // let mut file = OpenOptions::new().write(true).open("thing2.html").unwrap();
  // file.write_all(text.as_bytes()).expect("Unable to write file!");

  //START PARSING
  let html = scraper::Html::parse_document(text);

  let selector = scraper::Selector::parse("tbody").unwrap();
  let my_table = html.select(&selector).max_by_key(|tbody| tbody.children().count()).unwrap();
  let year_trs_selector = scraper::Selector::parse(":scope > tr:nth-child(6n+1)").unwrap();
  let year_trs = my_table.select(&year_trs_selector);

  let mut courses = Vec::<Course>::new();

  for tr in year_trs {
    let year_string = tr.text().collect::<String>().replace(char::is_whitespace, "");
    
    if !year_string.starts_with("Година:") {
      continue;
    }

    let year = year_string.split(&[':', '/'][..]).map(String::from).collect::<Vec<_>>().get(1).unwrap().clone().parse::<u16>().unwrap();

    let tbl = tr.next_sibling_element().unwrap();//tr.select(&adj_tbl_sel).next().unwrap();
    let semester_tables_sel = scraper::Selector::parse(":scope table table").unwrap();
    let semester_tables = tbl.select(&semester_tables_sel);

    for (semester_index, semester_table) in semester_tables.enumerate() {
      let semester = ExamSession::from_susi_index(semester_index as u8)?;

      let tr_sel = scraper::Selector::parse(":scope tr:not(.greyType2)").unwrap(); //TVA ZAKVO ISKA :SCOPE?????

      let td_sel = scraper::Selector::parse("td").unwrap();
      let trs = 
        semester_table.select(&tr_sel)
        .skip(1)
        .filter(|tr| tr.text().any(|td| !td.chars().all(char::is_whitespace)))
        .map(|tr| {
          // let t = tr.text().map(|td| td.trim_matches(char::is_whitespace)).collect::<Vec<_>>();
          let t = tr
            .select(&td_sel)
            .map(|td| 
              td.text()
              .collect::<String>()
              .trim_matches(char::is_whitespace)
              .to_string()
            ).collect::<Vec<_>>();
          t
        });

      let mut semester_courses = trs.map(|tr| Course::from_susi_row(year, semester, tr)).collect::<Result<Vec<_>, _>>()?;
      courses.append(&mut semester_courses);
    }
  }
  // courses.iter().for_each(|c| {dbg!(c);});
  let electives_susi = courses.into_iter().filter(|c| c.is_elective && c.is_passed).collect::<Vec<_>>();

  //COURSES HAVE BEEN FETCHED & PARSED
  //STARTING TO CALCULATE CATEGORIES
  let electives_with_categories: HashSet<ElectiveCourse> = fetch_all_elective_pages_from_fmi_site().await?;

  //fuzzy match category names

  //first, build a corpus to hold all the received categories
  let mut corpus = CorpusBuilder::new()
    .arity(2)
    .pad_full(Pad::Auto)
    .finish();

  //feed the category names to the corpus
  electives_with_categories.iter().for_each(|cat| corpus.add_text(&cat.name));
  

  //
  let courses_with_electives_cats = //(electives_susi_single_cat, electives_susi_multiple_cat): (Vec<SusiFmiPair>, Vec<SusiFmiPair>) = 
    electives_susi.into_iter()
      .map(|susi_course| {
        let temp = electives_with_categories.iter()
          .find(|fmi_elective_course| {
            fmi_elective_course.name == susi_course.name
          })
          .or_else(|| {
            let closest_match = corpus.search(susi_course.name.as_str(), 0.9).into_iter().next();
            match closest_match {
              None => {
                println!("Warning! Course {} couldn't be found, so I'm ignoring it.", susi_course.name);
                println!();
                None
              }
              Some(SearchResult { text, similarity }) => {
                println!("No exact match found for course {}.", susi_course.name);
                println!("Closest match is {} with similarity {}.", text, similarity);
                println!();
                Some(electives_with_categories.iter().find(|c| c.name == text).unwrap())
              }
            }
          })
          .context(format!("Your elective {} hasn't been found in FMI's elective course table pages.", susi_course.name)) //TODO: this context is unncessesary now
          .map(|fmi_elective_course| (susi_course, (*fmi_elective_course).clone()));
        temp
      })
      .filter_map(Result::ok)
      .collect::<Vec<_>>();

  let courses_with_cats = 
    courses_with_electives_cats.into_iter()
      .map(|(susi, fmi)| 
        (susi, fmi.categories.clone().into_iter().collect::<BTreeSet<_>>()))
      .collect::<Vec<_>>();

  let courses_with_cats = courses_with_cats.into_iter().collect::<BTreeMap<_, _>>();

  let category_requirements = 
    read_categories_config()?.into_iter()
      .map(|(k, v)| 
        (k.into_iter().collect::<BTreeSet<_>>(), v))
      .collect::<BTreeMap<_, _>>();
      
  let result = susi::course_arranger::calculate_arrangements(courses_with_cats, category_requirements);
  let optimal = result.iter().min_by_key(|(_, (_, courses_left))| courses_left.len());
  // dbg!(optimal);
  let optimal = optimal.unwrap();

  display(&optimal.0, &optimal.1.0, &optimal.1.1);

  Ok(())
}

fn display(
  arrangement: &susi::course_arranger::Arrangement<Course, ElectiveCategory>,
  remaining_requirements: &susi::course_arranger::CategoryRequirements<ElectiveCategory>,
  remaining_courses: &susi::course_arranger::CoursesWithCats<Course, ElectiveCategory>
) -> () {
  println!("Your optimal arrangement is:\n\n");

  for (cats, courses_with_cats) in arrangement.iter().sorted_by_key(|(c, d)| c.len()) {
    let cats_string = cats.iter().join(" | ");
    println!("{} ->", cats_string);

    let course_with_cats_string = courses_with_cats.iter().map(|(course, cats)| format!("  {} ({})", course.name, cats.iter().join("|"))).join("\n");
    println!("{}\n", course_with_cats_string)
  }

  println!("\n");

  if remaining_requirements.is_empty() {
    println!("Hurray! No requirements left!");
  } else {
    println!("Requirements left:\n");

    for (categories, count) in remaining_requirements {
      println!("  {} -> {}", categories.iter().join(" | "), count);
    }
  }

  println!("\n");

  if remaining_courses.is_empty() {
    println!("You have no unmapped courses, i.e. ne si sa preebal s neshta, det ne ti trqat <3");
  } else {
    println!("Courses left:\n");

    for (course, cats) in remaining_courses {
      println!("  {} ({})", course.name, cats.iter().join("|"));
    }
  }
}