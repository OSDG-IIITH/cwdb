use chrono::Datelike;
use regex::Regex;
use sqlx::PgConnection;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::courses::CourseRegistry;

static RE_TOKEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\dk\d{2}|[a-z]+|\d+").unwrap());
static RE_CAMEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([a-z])([A-Z])").unwrap());
static RE_BATCH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:\dk|ug20)(\d{2})").unwrap());
static RE_YEARSEM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([1-5])-([12])$").unwrap());

const QUIZ: &[&str] = &["quiz", "quizzes", "quizes", "clquiz", "classquiz", "tutquiz", "tutq", "surprise", "classtest", "sliptest"];
const MID: &[&str] = &["mid", "mids", "midsem", "midsems", "midterm", "midterms"];
const END: &[&str] = &["end", "endsem", "endsems", "endterm", "endterms"];
const FINAL: &[&str] = &["final", "finals"];
const LABEXAM: &[&str] = &["labexam", "labexams", "labquiz"];
const EXAM: &[&str] = &["exam", "exams", "paper", "papers", "pyq", "pyqs", "qp", "qps"];
const ASSIGNMENT: &[&str] = &["assignment", "assignments", "assign", "assgn", "hw", "hws", "homework", "homeworks", "pset", "psets"];
const LAB: &[&str] = &["lab", "labs", "labassignments", "practical", "practicals", "experiment", "experiments"];
const PROJECT: &[&str] = &["project", "projects"];
const REFERENCE: &[&str] = &["textbook", "textbooks", "book", "books", "reading", "readings", "reference", "references", "manual", "manuals"];
const NOTES: &[&str] = &["notes", "note", "lecturenotes", "handout", "handouts"];
const LEC: &[&str] = &["lec", "lecs", "lecture", "lectures", "slides", "slide", "ppt", "ppts"];
const TUT: &[&str] = &["tut", "tuts", "tutorial", "tutorials"];
const OTHER: &[&str] = &["mock", "mocks", "labexammocks", "practiceset", "template", "templates", "submission", "submissions", "mysubmissions", "syllabus"];
const PRACTICE: &[&str] = &["sheet", "sheets", "problem", "problems", "set", "sets", "questions", "paper", "papers"];
const KEY: &[&str] = &["solution", "solutions", "soln", "solns", "sol", "sols", "answer", "answers"];
const PROMPT: &[&str] = &["statement", "problem", "problems", "question", "questions"];
const DOCS: &[&str] = &["pdf", "png", "jpg", "jpeg", "doc", "docx"];
const TEXTS: &[&str] = &["pdf", "png", "jpg", "jpeg", "doc", "docx", "md", "txt", "ppt", "pptx"];
const BEFORE: &[&str] = &["before", "after", "pre", "post", "till", "upto"];
const GROUP: &[&str] = &["past", "year", "years", "old", "older", "prev", "previous"];
const CONTAINER: &[&str] = &["assignments", "labs", "projects"];
const WEAK: &[&str] = &["references", "notes", "slides"];

/// rule-based metadata for one file
#[derive(Debug, Default)]
pub struct Guess {
    pub kind: Option<&'static str>,
    pub examtype: Option<&'static str>,
    pub slidetype: Option<&'static str>,
    pub n: Option<i32>,
    pub key: bool,
    pub year: Option<i32>,
    pub season: Option<&'static str>,
    pub confidence: Option<f32>,
}

#[derive(Clone, Copy)]
struct Hit {
    kind: &'static str,
    sub: Option<&'static str>,
    n: Option<i32>,
}

/// lowercase with camel case split apart
fn lower(s: &str) -> String {
    RE_CAMEL.replace_all(s, "$1 $2").to_lowercase()
}

fn tokens(s: &str) -> Vec<&str> {
    RE_TOKEN.find_iter(s).map(|m| m.as_str()).collect()
}

fn digits(w: &str) -> bool {
    !w.is_empty() && w.bytes().all(|b| b.is_ascii_digit())
}

/// tokens like 2k24
fn kyear(w: &str) -> bool {
    w.len() == 4 && w.as_bytes()[1] == b'k'
}

fn find(t: &[&str], words: &[&str]) -> Option<usize> {
    t.iter().position(|w| words.contains(w))
}

fn pair(t: &[&str], a: &[&str], b: &[&str]) -> bool {
    t.windows(2).any(|w| a.contains(&w[0]) && b.contains(&w[1]))
}

/// number right after token i, up to max
fn num(t: &[&str], i: usize, max: i32) -> Option<i32> {
    t.get(i + 1)
        .filter(|w| digits(w))
        .and_then(|w| w.parse().ok())
        .filter(|n| (1..=max).contains(n))
}

/// lowercase file extension
pub fn ext(path: &str) -> Option<String> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let (stem, ext) = name.rsplit_once('.')?;
    (!stem.is_empty() && !ext.is_empty()).then(|| ext.to_lowercase())
}

/// kind hinted by one path segment, ext is set for the filename
fn segment(t: &[&str], ext: Option<&str>) -> Option<Hit> {
    let hit = |kind, sub, n| Some(Hit { kind, sub, n });
    let roll = t.iter().any(|w| w.len() == 10 && w.starts_with("20") && digits(w));
    if find(t, OTHER).is_some()
        || pair(t, &["practice"], PRACTICE)
        || pair(t, &["course"], &["plan"])
        || pair(t, &["lab"], &["report", "reports"])
        || (ext.is_some() && roll)
    {
        return hit("other", None, None);
    }
    if let Some(i) = find(t, LABEXAM) {
        return hit("exams", Some("lab"), num(t, i, 9));
    }
    if find(t, &["lab", "labs"]).is_some()
        && let Some(i) = find(t, &["exam", "exams", "quiz"])
    {
        return hit("exams", Some("lab"), num(t, i, 9));
    }
    if let Some(b) = find(t, BEFORE)
        && t[b..].iter().any(|w| [QUIZ, MID, END].iter().any(|e| e.contains(w)))
    {
        return None;
    }
    if pair(t, &["class", "slip"], &["test", "tests"]) {
        return hit("exams", Some("quiz"), find(t, &["test", "tests"]).and_then(|i| num(t, i, 9)));
    }
    if pair(t, &["lecture", "lectures", "lec"], &["notes"]) {
        return hit("notes", None, None);
    }
    if let Some(i) = find(t, ASSIGNMENT) {
        return hit("assignments", None, num(t, i, 99));
    }
    for (words, sub) in [(QUIZ, "quiz"), (MID, "mid"), (END, "end")] {
        if let Some(i) = find(t, words) {
            let i = if matches!(t.get(i + 1), Some(&("sem" | "semester" | "term" | "exam"))) { i + 1 } else { i };
            return hit("exams", Some(sub), num(t, i, 9));
        }
    }
    if ext.is_some_and(|e| DOCS.contains(&e))
        && let Some(n) = find(t, &["q"]).and_then(|i| num(t, i, 9))
    {
        return hit("exams", Some("quiz"), Some(n));
    }
    if find(t, EXAM).is_some() {
        return hit("exams", find(t, FINAL).map(|_| "end"), None);
    }
    if pair(t, &["problem"], &["set", "sets"]) {
        return hit("assignments", None, None);
    }
    if let Some(n) = find(t, &["a"]).and_then(|i| num(t, i, 20)) {
        return hit("assignments", None, Some(n));
    }
    if let Some(i) = find(t, LAB) {
        return hit("labs", None, num(t, i, 99));
    }
    if let Some(i) = find(t, PROJECT) {
        return hit("projects", None, num(t, i, 99));
    }
    if find(t, REFERENCE).is_some() {
        return hit("references", None, None);
    }
    if find(t, NOTES).is_some() {
        return hit("notes", None, None);
    }
    if let Some(i) = find(t, TUT) {
        return hit("slides", Some("tut"), num(t, i, 99));
    }
    if let Some(i) = find(t, LEC) {
        return hit("slides", Some("lec"), num(t, i, 99));
    }
    if let Some(i) = find(t, FINAL) {
        return hit("exams", Some("end"), num(t, i, 9));
    }
    None
}

fn hitof(seg: &str) -> Option<Hit> {
    let lower = lower(seg);
    segment(&tokens(&lower), None)
}

/// two- or four-digit year within range
fn yearof(w: &str, max: i32) -> Option<i32> {
    let y: i32 = w.parse().ok().filter(|_| digits(w))?;
    let y = if w.len() == 2 { 2000 + y } else if w.len() == 4 { y } else { return None };
    (2015..=max).contains(&y).then_some(y)
}

/// batch year from a repo name like 2k24 or ug2023
fn batch(repo: &str) -> Option<i32> {
    let lower = repo.to_lowercase();
    let caps = RE_BATCH.captures(&lower)?;
    caps[1].parse::<i32>().ok().map(|y| 2000 + y)
}

/// classify a file by its full path
pub fn classify(path: &str, repo: &str, term: Option<&str>) -> Guess {
    let lower = lower(path);
    let mut segs: Vec<&str> = lower.split('/').collect();
    let name = segs.pop().unwrap_or_default();
    let ext = ext(name);
    let stem = match &ext {
        Some(e) => &name[..name.len() - e.len() - 1],
        None => name,
    };
    let raw: Vec<&str> = std::iter::once(stem).chain(segs.iter().rev().copied()).collect();
    let toks: Vec<Vec<&str>> = raw.iter().map(|s| tokens(s)).collect();
    let hits: Vec<Option<Hit>> = toks
        .iter()
        .enumerate()
        .map(|(i, t)| segment(t, (i == 0).then(|| ext.as_deref().unwrap_or(""))))
        .collect();

    let mut g = Guess::default();
    let mut chosen = hits.iter().enumerate().find_map(|(i, h)| h.map(|h| (i, h)));
    if let Some((i, h)) = chosen
        && (WEAK.contains(&h.kind) || (h.kind == "exams" && h.sub.is_none()))
        && let Some(c) = hits[i + 1..].iter().flatten().find(|c| CONTAINER.contains(&c.kind))
    {
        chosen = Some((i, *c));
    }

    match chosen {
        Some((i, h)) => {
            let same = || hits.iter().flatten().filter(move |c| c.kind == h.kind);
            let sub = h.sub.or_else(|| same().find_map(|c| c.sub));
            g.kind = Some(h.kind);
            g.n = h.n.or_else(|| same().find_map(|c| c.n));
            g.confidence = Some(if i == 0 { 0.9 } else { 0.7 });
            match h.kind {
                "exams" => g.examtype = sub,
                "slides" => g.slidetype = sub.or(Some("lec")),
                _ => {}
            }
        }
        None if matches!(ext.as_deref(), Some("ppt" | "pptx" | "odp")) => {
            g.kind = Some("slides");
            g.slidetype = Some("lec");
            g.confidence = Some(0.5);
        }
        None => {}
    }

    g.key = toks.iter().take(2).any(|t| {
        find(t, KEY).is_some()
            || t.last().is_some_and(|w| matches!(*w, "key" | "keys"))
            || pair(t, &["answer", "answers", "ans"], &["key", "keys"])
    });

    if g.kind != Some("references") {
        let text = ext.as_deref().is_some_and(|e| TEXTS.contains(&e));
        let (year, season) = when(&raw, &toks, repo, term, g.kind == Some("exams"), text);
        g.year = year;
        g.season = season;
    }
    g
}

/// year and season from the path, the repo batch and the course term
fn when(
    raw: &[&str],
    toks: &[Vec<&str>],
    repo: &str,
    term: Option<&str>,
    exam: bool,
    text: bool,
) -> (Option<i32>, Option<&'static str>) {
    let max = chrono::Utc::now().year() + 1;
    let (mut year, mut season, mut sem) = (None, None, None);

    for (i, t) in toks.iter().enumerate().skip(usize::from(!text)) {
        let mut found = None;
        for (j, w) in t.iter().enumerate() {
            let next = t.get(j + 1).copied();
            let mut mark = |s: &'static str| {
                season.get_or_insert(s);
                if let Some(y) = next.and_then(|n| yearof(n, max)) {
                    found.get_or_insert(y);
                }
            };
            match *w {
                "monsoon" | "fall" => mark("M"),
                "spring" => mark("S"),
                "m" | "s" if i == 0 && next.is_some_and(|n| n.len() == 2 && yearof(n, max).is_some()) => {
                    mark(if *w == "m" { "M" } else { "S" })
                }
                "sem" | "semester" if j == 0 || !(MID.contains(&t[j - 1]) || END.contains(&t[j - 1])) => {
                    if let Some(k) = num(t, j, 8) {
                        sem.get_or_insert(k);
                    }
                }
                _ if w.len() == 4 && digits(w) => {
                    if let Some(y) = yearof(w, max) {
                        found.get_or_insert(y);
                        match next {
                            Some("m") => season = season.or(Some("M")),
                            Some("s") => season = season.or(Some("S")),
                            _ => {}
                        }
                    }
                }
                _ if kyear(w) => {
                    found = found.or_else(|| yearof(&w[2..], max));
                }
                _ => {}
            }
        }
        if found.is_none() && i == 0 && exam {
            found = t.iter().rev().find(|w| digits(w)).filter(|w| w.len() == 2).and_then(|w| yearof(w, max));
        }
        if let Some(c) = RE_YEARSEM.captures(raw[i]) {
            let (y, s): (i32, i32) = (c[1].parse().unwrap_or(0), c[2].parse().unwrap_or(0));
            sem.get_or_insert((y - 1) * 2 + s);
        }
        year = year.or(found);
    }

    let season = season
        .or_else(|| sem.map(|k| if k % 2 == 1 { "M" } else { "S" }))
        .or(match term {
            Some("monsoon") => Some("M"),
            Some("spring") => Some("S"),
            _ => None,
        });
    let year = year.or_else(|| Some(batch(repo)? + sem? / 2));
    (year, season)
}

/// unit folder a file sits in, if any
fn unitof(path: &str) -> Option<String> {
    let segs: Vec<&str> = path.split('/').collect();
    let folders = &segs[..segs.len() - 1];
    let generic = |s: &str| {
        let lower = lower(s);
        let t = tokens(&lower);
        !t.is_empty()
            && t.iter().all(|w| {
                [ASSIGNMENT, LAB, PROJECT, GROUP].iter().any(|g| g.contains(w))
                    || (digits(w) && w.len() == 4)
                    || kyear(w)
            })
    };
    let container = |s: &str| hitof(s).is_some_and(|h| CONTAINER.contains(&h.kind));
    let i = folders.iter().position(|s| container(s))?;
    let j = (i..folders.len()).find(|&j| !generic(folders[j]))?;
    (hitof(folders[j]).is_none() || container(folders[j])).then(|| folders[..=j].join("/"))
}

/// primary file of a unit: readme at its root, then a statement, then the first pdf
fn primary<'a>(unit: &str, files: &[&'a str]) -> Option<&'a str> {
    let stem = |f: &str| {
        let name = lower(f.rsplit('/').next().unwrap_or(f));
        name.split('.').next().unwrap_or_default().to_string()
    };
    let mut files = files.to_vec();
    files.sort_by_key(|f| (f.matches('/').count(), f.to_lowercase()));
    files
        .iter()
        .find(|f| stem(f).replace(' ', "") == "readme" && f.rsplit_once('/').is_some_and(|(dir, _)| dir == unit))
        .or_else(|| {
            files.iter().find(|f| {
                ext(f).is_some_and(|e| TEXTS.contains(&e.as_str())) && find(&tokens(&stem(f)), PROMPT).is_some()
            })
        })
        .or_else(|| files.iter().find(|f| ext(f).as_deref() == Some("pdf")))
        .copied()
}

/// primary files of folder units mapped to their unit folder
pub fn units<'a>(paths: impl IntoIterator<Item = &'a str>) -> HashMap<String, String> {
    let mut groups: HashMap<String, Vec<&str>> = HashMap::new();
    for p in paths {
        if let Some(u) = unitof(p) {
            groups.entry(u).or_default().push(p);
        }
    }
    groups
        .into_iter()
        .filter(|(_, files)| files.iter().any(|f| ext(f).as_deref() != Some("pdf")))
        .filter_map(|(u, files)| primary(&u, &files).map(|p| (p.to_string(), u)))
        .collect()
}

/// find or create an offering
pub async fn offering(db: &mut PgConnection, course: &str, season: &str, year: i32) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        "WITH found AS (SELECT id FROM offerings WHERE course = $1 AND season = $2 AND year = $3), \
         made AS (INSERT INTO offerings (course, season, year) SELECT $1, $2, $3 \
                  WHERE NOT EXISTS (SELECT 1 FROM found) ON CONFLICT DO NOTHING RETURNING id) \
         SELECT id FROM found UNION ALL SELECT id FROM made",
    )
    .bind(course)
    .bind(season)
    .bind(year as i16)
    .fetch_one(db)
    .await
}

/// rerun rules on a source's rows that are not locked, returns rows changed
pub async fn apply(
    db: &mut PgConnection,
    courses: &CourseRegistry,
    sourceid: i32,
    repo: &str,
    aliases: &HashMap<String, String>,
) -> Result<u64, sqlx::Error> {
    let rows: Vec<(i32, String, Option<String>)> =
        sqlx::query_as("SELECT id, file_path, classifiedby FROM resources WHERE source_id = $1")
            .bind(sourceid)
            .fetch_all(&mut *db)
            .await?;
    let units = units(rows.iter().map(|r| r.1.as_str()));
    let mut offerings: HashMap<(String, &str, i32), i32> = HashMap::new();
    let mut changed = 0;

    for (id, path, by) in &rows {
        if by.as_deref().is_some_and(|b| b != "rule") {
            continue;
        }
        let course = courses.resolve(path, repo, aliases);
        let g = classify(path, repo, course.as_deref().and_then(|c| courses.term(c)));
        let offering = match (&course, g.season, g.year) {
            (Some(c), Some(s), Some(y)) => match offerings.get(&(c.clone(), s, y)) {
                Some(&o) => Some(o),
                None => {
                    let o = offering(db, c, s, y).await?;
                    offerings.insert((c.clone(), s, y), o);
                    Some(o)
                }
            },
            _ => None,
        };

        changed += sqlx::query(
            "UPDATE resources SET course = $2, offering = $3, kind = $4, examtype = $5, slidetype = $6, \
             n = $7, key = $8, unit = $9, confidence = $10, classifiedby = 'rule', classifiedat = NOW(), updated_at = NOW() \
             WHERE id = $1 AND (course, offering, kind, examtype, slidetype, n, key, unit, confidence, classifiedby) \
             IS DISTINCT FROM ($2, $3, $4, $5, $6, $7, $8, $9, $10, 'rule')",
        )
        .bind(id)
        .bind(&course)
        .bind(offering)
        .bind(g.kind)
        .bind(g.examtype)
        .bind(g.slidetype)
        .bind(g.n)
        .bind(g.key)
        .bind(units.get(path))
        .bind(g.confidence)
        .execute(&mut *db)
        .await?
        .rows_affected();
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// path, repo, course term, kind, examtype or slidetype, n, key, year, season
    type Case = (&'static str, &'static str, Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<i32>, bool, Option<i32>, Option<&'static str>);

    const CASES: &[Case] = &[
        ("Electives/BouquetSys/DS-Distributed Systems/DS_quiz1_2024_M_key.pdf", "iiitprevpapers", Some("both"), Some("exams"), Some("quiz"), Some(1), true, Some(2024), Some("M")),
        ("Electives/BouquetSys/DBS-Database Systems/DBS_mid2_2017_s.pdf", "iiitprevpapers", None, Some("exams"), Some("mid"), Some(2), false, Some(2017), Some("S")),
        ("Year1/CS/RA-Real Analysis/RA_end2_2022_M.pdf", "iiitprevpapers", None, Some("exams"), Some("end"), Some(2), false, Some(2022), Some("M")),
        ("Electives/ACN-Advanced Computer Networks/ACN_clquiz3_2019.pdf", "iiitprevpapers", Some("monsoon"), Some("exams"), Some("quiz"), Some(3), false, Some(2019), Some("M")),
        ("Electives/CV-Computer Vision/CV_quiz1_2024_S_key.pdf", "iiitprevpapers", None, Some("exams"), Some("quiz"), Some(1), true, Some(2024), Some("S")),
        ("Year2/CS/PRS-Probability and Statistics/PRS_surprise_2024_M_key.pdf", "iiitprevpapers", None, Some("exams"), Some("quiz"), None, true, Some(2024), Some("M")),
        ("Electives/BouquetSys/DS-Distributed Systems/DS_SlipTest1_2026_S.pdf", "iiitprevpapers", None, Some("exams"), Some("quiz"), Some(1), false, Some(2026), Some("S")),
        ("Year2/CS/SCI1-Science-I/SCI1_papers_2018.pdf", "iiitprevpapers", Some("monsoon"), Some("exams"), None, None, false, Some(2018), Some("M")),
        ("sem1/dsm/question_papers/dsm_end_23.pdf", "iiith-ug2023-resources", Some("monsoon"), Some("exams"), Some("end"), None, false, Some(2023), Some("M")),
        ("sem2/la/question_papers/la_tutq2_24.pdf", "iiith-ug2023-resources", Some("spring"), Some("exams"), Some("quiz"), Some(2), false, Some(2024), Some("S")),
        ("sem2/aec/question_papers/aec_q2_24.png", "iiith-ug2023-resources", None, Some("exams"), Some("quiz"), Some(2), false, Some(2024), Some("S")),
        ("sem1/cpro/practice_problems/week6.md", "iiith-ug2023-resources", None, Some("other"), None, None, false, Some(2023), Some("M")),
        ("Semester-3/CXX/AAD/Exams/2K25/Quiz2 2K25.pdf", "2K25-Resources", Some("monsoon"), Some("exams"), Some("quiz"), Some(2), false, Some(2025), Some("M")),
        ("Semester-2/Common/LA/Exams/2K26/LA S26_ MidSem Rubric .pdf", "2K25-Resources", None, Some("exams"), Some("mid"), None, false, Some(2026), Some("S")),
        ("Semester-1/CND/CIS-1/Material/2K18/MidSem1/formic_monomer.com", "2K25-Resources", None, Some("exams"), Some("mid"), Some(1), false, Some(2018), Some("M")),
        ("Semester-1/CGD/2k24 exams/Endsems/Endsems Spatial Thinking and Practice.pdf", "2K25-Resources", None, Some("exams"), Some("end"), None, false, Some(2024), Some("M")),
        ("Semester-1/Common/DSM/Practicals/2k25/Lab Reports/DSM_lap_report_2 (2).pdf", "2K25-Resources", None, Some("other"), None, None, false, Some(2025), Some("M")),
        ("Semester-1/Common/DSM/lecture_slides/ubaidulla/Lec25_Memory architecture - 2.pdf", "2K25-Resources", None, Some("slides"), Some("lec"), Some(25), false, Some(2025), Some("M")),
        ("Semester-2/Common/LA/Notes/2K24/H2-Notes/Topics covered (H2)_before_tut_quiz.pdf", "2K25-Resources", None, Some("notes"), None, None, false, Some(2024), Some("S")),
        ("Semester-2/CSX/CSO/Assignment/2023113019_assign2_mid/2023113019/q1/q1.c", "2K25-Resources", None, Some("assignments"), None, Some(2), false, Some(2026), Some("S")),
        ("Semester-2/Common/ISS/Material/Deploy a Flask App to Render.pdf", "2K25-Resources", None, None, None, None, false, Some(2026), Some("S")),
        ("papers/Semester 4/CS7.301 Machine, Data and Learning/CS7.301 Midsem Answer Key.pdf", "exams2k24", Some("spring"), Some("exams"), Some("mid"), None, true, Some(2026), Some("S")),
        ("Semester - 1 ( Monsoon'24 )/DSM/Lab Reports/2024113014_Group14_Lab5.pdf", "IIITH-CourseWork", None, Some("other"), None, None, false, Some(2024), Some("M")),
        ("electives/humanities/Theories and Practices of Nationalism/spring_26/tpn_quiz1.pdf", "iiith-pyqs", None, Some("exams"), Some("quiz"), Some(1), false, Some(2026), Some("S")),
        ("core/common/Digital Systems and Microcontrollers/monsoon_22/dsm_endsem_makeup_exam.pdf", "iiith-pyqs", None, Some("exams"), Some("end"), None, false, Some(2022), Some("M")),
        ("Year3/Data Analytics-1/DA_ENDSEMS.pdf", "Question_papers_IIITH", Some("monsoon"), Some("exams"), Some("end"), None, false, None, Some("M")),
        ("Year1/ISS_2024/ISS MOCK lab exam SECtion B.pdf", "Question_papers_IIITH", Some("spring"), Some("other"), None, None, false, Some(2024), Some("S")),
        ("Year1/ISS_2024/ISS_finalExam.pdf", "Question_papers_IIITH", Some("spring"), Some("exams"), Some("end"), None, false, Some(2024), Some("S")),
        ("sem1/ness/textbooks/Alan V. Oppenheim, Alan S. Willsky, with S. Hamid-Signals and Systems-Prentice Hall (1996).pdf", "iiith-ug2023-resources", None, Some("references"), None, None, false, None, None),
        ("sem-1/hsl/5 Election Analysis/election/election/middlewares.py", "my-iiith", None, None, None, None, false, None, Some("M")),
        ("sem-1/hsl/5 Election Analysis/election/election/spiders/wb2016data.csv", "my-iiith", None, None, None, None, false, None, Some("M")),
        ("SEM-4/ELECTIVES/CG-ELECTIVE/Rendering Algorithms.pdf", "2k24", None, None, None, None, false, Some(2026), Some("S")),
        ("SEM-4/SC-2/before_mid/6/Class_6_ODE_LVmodel.pdf", "2k24", None, None, None, None, false, Some(2026), Some("S")),
        ("SEM-4/IHS/Economics/slides/Lecture 2 Key Economic Measures.pdf", "2k24", None, Some("slides"), Some("lec"), Some(2), false, Some(2026), Some("S")),
        ("SEM-4/MDL/notes/mdl lec 10.pdf", "2k24", None, Some("slides"), Some("lec"), Some(10), false, Some(2026), Some("S")),
        ("SEM-2/ISS/labs/lab2-bash/lab2_questions/solutions/lab2_expected_outputs/awk-output-2.txt", "2k24", None, Some("labs"), None, Some(2), false, Some(2025), Some("S")),
        ("SEM-2/ISS/labs/ISS_Project_Spring_2023/src/app.py", "2k24", None, Some("projects"), None, None, false, Some(2023), Some("S")),
        ("sem-2/la/assignments/solutions/Assignment2_Solutions.pdf", "my-iiith", None, Some("assignments"), None, Some(2), true, None, Some("S")),
        ("sem-2/iss/Assignments/Assignment 3B/2021115008/Record-service.py", "my-iiith", None, Some("assignments"), None, Some(3), false, None, Some("S")),
        ("sem-6/smai/assignments/2/rollnumber_A2_Q3/ATnT/s31/1.pgm", "my-iiith", None, Some("assignments"), None, Some(2), false, None, Some("S")),
        ("sem1/cpro/assignment4/2023114011_cpro_a4/q3.c", "IIIT-Coursework", None, Some("assignments"), None, Some(4), false, None, Some("M")),
        ("resources/sem3/Pns/Tut-solutions/Tutorial 5 Solutions.pdf", "course-material-2k23", None, Some("slides"), Some("tut"), Some(5), true, Some(2024), Some("M")),
        ("resources/sem1/DSM/question_papers/answer_keys/End Semester Key/Key.pdf", "course-material-2k23", None, Some("exams"), Some("end"), None, true, Some(2023), Some("M")),
        ("resources/sem2/IoT Spring 24 Final Exam Key.pdf", "course-material-2k23", None, Some("exams"), Some("end"), None, true, Some(2024), Some("S")),
        ("1-1/03 Digital Systems and Microcontrollers/Exam QPs/DSM Quiz 2 QP.pdf", "ECD-IIITHYd", None, Some("exams"), Some("quiz"), Some(2), false, None, Some("M")),
        ("1-2/01 Analog Electronic Circuits/Assignments/A3/A3 QP.pdf", "ECD-IIITHYd", None, Some("assignments"), None, Some(3), false, None, Some("S")),
        ("2-1/02 Signal Processing/Assignments/A4 Submission.pdf", "ECD-IIITHYd", None, Some("other"), None, None, false, None, Some("M")),
        ("2-1/AAD Nov 21.pdf", "Famous-Last-Words", None, None, None, None, false, None, Some("M")),
        ("2-2/IHS (Sociology) Notes .pdf", "Famous-Last-Words", None, Some("notes"), None, None, false, None, Some("S")),
        ("pre-quiz1/lec3.pdf", "MDL-lecs", None, Some("slides"), Some("lec"), Some(3), false, None, None),
    ];

    #[test]
    fn rules() {
        for &(path, repo, term, kind, sub, n, key, year, season) in CASES {
            let g = classify(path, repo, term);
            let got = (g.kind, g.examtype.or(g.slidetype), g.n, g.key, g.year, g.season);
            assert_eq!(got, (kind, sub, n, key, year, season), "{path}");
        }
    }

    #[test]
    fn folder_units() {
        let paths = [
            "sem-6/smai/assignments/1/rollnumber_A1_Q1/Q1Data.csv",
            "sem-6/smai/assignments/1/rollnumber_A1_Q1/Q1ProblemStatement.pdf",
            "sem-6/smai/assignments/1/rollnumber_A1_Q2/Q2.ipynb",
            "sem-6/smai/assignments/2/rollnumber_A2_Q1/Q1.ipynb",
            "sem-6/smai/assignments/2/rollnumber_A2_Q3/ATnT/README",
            "sem-6/smai/assignments/3/Assignment_3.pdf",
            "sem-6/smai/assignments/3/rollnumber_A3_Q3/RNN_Sentiment_Classification.ipynb",
            "sem-6/smai/assignments/4.pdf",
            "sem-2/iss/Assignments/Assignment 1/q1.sh",
            "sem-2/iss/Assignments/Assignment 1/README.txt",
            "SEM-2/ISS/labs/lab2-bash/Lab 2 ISS.pdf",
            "SEM-2/ISS/labs/lab2-bash/lab2_questions/data.csv",
            "SEM-2/ISS/labs/lab2-bash/lab2_questions/lab_activity.pdf",
            "Semester-2/Common/DSA/Assignments/2022/question_3_data.zip",
            "Semester-2/Common/DSA/Assignments/2022/DSA_Assignment_3.pdf",
            "sem1/dsm/exams/2024/quiz1.pdf",
            "sem1/dsm/exams/2024/mid.pdf",
        ];
        let mut got: Vec<(String, String)> = units(paths).into_iter().collect();
        got.sort();
        let want = [
            ("SEM-2/ISS/labs/lab2-bash/Lab 2 ISS.pdf", "SEM-2/ISS/labs/lab2-bash"),
            ("sem-2/iss/Assignments/Assignment 1/README.txt", "sem-2/iss/Assignments/Assignment 1"),
            ("sem-6/smai/assignments/1/rollnumber_A1_Q1/Q1ProblemStatement.pdf", "sem-6/smai/assignments/1"),
            ("sem-6/smai/assignments/3/Assignment_3.pdf", "sem-6/smai/assignments/3"),
        ];
        assert_eq!(got, want.map(|(a, b)| (a.to_string(), b.to_string())));
    }

    /// prints rule coverage over an index.json, set INDEX to override the path
    #[test]
    #[ignore]
    fn coverage() {
        let path = std::env::var("INDEX").unwrap_or("../frontend/static/data/index.json".into());
        let index: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let resources = index["resources"].as_array().unwrap();
        let mut bysource: HashMap<&str, Vec<&serde_json::Value>> = HashMap::new();
        for r in resources {
            bysource.entry(r["source_id"].as_str().unwrap()).or_default().push(r);
        }

        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut bump = |k: String, n: usize| *counts.entry(k).or_default() += n;
        for (source, rows) in &bysource {
            let repo = index["sources"][source]["repo"].as_str().unwrap_or_default();
            let units = units(rows.iter().map(|r| r["path"].as_str().unwrap()));
            bump("units".into(), units.len());
            for r in rows {
                let course = r["course_id"].as_str();
                let term = course.and_then(|c| index["courses"][c]["semester"].as_str());
                let g = classify(r["path"].as_str().unwrap(), repo, term);
                bump(format!("kind {}", g.kind.unwrap_or("none")), 1);
                if let Some(e) = g.examtype.or((g.kind == Some("exams")).then_some("none")) {
                    bump(format!("examtype {e}"), 1);
                }
                if let Some(s) = g.slidetype {
                    bump(format!("slidetype {s}"), 1);
                }
                bump("key".into(), usize::from(g.key));
                bump("course".into(), usize::from(course.is_some()));
                let dated = course.is_some() && g.year.is_some() && g.season.is_some();
                bump(format!("offering {}", if dated { "yes" } else { "no" }), 1);
            }
        }

        let mut lines: Vec<_> = counts.into_iter().collect();
        lines.sort();
        println!("{} files", resources.len());
        for (k, v) in lines {
            let pct = 100.0 * v as f64 / resources.len() as f64;
            println!("{k:<24} {v:>6} {pct:>5.1}%");
        }
    }
}
