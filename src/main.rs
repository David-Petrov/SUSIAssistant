use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::env;
use itertools::Itertools;
// use nucleo::{self, MatcherConfig};
use ngrammatic::{CorpusBuilder, Pad, SearchResult};

use reqwest::{cookie::CookieStore, header::HeaderMap};
use reqwest::header;
use dotenvy::dotenv;
use scraper::{self, Element};
use susi::common::*;
use susi::*;

use anyhow::{anyhow, Result, bail, Context};

use std::hash::{Hash, Hasher};

async fn fetch() -> Result<String> {
  dotenv()?;

    // Load an existing set of cookies, serialized as json
  // let cookie_store = reqwest_cookie_store::CookieStore::new(None);
  // let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
  // let cookie_store = std::sync::Arc::new(cookie_store);
  
  // Build a `reqwest` Client, providing the deserialized store
  let client = reqwest::Client::builder()
      // .cookie_store(true)
      .cookie_store(true)
      // .cookie_provider(std::sync::Arc::clone(&cookie_store))
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
    // ("__EVENTARGUMENT", ""),
    // ("__VIEWSTATE", ""),
    ("btnSubmit", "Влез")
    ]);
  // let mut headers = HeaderMap::new();
  // let session_cookie = cookie_store.cookies(&reqwest::Url::parse("https://susi.uni-sofia.bg").unwrap());
  // headers.insert(header::COOKIE, session_cookie.unwrap());
  // headers.insert(header::ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
  // headers.insert(header::CACHE_CONTROL, "max-age=0".parse().unwrap());
  // headers.insert(header::CONNECTION, "keep-alive".parse().unwrap());
  // headers.insert(header::CONTENT_TYPE, "application/x-www-form-urlencoded".parse().unwrap());
  // headers.insert(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36".parse().unwrap());
  // headers.insert(header::DNT, "1".parse().unwrap());

  let req = 
    client.post("https://susi.uni-sofia.bg/ISSU/forms/Login.aspx")
    .form(&form)
    ;//.headers(headers);

  let res = req.send().await?;
  // println!("{:?}", res);
  let text = res.text().await?;
  
  // TODO: choose another way to ensure logged-in status
  if text.contains("Вход за студенти в студентска информационна система") {
    let html = scraper::Html::parse_document(text.as_str());
    let msg_selector = scraper::Selector::parse("#PageError1_lblError").unwrap();
    let msg = html.select(&msg_selector).next().unwrap().text().collect::<String>();
    println!("Failed to log in. Message from susi: {}", msg);
    bail!("Exiting.")
  }
  
  let form = HashMap::from([
    ("__EVENTTARGET", "Report_Exams1$btnReportExams"),
    ("Report_Exams1:chkTaken", "on"),
    ("Report_Exams1:chkNotTaken", "on"),
    ("__VSTATE", "eJyFUs1u00AQTrbeVkkAF4EiuCRG4kBBOHEORfKNtKBG%2fAi1BY6WU48TK4632Gul5RQCD8CBO0LiAaKolaqUhgfgMn4ZrrDeJqECpEir2fF8s998M%2bNfWfVGlpYsa4MFPGR%2btA1vYi%2bEFyzidXuv8wQOLau4RG9vge1A%2bAyC2DDP%2fafgcsNsdO0W1GPOWWDQ4jbss5Bbjw7sbmSYe%2b3Ort2BgN78J%2f6ccQmpNG%2fUajWjWquur6tOkbjC5ElWWCV1CSHSZlW1SErKLhxwuolfcYjj5D2eJgM8TgYafseJht9wmH5P7xMc41DDE3HwLPmAk6R%2fEXEcWWR5aonkX3nlRV7Th7ZAlTRSzqbusqzdDsGla23O9yOzUun1errb9fQ48O5HzPVsvdmqiBMwByoPjKrhkCXJkKGr%2bCUZCLVHQttZMjC1OXuG3sFPopeR0HSk4Uc8FSnj5J2wxzjS8HPandA9wZF4k05CkZNQSrnHLIQN5rMwnykp1q2dOlEk7blmUXs6y%2bviwTw2daikULdY6L0Va7f9h77XCgp313YOIw5d%2fTU09ZeN9Jr9FfpfuaSskH5%2fdb6ZcgZE9ZWUt0wL95T%2f47kFeGEBfnnWgLog8eos8dqfJeblhnMRjx0IeGOT%2fPh5RYQvyaFeWDadu78BqPromQ%3d%3d"),
    ("__EVENTVALIDATION", "/wEdABqvVXD1oYELeveMr0vHCmYPk+ke+BAbyLqnQpjlaNqs9BosptrOJf0OGQ2bo6dFRXz8q5+DndxsmV3aV5gYlUQWj8pTA/Ixdz+XpULM/NqDRSZsccKkOKD9sWprJstOcAT9QBJJOUpT2G0QJ47ZnBazIQ8oEXfirBQnzNUuhazC3Xbsn/QfLhJ3qwFT04tAswnkiBBmcDoe6zY89K4o5Q1rtju6WSiyKyaepGAi0n7wjYDRwDzwwRHzE6kj8O+wGsmxwApm9hqmEdcC6T1gvLTHYQNfHa77/eZ1ntMskHcVWQLTISB3i0am0UN+dTycRNPQE2pDkE/K/BHXHTQKuNpjjcNMxb95paTM28SkCGfw84o6V7Vy07Ryjoql50PcvIHlXcFROutUNIsuIM+RKK/7GO1r9BNhmidv+fUDK2Kk/cdvzB8LLIKjZnKk081M9Bkxhot86/SAY+zWQR8lb0g6jbwe8k2SRj8pm7OzxffmJMl4dogi+xEK/srZ1VU/9fwx/S31TrVlmeuLeOP+ves64vbDp3+heRHYRHrvVWXhl1ttx+qVddV8EFD3eR7UWVyacI5z9JYdzrzOao0PdE/8"),
    // ("__VIEWSTATE", ""),
  ]);
  let req = client
    .post("https://susi.uni-sofia.bg/ISSU/forms/students/ReportExams.aspx")
    .form(&form)
    ;//.header(reqwest::header::COOKIE, cookie_store.cookies(&reqwest::Url::parse("https://susi.uni-sofia.bg").unwrap()).unwrap());
  // println!("{:?}", req);
  let res = req.send().await?;
  let text = res.text().await?;
  Ok(text)
}



#[tokio::main]
async fn main() -> Result<()>{
  let text = fetch().await?;//include_str!("../thing.html");
  let text = text.as_str();
  
  let mut file = OpenOptions::new().write(true).open("thing.html").unwrap();
  file.write_all(text.as_bytes()).expect("Unable to write file!");

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
    // println!("For year {}: \n", year);
    
    let tbl = tr.next_sibling_element().unwrap();//tr.select(&adj_tbl_sel).next().unwrap();
    let semester_tables_sel = scraper::Selector::parse(":scope table table").unwrap();
    let semester_tables = tbl.select(&semester_tables_sel);

    for (semester_index, semester_table) in semester_tables.enumerate() {
      let semester = ExamSession::from_susi_index(semester_index as u8)?;

      // println!("Za sem: {semester:?}: \n");

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
  // let year_semesters = electives_susi.iter().map(|c| (c.year, c.semester))

  let electives_with_categories: HashSet<ElectiveCourse> = fetch_all_elective_pages_from_fmi_site().await?;

  //fuzy match cat names
  let mut corpus = CorpusBuilder::new()
    .arity(2)
    .pad_full(Pad::Auto)
    .finish();
  let known_cats = electives_with_categories.iter().map(|c| c.name.clone());
  
  for cat in known_cats {
    corpus.add_text(&cat);
  }

  // type SusiFmiPair = (Course, ElectiveCourse);
  //TODO: If a course already passed has vanished and must be sought in the archive from previous years' tables, we must scrape a bit more.
  let courses_with_electives_cats = //(electives_susi_single_cat, electives_susi_multiple_cat): (Vec<SusiFmiPair>, Vec<SusiFmiPair>) = 
    electives_susi.into_iter()
      .map(|susi_course| {
        let temp = electives_with_categories.iter()
          .find(|fmi_elective_course| {
            fmi_elective_course.name == susi_course.name
          })// || (susi_course.name == "Увод в програмирането - практикум - спец. СИ" && fmi_elective_course.name == "Увод в програмирането - практикум"))
          .or_else(|| {
            let closest_match = corpus.search(susi_course.name.as_str(), 0.9).into_iter().next();
            match closest_match {
              None => {
                println!("Warning! Course {} couldn't be found, so I'm ignoring it.", susi_course.name);
                None
              }
              Some(SearchResult { text, similarity }) => {
                println!("Closest match to {} is {} with similarity {}.", susi_course.name, text, similarity); //TODO: Prettify this message for end user
                Some(electives_with_categories.iter().find(|c| c.name == text).unwrap())
              }
            }
          })
          .context(format!("Your elective {} hasn't been found in FMI's elective course table pages.", susi_course.name)) //TODO: has to do with the above comment
          .map(|fmi_elective_course| (susi_course, (*fmi_elective_course).clone()));
        temp
      })
      .filter_map(Result::ok)
      .collect::<Vec<_>>();

  // let kur = electives_susi.iter().find(|c| c.name == "Увод в програмирането - практикум - спец. СИ").unwrap().clone();

  // let mut corpus = CorpusBuilder::new()
  //   .arity(2)
  //   .pad_full(Pad::Auto)
  //   .finish();

  // let known_cats = electives_with_categories.iter().map(|c| c.name.clone());
  // for cat in known_cats {
  //   corpus.add_text(&cat);
  // }

  // let result = corpus.search("Увод в програмирането - практикум - спец. СИ", 0.9);

  // for k in courses_with_electives_cats {
  //   if k.is_err() {
  //     dbg!("hui");
  //   }
  // }

  // let kur2 = kur.collect::<Result<Vec<_>>>()?;
  let courses_with_cats = courses_with_electives_cats.into_iter().map(|(susi, fmi)| (susi, fmi.categories.clone().into_iter().collect::<BTreeSet<_>>())).collect::<Vec<_>>();
  // dbg!(courses_with_cats);
  let courses_with_cats = courses_with_cats.into_iter().collect::<BTreeMap<_, _>>();
  let category_requirements = read_categories_config()?.into_iter().map(|(k, v)| (k.into_iter().collect::<BTreeSet<_>>(), v)).collect::<BTreeMap<_, _>>();
      
  let result = susi::course_arranger::calculate_arrangements(courses_with_cats, category_requirements);
  let optimal = result.iter().min_by_key(|(_, (_, courses_left))| courses_left.len());
  dbg!(optimal);
      
      
      
      
      
      
      // .into_iter()
      // .partition_map(|pair| {
      //   use itertools::Either;
      //   let fmi_elective_course = &pair.1;
      //   match fmi_elective_course.categories[..] {
      //     [_] => Either::Left(pair),
      //     _ => Either::Right(pair),
      //   }
      // });

  // let electives_susi_single_cat = electives_susi_single_cat.into_iter()
  //   .map(|(susi, fmi)| (susi, fmi.categories[0]))
  //   .collect::<Vec<_>>();


  
  // for (course, cat) in electives_susi_single_cat {
    
  // }
      
  
    

  
  
  //NOW WE START CALCULATING THE BROIKI PER CATEGORY
  //starting with the easy part
  

  //remaining IS THE HARD PART
  // let malka_rekursivna_funkciika = |electives_left: HashMap<Vec<ElectiveCourse>, u32>, categories_left: Vec<&ElectiveCourse>| {

  // };

  Ok(())
}
