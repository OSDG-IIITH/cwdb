use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::Path;

const KINDS: &[&str] = &["exams", "assignments", "labs", "slides", "notes", "references", "projects", "other"];
const EXAMTYPES: &[&str] = &["quiz", "mid", "end", "lab"];
const SLIDETYPES: &[&str] = &["lec", "tut"];
const BATCH: usize = 200;
/// rows the rules left with an empty field
pub const GAPS: &str = "AND (r.kind IS NULL OR r.course IS NULL OR (r.kind = 'exams' AND r.examtype IS NULL) \
    OR (r.kind = 'slides' AND r.slidetype IS NULL) OR (r.key AND r.keyfor IS NULL))";
const ROWS: &str = "SELECT r.id, r.source_id, s.repo, r.file_path, r.course, r.kind, r.examtype, r.slidetype, r.n, r.key, \
    r.keyfor, r.offering, o.season::text AS season, o.year::int AS year \
    FROM resources r JOIN sources s ON s.id = r.source_id LEFT JOIN offerings o ON o.id = r.offering";

/// one row of a model answer
#[derive(Deserialize, Debug)]
pub struct Out {
    pub id: i32,
    pub course: Option<String>,
    pub kind: Option<String>,
    pub examtype: Option<String>,
    pub slidetype: Option<String>,
    pub n: Option<i32>,
    #[serde(default)]
    pub key: bool,
    pub keyfor: Option<String>,
    pub year: Option<i32>,
    pub season: Option<String>,
    pub confidence: f32,
}

#[derive(Serialize)]
struct Have {
    course: Option<String>,
    kind: Option<String>,
    examtype: Option<String>,
    slidetype: Option<String>,
    n: Option<i32>,
    key: Option<bool>,
    year: Option<i32>,
    season: Option<String>,
}

#[derive(Serialize)]
struct Item {
    id: i32,
    source: String,
    path: String,
    have: Have,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    peers: Vec<String>,
}

/// file name without extension, key words or punctuation
fn stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path).to_lowercase();
    let name = name.rsplit_once('.').map_or(name.as_str(), |n| n.0).to_string();
    let words = ["key", "keys", "solution", "solutions", "soln", "sol", "answer", "answers"];
    name.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && !words.contains(w))
        .collect()
}

/// shared leading characters of two stems
fn overlap(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

/// quotes a string for sql
fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn opt<T: ToString>(v: Option<T>, text: bool) -> String {
    match v {
        Some(v) if text => quote(&v.to_string()),
        Some(v) => v.to_string(),
        None => "NULL".into(),
    }
}

/// writes rows with gaps (or every unlocked row with `full`) as batches of json under `dir/in`
async fn export(db: &PgPool, dir: &Path, repos: &[String], full: bool) -> Result<(), Box<dyn std::error::Error>> {
    let gaps = if full { "" } else { GAPS };
    let rows = sqlx::query(&format!(
        "{ROWS} WHERE NOT r.hidden AND r.classifiedby IS DISTINCT FROM 'manual' AND r.classifiedby IS DISTINCT FROM 'llm' {gaps} \
         AND (cardinality($1::text[]) = 0 OR s.repo = ANY($1) OR s.owner || '/' || s.repo = ANY($1)) ORDER BY s.repo, r.file_path"
    ))
    .bind(repos)
    .fetch_all(db)
    .await?;

    let courses: Vec<(String, String, Vec<String>, Option<String>)> =
        sqlx::query_as("SELECT code, name, shortnames, semester FROM courses ORDER BY code").fetch_all(db).await?;
    let courses: Vec<_> = courses.into_iter().map(|c| serde_json::json!([c.0, c.1, c.2, c.3])).collect();

    let sources: Vec<i32> = rows.iter().map(|r| r.get("source_id")).collect();
    let mut folders: HashMap<(i32, String), Vec<String>> = HashMap::new();
    for (source, path) in sqlx::query_as::<_, (i32, String)>(
        "SELECT source_id, file_path FROM resources WHERE source_id = ANY($1) AND NOT key",
    )
    .bind(&sources)
    .fetch_all(db)
    .await?
    {
        let dir = path.rsplit_once('/').map_or("", |d| d.0).to_string();
        folders.entry((source, dir)).or_default().push(path);
    }

    let items: Vec<Item> = rows
        .iter()
        .map(|r| {
            let path: String = r.get("file_path");
            let wantspeers = !full && r.get::<bool, _>("key") && r.get::<Option<i32>, _>("keyfor").is_none();
            let dir = path.rsplit_once('/').map_or("", |d| d.0).to_string();
            let peers = folders.get(&(r.get("source_id"), dir)).filter(|_| wantspeers);
            Item {
                id: r.get("id"),
                source: r.get("repo"),
                peers: peers
                    .map(|p| {
                        let mut p = p.clone();
                        p.sort_by_key(|q| std::cmp::Reverse(overlap(&stem(q), &stem(&path))));
                        p.truncate(30);
                        p
                    })
                    .unwrap_or_default(),
                path,
                have: Have {
                    course: r.get("course"),
                    kind: r.get::<Option<String>, _>("kind").filter(|_| !full),
                    examtype: r.get::<Option<String>, _>("examtype").filter(|_| !full),
                    slidetype: r.get::<Option<String>, _>("slidetype").filter(|_| !full),
                    n: r.get::<Option<i32>, _>("n").filter(|_| !full),
                    key: (!full).then(|| r.get("key")),
                    year: r.get("year"),
                    season: r.get("season"),
                },
            }
        })
        .collect();

    std::fs::create_dir_all(dir.join("in"))?;
    let mode = if full { "full" } else { "fill" };
    for (i, chunk) in items.chunks(BATCH).enumerate() {
        let file = std::fs::File::create(dir.join("in").join(format!("{}.json", i + 1)))?;
        serde_json::to_writer(file, &serde_json::json!({ "mode": mode, "courses": courses, "rows": chunk }))?;
    }
    println!("{} rows in {} batches, {mode} mode", items.len(), items.chunks(BATCH).len());
    Ok(())
}

/// validates answers and renders one update per row, never touching manual rows; without `full` only empty fields are filled
async fn render(db: &PgPool, outs: Vec<Out>, full: bool) -> Result<(String, usize, Vec<String>), Box<dyn std::error::Error>> {
    let codes: HashSet<String> = sqlx::query_scalar("SELECT code FROM courses").fetch_all(db).await?.into_iter().collect();
    let ids: Vec<i32> = outs.iter().map(|o| o.id).collect();
    let rows = sqlx::query(&format!("{ROWS} WHERE r.id = ANY($1) AND r.classifiedby IS DISTINCT FROM 'manual'"))
        .bind(&ids)
        .fetch_all(db)
        .await?;
    let current: HashMap<i32, sqlx::postgres::PgRow> = rows.into_iter().map(|r| (r.get("id"), r)).collect();

    let sources: Vec<i32> = current.values().map(|r| r.get("source_id")).collect();
    let paths: HashMap<(i32, String), i32> = sqlx::query_as::<_, (i32, String, i32)>(
        "SELECT source_id, file_path, id FROM resources WHERE source_id = ANY($1)",
    )
    .bind(&sources)
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|(s, p, i)| ((s, p), i))
    .collect();

    let terms: HashMap<String, Option<String>> = sqlx::query_as("SELECT code, semester FROM courses")
        .fetch_all(db)
        .await?
        .into_iter()
        .collect();

    let mut offerings = String::new();
    let mut made = HashSet::new();
    let mut updates = String::new();
    let mut errors = Vec::new();
    let mut count = 0;

    for o in outs {
        let Some(cur) = current.get(&o.id) else {
            errors.push(format!("{}: missing or manual", o.id));
            continue;
        };
        let bad = |why: &str| format!("{}: {why}", o.id);
        let check = |v: &Option<String>, set: &[&str]| v.as_deref().is_none_or(|v| set.contains(&v));
        if !(0.0..=1.0).contains(&o.confidence) {
            errors.push(bad("confidence"));
            continue;
        }
        if !check(&o.kind, KINDS) || !check(&o.examtype, EXAMTYPES) || !check(&o.slidetype, SLIDETYPES) {
            errors.push(bad("kind or type"));
            continue;
        }
        if o.course.as_ref().is_some_and(|c| !codes.contains(c)) {
            errors.push(bad("unknown course"));
            continue;
        }
        let source: i32 = cur.get("source_id");
        let was = |col: &str| cur.get::<Option<String>, _>(col);
        let pick = |old: Option<String>, new: Option<String>| if full { new } else { old.or(new) };

        let course = pick(was("course"), o.course.clone());
        let kind = pick(was("kind"), o.kind.clone());
        let examtype = pick(was("examtype"), o.examtype.clone()).filter(|_| kind.as_deref() == Some("exams"));
        let slidetype = pick(was("slidetype"), o.slidetype.clone()).filter(|_| kind.as_deref() == Some("slides"));
        let n = if full { o.n } else { cur.get::<Option<i32>, _>("n").or(o.n) };
        let key = if full { o.key } else { cur.get::<bool, _>("key") };
        let found = match &o.keyfor {
            Some(p) if key => match paths.get(&(source, p.clone())) {
                Some(&t) if t != o.id => Some(t),
                _ => {
                    errors.push(bad("keyfor path not found"));
                    None
                }
            },
            _ => None,
        };
        let keyfor = if full { found } else { cur.get::<Option<i32>, _>("keyfor").or(found) }.filter(|_| key);
        let keyforsql = match (&o.keyfor, found) {
            (Some(p), Some(_)) => format!("(SELECT id FROM resources WHERE source_id = {source} AND file_path = {})", quote(p)),
            _ => "NULL".into(),
        };

        let year = o.year.filter(|y| (2000..=2100).contains(y));
        let season = o.season.clone().filter(|s| s == "M" || s == "S").or_else(|| match course.as_ref().and_then(|c| terms.get(c)?.as_deref()) {
            Some("monsoon") => Some("M".into()),
            Some("spring") => Some("S".into()),
            _ => None,
        });
        let derived = match (&course, &season, year) {
            (Some(c), Some(s), Some(y)) => {
                if made.insert((c.clone(), s.clone(), y)) {
                    writeln!(offerings, "INSERT INTO offerings (course, season, year) SELECT {0}, {1}, {y} WHERE EXISTS (SELECT 1 FROM courses WHERE code = {0}) ON CONFLICT DO NOTHING;", quote(c), quote(s))?;
                }
                Some(format!("(SELECT id FROM offerings WHERE course = {} AND season = {} AND year = {y})", quote(c), quote(s)))
            }
            _ => None,
        };
        let hadoffering = cur.get::<Option<i32>, _>("offering").is_some();

        let mut sets = Vec::new();
        let mut add = |col: &str, value: String, changed: bool| {
            if full {
                sets.push(format!("{col} = {value}"));
            } else if changed {
                sets.push(format!("{col} = COALESCE({col}, {value})"));
            }
        };
        add("course", course.as_ref().map_or("NULL".into(), |c| format!("(SELECT code FROM courses WHERE code = {})", quote(c))), course != was("course"));
        add("kind", opt(kind.as_ref(), true), kind != was("kind"));
        add("examtype", opt(examtype.as_ref(), true), examtype != was("examtype"));
        add("slidetype", opt(slidetype.as_ref(), true), slidetype != was("slidetype"));
        add("n", opt(n, false), n != cur.get::<Option<i32>, _>("n"));
        add("key", key.to_string(), key != cur.get::<bool, _>("key"));
        add("keyfor", keyforsql, keyfor != cur.get::<Option<i32>, _>("keyfor"));
        if full || (!hadoffering && derived.is_some()) {
            add("offering", derived.unwrap_or_else(|| "NULL".into()), true);
        }
        if sets.is_empty() {
            continue;
        }

        let rulerow = was("kind").is_some();
        let disagree = full
            && rulerow
            && (was("kind") != kind
                || (was("examtype").is_some() && was("examtype") != examtype)
                || (was("course").is_some() && was("course") != course));
        let linked = keyfor != cur.get::<Option<i32>, _>("keyfor");
        let own = stem(&cur.get::<String, _>("file_path"));
        let odd = linked
            && keyfor.and_then(|t| paths.iter().find(|(_, i)| **i == t)).is_some_and(|((_, p), _)| {
                let other = stem(p);
                !other.contains(&own) && !own.contains(&other)
            });
        let review = o.confidence < 0.8 || disagree || odd;

        let same = if full {
            String::new()
        } else {
            format!(" AND kind IS NOT DISTINCT FROM {} AND course IS NOT DISTINCT FROM {}", opt(was("kind").as_ref(), true), opt(was("course").as_ref(), true))
        };
        writeln!(
            updates,
            "UPDATE resources SET {}, confidence = {}, needsreview = {review}, classifiedby = 'llm', classifiedat = NOW(), updated_at = NOW() \
             WHERE id = {} AND classifiedby IS DISTINCT FROM 'manual'{same};",
            sets.join(", "),
            o.confidence,
            o.id
        )?;
        count += 1;
    }
    let head = format!("-- {count} classified rows, keyed by resource id; manual rows are never touched\n");
    Ok((format!("{head}{offerings}{updates}"), count, errors))
}

/// reads every answer under `dir/out`, then writes it to the database or to a .sql file
async fn import(db: &PgPool, dir: &Path, sql: Option<&str>, full: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut outs: Vec<Out> = Vec::new();
    for entry in std::fs::read_dir(dir.join("out"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            outs.extend(serde_json::from_str::<Vec<Out>>(&std::fs::read_to_string(&path)?)?);
        }
    }
    let mut seen = HashSet::new();
    outs.retain(|o| seen.insert(o.id));
    let (script, count, errors) = render(db, outs, full).await?;
    for e in &errors {
        eprintln!("skipped {e}");
    }
    match sql {
        Some(file) => std::fs::write(file, script)?,
        None => {
            let mut tx = db.begin().await?;
            sqlx::raw_sql(&script).execute(&mut *tx).await?;
            tx.commit().await?;
        }
    }
    println!("{count} rows, {} skipped", errors.len());
    Ok(())
}

/// runs `classifyexport [--full] [repo...]` or `classifyimport [--full] [file.sql]` against DATABASE_URL, files live in ./classify
pub async fn run(cmd: &str, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    let dir = Path::new("classify");
    let full = args.iter().any(|a| a == "--full");
    let rest: Vec<String> = args.iter().filter(|a| *a != "--full").cloned().collect();
    match cmd {
        "classifyexport" => export(&db, dir, &rest, full).await,
        _ => import(&db, dir, rest.first().map(String::as_str), full).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems() {
        assert_eq!(stem("a/DS_quiz1_2024_M_key.pdf"), stem("a/DS_quiz1_2024_M.pdf"));
        assert_eq!(stem("x/Quiz1 Sol 2K25.pdf"), "quiz12k25");
        assert!(overlap(&stem("PRS_quiz2_2024_M.pdf"), &stem("PRS_quiz2_2024_M_key.pdf")) > overlap(&stem("MA6.101 Endsems.pdf"), &stem("PRS_quiz2_2024_M_key.pdf")));
    }
}
