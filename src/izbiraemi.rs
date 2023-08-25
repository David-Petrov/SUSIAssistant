use std::collections::HashMap;
use susi::common::*;
use itertools::Itertools;

fn main() {
  let values = read_categories_config().unwrap();
  // let (single_cats, multiple_cats) : (HashMap<ElectiveCategory, u32>, HashMap<Vec<ElectiveCategory>, u32>) = 
  //   values.into_iter().partition_map(|pair| {
  //     use itertools::Either;

  //     match pair.0[..] {
  //       [c] => Either::left((c, pair.1)),
  //       _ => Either::right(pair),
  //     }
  //   });

  let single_cats: HashMap<ElectiveCategory, u8> = HashMap::from([
    (ElectiveCategory::AppliedMaths, 1),
    (ElectiveCategory::CompSciCore, 2)
  ]);

  let mut cats = single_cats.clone();
  // let cats = std::cell::RefCell::new(single_cats);

  // let mut v = cats.get_mut(&ElectiveCategory::AppliedMaths).unwrap();
  // *v = *v - 1;
  // cats.remove(&ElectiveCategory::AppliedMaths);

  // dbg!(cats.get(&ElectiveCategory::AppliedMaths));

  let courses: Vec<(&str, ElectiveCategory)> = vec![("hui", ElectiveCategory::AppliedMaths), ("kur", ElectiveCategory::CompSciCore), ("saq", ElectiveCategory::AppliedMaths)];

  let reduced_courses = courses.into_iter().filter(|(course, cat)| {
    if let Some(count) = cats.get_mut(&cat) {
      *count -= 1;
      if *count == 0 {
        cats.remove(&cat);
      }
      false
    } else { true }
  }).collect::<Vec<_>>();

  dbg!(cats);
  dbg!(reduced_courses);
}
// type Rtype = Vec<(&'static str, ElectiveCategory)>;
// fn recursor(courses: &mut Rtype, cats: &mut HashMap<ElectiveCategory, u8>) -> () {

//   // let course = courses.pop();
//   if let Some(((course, cat), rest)) = courses.split_first() {
//     if let Some(count) = cats.get_mut(&cat) {
//       *count = *count - 1;
//       if *count == 0 {
//         cats.remove(&cat);
//       }
//       courses.remove(0);
//     } else {
//       return;
//     }
//   } else {
//     return;
//   }

  // let course = courses.get(0)

  // match courses[..] {
  //   [course, rest @ ..] => {
  //     if cats.is_empty() {
  //       return;
  //     }

  //     vec![]
  //   },
  //   [] => (),
  // }
// }