use std::collections::HashSet;

use susi::common::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
  let courses: HashSet<ElectiveCourse> = fetch_all_elective_pages_from_fmi_site().await?;

  for course in courses {
    println!("{:?}", course);
  }

  Ok(())
}