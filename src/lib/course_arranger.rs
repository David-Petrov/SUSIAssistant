use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};

pub type Set<V> = BTreeSet<V>;
pub type Map<K, V> = BTreeMap<K, V>;
pub type CoursesWithCats<Course, Category> = Map<Course, Set<Category>>;
pub type CategoryRequirements<Category> = Map<Set<Category>, u8>;
pub type Arrangement<Course, Category> = Map<Set<Category>, CoursesWithCats<Course, Category>>;
#[rustfmt::skip]
pub type Conclusion<Course, Category> = Vec<(Arrangement<Course, Category>, (CategoryRequirements<Category>, CoursesWithCats<Course, Category>))>;

pub fn calculate_arrangements<Course, Category>(
    courses_with_cats: CoursesWithCats<Course, Category>,
    category_requirements: CategoryRequirements<Category>,
) -> Conclusion<Course, Category>
where
    Course: Clone + Ord,
    Category: Clone + Ord,
{
    fn go<Course, Category>(
        arrangement: Arrangement<Course, Category>,
        courses_with_cats: CoursesWithCats<Course, Category>,
        category_requirements: CategoryRequirements<Category>,
    ) -> Conclusion<Course, Category>
    where
        Course: Clone + Ord,
        Category: Clone + Ord,
    {
        // Nothing more to satisfy - done (good)
        if category_requirements.is_empty() {
            return vec![(arrangement, (category_requirements, courses_with_cats))];
        };

        // No more courses - done (bad)
        let mut rest_courses_with_cats = courses_with_cats.clone();

        let Some((course, c_cats, valid_category_requirements)) =
            rest_courses_with_cats.iter().find_map(|(course, c_cats)| {
                // Recurse for all valid possibilities/configurations
                let mut valid_category_requirements = category_requirements.clone();
                valid_category_requirements
                    .retain(|cats, _| cats.intersection(&c_cats.clone()).next().is_some());

                if valid_category_requirements.is_empty() {
                    None
                } else {
                    Some((
                        course.clone(),
                        c_cats.clone(),
                        valid_category_requirements.clone(),
                    ))
                }
            })
        else {
            return vec![(arrangement, (category_requirements, courses_with_cats))];
        };

        // Remove (if found) suitable course for dispatch
        let _ = rest_courses_with_cats.remove(&course);

        let course_with_cats = (course.clone(), c_cats.clone());

        valid_category_requirements
            .iter()
            .sorted_by_cached_key(|(_, count)| **count)
            .flat_map(|(cats, _)| {
                let new_arrangement = {
                    let mut arrangement = arrangement.clone();

                    if let Some(this_cats_left_count) = arrangement.get_mut(cats) {
                        this_cats_left_count.append(&mut Map::from([course_with_cats.clone()]));
                    } else {
                        arrangement.insert(cats.clone(), Map::from([course_with_cats.clone()]));
                    }

                    arrangement
                };

                let new_category_requirements = {
                    let mut category_requirements = category_requirements.clone();

                    let category_requirement =
                        category_requirements.get_mut(cats).expect("Impossibru");

                    *category_requirement -= 1;

                    // Remove requirement if fulfilled (0 left)
                    if *category_requirement == 0 {
                        category_requirements.remove(cats);
                    }

                    category_requirements
                };

                go(
                    new_arrangement,
                    rest_courses_with_cats.clone(),
                    new_category_requirements,
                )
            })
            .collect::<Vec<_>>()
    }

    go(Map::new(), courses_with_cats, category_requirements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fmi_elective::*;

    #[test]
    fn test() {
        use ElectiveCategory::*;

        let category_requirements: CategoryRequirements<ElectiveCategory> = Map::from([
            (Set::from([AppliedMaths]), 1),
            (Set::from([CompSciCore]), 2),
        ]);

        let courses_with_cats: CoursesWithCats<&str, ElectiveCategory> = Map::from([
            ("c1", Set::from([AppliedMaths, CompSciCore])),
            ("c2", Set::from([AppliedMaths])),
            ("c3", Set::from([CompSciCore])),
        ]);

        let arrangements = calculate_arrangements(courses_with_cats, category_requirements);
        let optimal_arrangement = arrangements
            .iter()
            .min_by_key(|(_, (_, courses_left))| courses_left.len())
            .unwrap();
        assert_eq!(0, optimal_arrangement.1 .1.len())

        // println!("Optimal distribution: {:?}", optimal_arrangement);
    }
}
