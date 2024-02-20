# SUSI Assistant

This project started as my final project for FMI's Rust course, but I intend to work it out as a finished idea.

## Features
The SUSI assistant is meant for students at the Faculty of Mathematics and Informatics at SU "St. Kliment Ohridski". It can:
- scrape the user's (a student) elective course data and calculate an optimal distribution of passed courses in accordance with the provided requirement configuration.

## Usage

- Create your own local copy of `.env.example` with the name `.env` where you enter your actual user name and password for FMI's SUSI.

- Setup your specialty's requirements for elective courses in `elective_categories_requirements.json` according to the following scheme:
```json
{
  "<ElectiveCategory1>": <Count1>,
  "<ElectiveCategory2> [| <ElectiveCategory2> | ...]": <Count2>,
  ...
  "<ElectiveCategoryN>": <CountN>,
  ["_": <CountAnyCategory>]
}
```
I've left my specialty's requirements configuration as an example to illustrate the idea.

A comprehensive list of all `"ElectiveCategoryN"` keys is extracted directly from FMI's pages:

| Ключ                      | Категория                               |
|---------------------------|-----------------------------------------|
| `"Д"`, `"Др."`, `"Други"` | Други                                   |
| `"И"`                     | Информатика                             |
| `"КП"`                    | Компютърен практикум                    |
| `"М"`                     | Математика                              |
| `"ПМ"`                    | Приложна математика                     |
| `"ОКН"`                   | Основи на компютърните науки            |
| `"ЯКН"`                   | Ядро на компютърните науки              |
| `"Стат"`, `"Ст"`          | Статистика                              |
| `"С"`                     | Семинар                                 |
| `"Х"`                     | Хуманитарни                             |
| `"_"`                     | Placeholder for any of the listed above |

- If everything with above configs is okay, simply execute the binary. You should see a detailed report in the standard output, something similar to mine:
```
No exact match found for course 'Увод в програмирането - практикум - спец. СИ'.
Closest match is 'Увод в програмирането - практикум-спец.СИ' with similarity 0.96484375.

No exact match found for course 'Обектно-ориентирано програмиране-практикум - сп. СИ'.
Closest match is 'Обектно-ориентирано програмиране - практикум - спец. СИ' with similarity 0.97127265.

[WARNING] Course 'Числени методи' couldn't be found, so I'm ignoring it.

No exact match found for course 'Функционално програмиране-практикум'.
Closest match is 'Функционално програмиране - практикум' with similarity 0.9775.


Your optimal arrangement is:

Практикум ->
  'Обектно-ориентирано програмиране-практикум - сп. СИ' (Практикум)
  'Функционално програмиране-практикум' (Практикум)

Математика ->
  'Състезателна математика II' (Математика)

ОКН ->
  'Функционално програмиране' (ОКН)

ЯКН ->
  'Съвременни Java технологии' (ЯКН)

Математика | Приложна математика ->
  'Алгебра 2' (Математика)
  'Случайни процеси' (Приложна математика)

ОКН | ЯКН ->
  'Дълбоко обучение с Тензорфлоу' (ЯКН)
  'Ламбда смятане и теория на доказателствата' (ОКН)

Информатика | Практикум | Математика | Приложна математика | ОКН | ЯКН | Статистика | Семинар | Хуманитарни | Други ->
  'Увод в програмирането - практикум - спец. СИ' (Практикум)


Requirements left:

  ОКН | ЯКН -> 2


You have no unmapped courses, i.e. no courses left that couldn't fit in a category requirement.
```

## How it works
In general, the problem every FMI student faces once at the beginning of each semester regarding the normative for elective courses consists of: 
- Checking the elective campaing for the upcoming semester and what courses are being offered
- Checking the summary of already taken / passed courses
- Opening the document with per-category requirements for the different elective categories
- And finally, mapping the taken courses to the category requirements (mostly intuitively optimal eyeballing) and working out a plan for the remaining categories in accordance with the current semester's available electives

As we all know, before successfully graduating with a Bachelor's degree from FMI, we must have fulfilled all the elective requirements for our chosen specialty. So the general plan for the electives is a year-long scheme that takes a lot of planing and attention.

This little scraping bot assists with that plan namely by automating the repetitive actions listed above and optimally calculating a distribution for the elective courses already taken per categories.

- First, it fetches and parses the student's grade report from SUSI
- Then it cross-references all the pages with elective offerings for past semesters from FMI's public site to extract the passed courses' metadata (currently only categories are used).
- Then it fetches the configuration with the per-category requirements.
- And finally, it calculates the optimal distribution among all the possible viable distributions. A distribution meaning certain courses being mapped to a certain requirement.

## Contributions

Notice the file `elective_archive_urls.json`? Well, that's where we store the links to FMI's pages with past semesters' elective campaigns. Since it's a shared resource for all users, a single update in this repo should serve well enough. If we've forgotten to update it, you might want to remind us by creating a PR with the update! 😊
