use crate::common::*;
use anyhow::{anyhow, Result, Context};
use std::{hash::{Hash, Hasher}, cmp::Ordering};

#[derive(Debug, Clone)]
pub struct Course {
  pub name: String,
  pub tutor: Option<String>,
  pub year: u16,
  pub semester: ExamSession,
  pub is_elective: bool,
  pub is_passed: bool,
  pub grade: Option<f32>,
  pub ects: f32,
}

impl Ord for Course {
  fn cmp(&self, other: &Self) -> Ordering {
    (self.year, self.semester, &self.name).cmp(&(other.year, other.semester, &other.name))
  }
}

impl PartialOrd for Course {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for Course {
  fn eq(&self, other: &Self) -> bool {
    (self.year, self.semester, &self.name) == (other.year, other.semester, &other.name)
  }
}

impl Eq for Course { }

impl Hash for Course {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
  }
}

impl Course {
  pub fn from_susi_row(year: u16, semester: ExamSession, args: Vec<String>) -> Result<Self> {
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
      semester,
      is_elective,
      is_passed,
      grade,
      ects,
    })
  }
}
