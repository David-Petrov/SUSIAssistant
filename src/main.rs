use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

use reqwest::{cookie::CookieStore, header::HeaderMap};
use reqwest::header;

use scraper;

async fn fetch() -> Result<String, reqwest::Error> {
    // Load an existing set of cookies, serialized as json
  let cookie_store = reqwest_cookie_store::CookieStore::new(None);
  let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
  let cookie_store = std::sync::Arc::new(cookie_store);
  // {
  //   // Examine initial contents
  //   println!("initial load");
  //   let store = cookie_store.lock().unwrap();
  //   for c in store.iter_any() {
  //     println!("{:?}", c);
  //   }
  // }
  
  
  // Build a `reqwest` Client, providing the deserialized store
  let client = reqwest::Client::builder()
      // .cookie_store(true)
      .cookie_provider(std::sync::Arc::clone(&cookie_store))
      .build()
      .unwrap();
    
  let form = HashMap::from([
    ("txtUserName", "davidap"), 
    ("txtPassword", "78603ghjAaBb!."),
    ("__EVENTVALIDATION", "/wEdAASvVXD1oYELeveMr0vHCmYPY3plgk0YBAefRz3MyBlTcHY2+Mc6SrnAqio3oCKbxYY85pbWlDO2hADfoPXD/5tdTuv7w4ACnajvAfo6U9t/biWbGiT2XZmQEcBPUoPMug0="),
    ("__VSTATE", "eJz7z8ifws%2fKZWhsamBhYWBgYsmfIsaUhkKIMDHyizHJsYdlFmcm5aRmpDAxA%2fnyDEAGKz%2b%2fGIscv0d%2bUWZVfl5JYo5jTmZ6HreWZnBlcUlqrl54apJeqCeIcgZKF%2bXnFOuhqWWSY4lXDHZiamgQAFoHtAlkFUtIakVJakoKEzvIfHlGbm0WJnkmFDXyzCB5TgLy3ATkeWEe4SegUBCmUJgfykoBAMHgO5k%3d"),
    ("__EVENTTARGET", ""),
    // ("__EVENTARGUMENT", ""),
    // ("__VIEWSTATE", ""),
    ("btnSubmit", "Влез")
    ]);
  let mut headers = HeaderMap::new();
  // let session_cookie = cookie_store.cookies(&reqwest::Url::parse("https://susi.uni-sofia.bg").unwrap());
  // headers.insert(header::COOKIE, session_cookie.unwrap());
  headers.insert(header::ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
  headers.insert(header::CACHE_CONTROL, "max-age=0".parse().unwrap());
  headers.insert(header::CONNECTION, "keep-alive".parse().unwrap());
  headers.insert(header::CONTENT_TYPE, "application/x-www-form-urlencoded".parse().unwrap());
  headers.insert(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36".parse().unwrap());
  // headers.insert(header::DNT, "1".parse().unwrap());

  let req = 
    client.post("https://susi.uni-sofia.bg/ISSU/forms/Login.aspx")
    .form(&form)
    .headers(headers);

  let res = req.send().await?;
  // println!("{:?}", res);
  let text = res.text().await?;
  if text.contains("Давид") {
    println!("Logged in!");
  } else {
    println!("Fail!");
  }
  // println!("{:?}", res.text().await.unwrap());

  
  // //PRINT CONTENTS
  // {
  //   // Examine the contents of the store.
  //   let store = cookie_store.lock().unwrap();
  //   for c in store.iter_any() {
  //     println!("{:?}", c);
  //   }
  // }

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
    .header(reqwest::header::COOKIE, cookie_store.cookies(&reqwest::Url::parse("https://susi.uni-sofia.bg").unwrap()).unwrap());
  // println!("{:?}", req);
  let res = req.send().await?;
  let text = res.text().await?;
  Ok(text)
}

struct Course {
  name: String,
  tutor: Option<String>,
  is_elective: bool,
  is_passed: bool,
  grade: Option<u8>,
  ects: u8,
}


#[tokio::main]
async fn main() -> Result<(), reqwest::Error>{
  let text = include_str!("../thing.html");
  if text.contains("Runtime Error") {
    println!("ERROR WE NE STAA");
  } else {
    println!("Q, STANA");
  }
  let mut file = OpenOptions::new().write(true).open("thing.html").unwrap();
  file.write_all(text.as_bytes()).expect("Unable to write file!");

  //START PARSING
  let html = scraper::Html::parse_document(text);
  let selector = scraper::Selector::parse("table table table").unwrap();

  for element in html.select(&selector) {
    println!("{:?}", element.select(&scraper::Selector::parse("tr:not(.greyType2)").unwrap())
      .skip(1)
      .map(|tr| {
        let t = tr.text().map(|td| td.trim_matches(char::is_whitespace)).filter(|td| !td.is_empty()).collect::<Vec<_>>();
        println!("{:?}", t);
        t
      })
      .count()
    );
  }

  Ok(())
}
