use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::Path;

const KINDS: &[&str] = &["exams", "assignments", "labs", "slides", "notes", "references", "projects", "other"];
const EXAMTYPES: &[&str] = &["quiz", "mid", "end", "lab"];
const SLIDETYPES: &[&str] = &["lec", "tut"];
const BATCH: usize = 200;
const ROWS: &str = "SELECT r.id, r.source_id, s.repo, r.file_path, r.course, r.kind, r.examtype, r.slidetype, r.n, r.key, \
    o.season::text AS season, o.year::int AS year \
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
struct Hint {
    course: Option<String>,
    year: Option<i32>,
    season: Option<String>,
}

#[derive(Serialize)]
struct Item {
    id: i32,
    source: String,
    path: String,
    hint: Hint,
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

/// writes unclassified and rule-only rows as batches of json under `dir/in`
async fn export(db: &PgPool, dir: &Path, repos: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let rows = sqlx::query(&format!(
        "{ROWS} WHERE NOT r.hidden AND r.classifiedby IS DISTINCT FROM 'manual' AND r.classifiedby IS DISTINCT FROM 'llm' \
         AND (cardinality($1::text[]) = 0 OR s.repo = ANY($1)) ORDER BY s.repo, r.file_path"
    ))
    .bind(repos)
    .fetch_all(db)
    .await?;

    let courses: Vec<(String, String, Vec<String>, Option<String>)> =
        sqlx::query_as("SELECT code, name, shortnames, semester FROM courses ORDER BY code").fetch_all(db).await?;
    let courses: Vec<_> = courses.into_iter().map(|c| serde_json::json!([c.0, c.1, c.2, c.3])).collect();

    let items: Vec<Item> = rows
        .iter()
        .map(|r| Item {
            id: r.get("id"),
            source: r.get("repo"),
            path: r.get("file_path"),
            hint: Hint { course: r.get("course"), year: r.get("year"), season: r.get("season") },
        })
        .collect();

    std::fs::create_dir_all(dir.join("in"))?;
    for (i, chunk) in items.chunks(BATCH).enumerate() {
        let file = std::fs::File::create(dir.join("in").join(format!("{}.json", i + 1)))?;
        serde_json::to_writer(file, &serde_json::json!({ "courses": courses, "rows": chunk }))?;
    }
    println!("{} rows in {} batches", items.len(), items.chunks(BATCH).len());
    Ok(())
}

/// validates answers and renders one update per row, never touching manual rows
async fn render(db: &PgPool, outs: Vec<Out>) -> Result<(String, usize, Vec<String>), Box<dyn std::error::Error>> {
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
        if o.year.is_some_and(|y| !(2000..=2100).contains(&y)) || !check(&o.season, &["M", "S"]) {
            errors.push(bad("year or season"));
            continue;
        }

        let examtype = o.examtype.filter(|_| o.kind.as_deref() == Some("exams"));
        let slidetype = o.slidetype.filter(|_| o.kind.as_deref() == Some("slides"));
        let source: i32 = cur.get("source_id");
        let keyfor = match &o.keyfor {
            Some(p) if o.key => match paths.get(&(source, p.clone())) {
                Some(&t) if t != o.id => Some(t),
                _ => {
                    errors.push(bad("keyfor path not found"));
                    None
                }
            },
            _ => None,
        };

        let season = o.season.clone().or_else(|| match o.course.as_ref().and_then(|c| terms.get(c)?.as_deref()) {
            Some("monsoon") => Some("M".into()),
            Some("spring") => Some("S".into()),
            _ => None,
        });
        let offering = match (&o.course, &season, o.year) {
            (Some(c), Some(s), Some(y)) => {
                if made.insert((c.clone(), s.clone(), y)) {
                    writeln!(offerings, "INSERT INTO offerings (course, season, year) VALUES ({}, {}, {y}) ON CONFLICT DO NOTHING;", quote(c), quote(s))?;
                }
                format!("(SELECT id FROM offerings WHERE course = {} AND season = {} AND year = {y})", quote(c), quote(s))
            }
            _ => "NULL".into(),
        };

        let was = |col: &str| cur.get::<Option<String>, _>(col);
        let rulerow = was("kind").is_some();
        let disagree = rulerow
            && (was("kind") != o.kind
                || (was("examtype").is_some() && was("examtype") != examtype)
                || (was("course").is_some() && was("course") != o.course));
        let review = o.confidence < 0.8 || disagree;

        writeln!(
            updates,
            "UPDATE resources SET course = {}, offering = {offering}, kind = {}, examtype = {}, slidetype = {}, n = {}, key = {}, \
             keyfor = {}, confidence = {}, needsreview = {review}, classifiedby = 'llm', classifiedat = NOW(), updated_at = NOW() \
             WHERE id = {} AND classifiedby IS DISTINCT FROM 'manual';",
            opt(o.course.as_ref(), true),
            opt(o.kind.as_ref(), true),
            opt(examtype, true),
            opt(slidetype, true),
            opt(o.n, false),
            o.key,
            opt(keyfor, false),
            o.confidence,
            o.id
        )?;
        count += 1;
    }
    let head = format!("-- {count} classified rows, keyed by resource id; manual rows are never touched\n");
    Ok((format!("{head}{offerings}{updates}"), count, errors))
}

/// reads every answer under `dir/out`, then writes it to the database or to a .sql file
async fn import(db: &PgPool, dir: &Path, sql: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let mut outs: Vec<Out> = Vec::new();
    for entry in std::fs::read_dir(dir.join("out"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            outs.extend(serde_json::from_str::<Vec<Out>>(&std::fs::read_to_string(&path)?)?);
        }
    }
    let (script, count, errors) = render(db, outs).await?;
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

/// runs `classifyexport [repo...]` or `classifyimport [file.sql]` against DATABASE_URL, files live in ./classify
pub async fn run(cmd: &str, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    let dir = Path::new("classify");
    match cmd {
        "classifyexport" => export(&db, dir, args).await,
        _ => import(&db, dir, args.first().map(String::as_str)).await,
    }
}
