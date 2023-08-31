use std::collections::{HashSet, BTreeMap, BTreeSet};

use itertools::Itertools;
use ngrammatic::{CorpusBuilder, Pad, SearchResult};
use anyhow::Result;

use susi::config::*;
use susi::susi_course::*;
use susi::fmi_elective::*;
use susi::course_arranger::{Arrangement, CategoryRequirements, CoursesWithCats, calculate_arrangements};

#[tokio::main]
async fn main() -> Result<()>{
  // Fetch HTML page with taken courses
  let raw_html = susi::susi_course::fetch_susi_courses_raw_html().await?;
  let electives_susi = susi::susi_course::parse_susi_course_html(&raw_html)?;

  // Courses have been fetched & parsed
  // Starting to calculate categories
  let electives_with_categories: HashSet<ElectiveCourse> = fetch_all_elective_pages_from_fmi_site().await?;

  ///// Fuzzy match category names /////

  // First, build a corpus to hold all the received categories
  let mut corpus = CorpusBuilder::new()
    .arity(2)
    .pad_full(Pad::Auto)
    .finish();

  // Feed the category names to the corpus
  electives_with_categories.iter().for_each(|cat| corpus.add_text(&cat.name));

  // Now try to pair each susi course with it's elective counterpart from FMI's site
  let courses_with_electives_cats =
    electives_susi.into_iter()
      .map(|susi_course| {
        electives_with_categories.iter()
          // First attempt an exact match
          .find(|fmi_elective_course| {
            fmi_elective_course.name == susi_course.name
          })
          // If exact match not found, try fuzzy match with enough similarity
          .or_else(|| {
            let closest_match = corpus
              .search(susi_course.name.as_str(), susi::consts::FUZZY_SEARCH_SIMILARITY_THRESHOLD)
              .into_iter()
              .next();

            // Alert the user right here, right now
            match closest_match {
              None => {
                println!("[WARNING] Course '{}' couldn't be found, so I'm ignoring it.\n", susi_course.name);

                None
              }
              Some(SearchResult { text: name, similarity }) => {
                println!("No exact match found for course '{}'.", susi_course.name);
                println!("Closest match is '{}' with similarity {}.\n", name, similarity);

                // SAFETY: we are sure that there is such an elective course because
                //         it is a result of a fuzzy match over all the courses' names
                Some(electives_with_categories.iter().find(|course| course.name == name).unwrap())
              }
            }
          })
          // Finally pair them together
          .map(|fmi_elective_course| (susi_course, (*fmi_elective_course).clone()))
      })
      .flatten() // Convert from Vec<Option<T>> to Vec<T> from Somes only
      .collect::<Vec<_>>();

  // Convert types for our arrangement calculator
  let courses_with_cats =
    courses_with_electives_cats.into_iter()
      .map(|(susi, fmi)| {
        (susi, fmi.categories.clone().into_iter().collect::<BTreeSet<_>>())
      })
      .collect::<Vec<_>>()
      .into_iter()
      .collect::<BTreeMap<_, _>>();

  // Convert types for our arrangement calculator
  let category_requirements =
    read_categories_config()?.into_iter()
      .map(|(k, v)|
        (k.into_iter().collect::<BTreeSet<_>>(), v)
      )
      .collect::<BTreeMap<_, _>>();

  // Calculate all arrangements
  let result = calculate_arrangements(courses_with_cats, category_requirements);

  // Get best arrangement (lest unarranged courses)
  let (arrangement, (remaining_requirements, remaining_courses)) = result.iter()
    .min_by_key(|(_, (_, courses_left))| courses_left.len())
    .unwrap();

  // Finally, display all arrangements
  display_arrangement_with_context(arrangement, remaining_requirements, remaining_courses);

  Ok(())
}

fn display_arrangement_with_context(
  arrangement: &Arrangement<Course, ElectiveCategory>,
  remaining_requirements: &CategoryRequirements<ElectiveCategory>,
  remaining_courses: &CoursesWithCats<Course, ElectiveCategory>
) {
  println!("\nYour optimal arrangement is:\n");

  arrangement.iter()
    .sorted_by_key(|(cats, _)| cats.len())
    .for_each(|(cats, courses_with_cats)| {
      let cats_string = cats.iter().join(" | ");
      println!("{} ->", cats_string);

      let course_with_cats_string = courses_with_cats.iter()
        .map(|(course, cats)| {
          format!("  '{}' ({})", course.name, cats.iter().join("|"))
        })
        .join("\n");
      println!("{}\n", course_with_cats_string)
    });

  println!();

  if remaining_requirements.is_empty() {
    println!("Hurray! No requirements left!\n");
  } else {
    println!("Requirements left:\n");

    remaining_requirements.iter().for_each(|(categories, count)| {
      println!("  {} -> {}", categories.iter().join(" | "), count);
    })
  }

  println!("\n");

  if remaining_courses.is_empty() {
    println!("You have no unmapped courses, i.e. no courses left that couldn't fit in a category requirement.\n");
  } else {
    println!("Courses left:\n");

    remaining_courses.iter().for_each(|(course, cats)| {
      println!("  {} ({})", course.name, cats.iter().join("|"));
    });
  }
}
