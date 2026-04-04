use regex::Regex;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::LazyLock;
use uuid::Uuid;

const SOFT_WORDS: &[&str] = &["and", "of", "in", "for", "to"];

static RE_PARENS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(.*?\)").unwrap());
static RE_SEMESTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)sem(?:ester)?[\s\-_]*(\d+)").unwrap());
static RE_CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)[A-Z]{2}\d+\.\d{3}").unwrap());

/// in-memory course registry for path-to-course matching
#[derive(Clone, Debug)]
pub struct CourseRegistry {
    aliases: HashMap<String, Vec<Uuid>>,
    codes: HashMap<String, Uuid>,
    names: Vec<(String, Vec<String>, Uuid)>, // (normalized name, content words, id)
    seasons: HashMap<Uuid, String>,
}

impl CourseRegistry {
    pub async fn load(db: &PgPool) -> Result<Self, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, Vec<String>, String)>(
            "SELECT id, code, name, aliases, semester FROM courses",
        )
        .fetch_all(db)
        .await?;

        let mut aliases: HashMap<String, Vec<Uuid>> = HashMap::new();
        let mut codes: HashMap<String, Uuid> = HashMap::new();
        let mut names = Vec::new();
        let mut seasons = HashMap::new();

        for (id, code, name, course_aliases, semester) in &rows {
            let norm = normalize(name);
            let words = contentwords(&norm).into_iter().map(String::from).collect();
            names.push((norm, words, *id));
            seasons.insert(*id, semester.clone());

            if !code.is_empty() {
                codes.insert(code.to_lowercase(), *id);
            }

            for alias in course_aliases {
                let key = alias.to_lowercase();
                aliases.entry(key).or_default().push(*id);
            }
        }

        for ids in aliases.values_mut() {
            ids.dedup();
        }

        names.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        tracing::info!(
            "loaded {} courses ({} aliases, {} codes)",
            names.len(),
            aliases.len(),
            codes.len()
        );

        Ok(Self {
            aliases,
            codes,
            names,
            seasons,
        })
    }

    /// extract season (spring/monsoon)
    fn extractseason(path: &str) -> Option<&'static str> {
        for seg in path.split('/') {
            let lower = seg.to_lowercase();
            if let Some(caps) = RE_SEMESTER.captures(&lower) {
                if let Ok(n) = caps[1].parse::<u32>() {
                    return if n % 2 == 1 {
                        Some("monsoon")
                    } else {
                        Some("spring")
                    };
                }
            }
            if lower.contains("monsoon") {
                return Some("monsoon");
            }
            if lower.contains("spring") {
                return Some("spring");
            }
        }
        None
    }

    pub fn resolve(&self, path: &str) -> Option<Uuid> {
        let season = Self::extractseason(path);
        let mut candidates: Vec<(Uuid, f32)> = Vec::new();
        let mut has_exact = false;

        for seg in path.split('/') {
            let norm = normalize(seg);
            if norm.is_empty() {
                continue;
            }

            // full name exact match & early exit
            for (name, _, id) in &self.names {
                if norm.contains(name.as_str()) {
                    candidates.push((*id, 1.0));
                    has_exact = true;
                    break;
                }
            }

            if has_exact {
                continue;
            }

            // word level fuzzy match
            let path_words: Vec<&str> = contentwords(&norm);
            if path_words.len() >= 2 {
                let mut best_fuzzy: Option<(Uuid, f32)> = None;
                for (_, course_words, id) in &self.names {
                    if course_words.len() < 2 {
                        continue;
                    }
                    let max_possible =
                        path_words.len().min(course_words.len()) as f32 / course_words.len() as f32;
                    if max_possible < 0.7 {
                        continue;
                    }
                    let cw_refs: Vec<&str> = course_words.iter().map(|s| s.as_str()).collect();
                    let score = wordscore(&path_words, &cw_refs);
                    if score >= 0.7 {
                        let weighted = score * 0.9;
                        if best_fuzzy.map_or(true, |(_, s)| weighted > s) {
                            best_fuzzy = Some((*id, weighted));
                        }
                    }
                }
                if let Some(hit) = best_fuzzy {
                    candidates.push(hit);
                }
            }

            // course code match
            for m in RE_CODE.find_iter(seg) {
                let code = m.as_str().to_lowercase();
                if let Some(&id) = self.codes.get(&code) {
                    candidates.push((id, 0.95));
                }
            }
        }

        // alias match per segment
        if !has_exact {
            for seg in path.split(&['/', '-', '_'][..]) {
                let key = seg.to_lowercase();
                if key.is_empty() {
                    continue;
                }
                if let Some(ids) = self.aliases.get(&key) {
                    let score = if ids.len() == 1 { 0.9 } else { 0.5 };
                    for &id in ids {
                        candidates.push((id, score));
                    }
                }
            }
        }

        let mut best: HashMap<Uuid, f32> = HashMap::new();
        for (id, score) in &candidates {
            let entry = best.entry(*id).or_insert(0.0);
            if *score > *entry {
                *entry = *score;
            }
        }

        // season boost
        if let Some(season) = season {
            for (id, score) in best.iter_mut() {
                if let Some(s) = self.seasons.get(id) {
                    if s == season {
                        *score += 0.3;
                    }
                }
            }
        }

        // filter and sort
        let mut results: Vec<(Uuid, f32)> = best
            .into_iter()
            .filter(|(_, s)| *s >= 0.5)
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        match results.as_slice() {
            [] => None,
            [(id, _)] => Some(*id),
            [(id1, s1), (_, s2), ..] if (s1 - s2).abs() > f32::EPSILON => Some(*id1),
            _ => None, // tie at top
        }
    }

    pub fn name(&self, id: Uuid) -> Option<&str> {
        self.names
            .iter()
            .find(|(_, _, cid)| *cid == id)
            .map(|(name, _, _)| name.as_str())
    }
}

fn normalize(s: &str) -> String {
    let s = s.to_lowercase();
    let s = RE_PARENS.replace_all(&s, "");
    let s = s
        .replace('&', " and ")
        .replace(['-', '_', ':', ',', '.'], " ");
    let words: Vec<String> = s
        .split_whitespace()
        .filter(|w| !matches!(*w, "a" | "an" | "the"))
        .map(|w| match w {
            "viii" => "8".to_string(),
            "vii" => "7".to_string(),
            "vi" => "6".to_string(),
            "iv" => "4".to_string(),
            "v" => "5".to_string(),
            "iii" => "3".to_string(),
            "ii" => "2".to_string(),
            "i" => "1".to_string(),
            _ => w.to_string(),
        })
        .collect();
    words.join(" ")
}

fn contentwords(s: &str) -> Vec<&str> {
    s.split_whitespace()
        .filter(|w| !SOFT_WORDS.contains(w))
        .collect()
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

fn wordmatch(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };

    if short.len() >= 4 && long.starts_with(short) {
        return true;
    }

    if short.len() >= 4 && long.len() >= 4 && levenshtein(a, b) <= 1 {
        return true;
    }
    false
}

fn wordscore(path_words: &[&str], course_words: &[&str]) -> f32 {
    if course_words.is_empty() {
        return 0.0;
    }
    let matched = course_words
        .iter()
        .filter(|cw| path_words.iter().any(|pw| wordmatch(pw, cw)))
        .count();
    matched as f32 / course_words.len() as f32
}


#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> CourseRegistry {
        let cso = Uuid::from_u128(1);
        let dsa_digital = Uuid::from_u128(2);
        let dsa_ds = Uuid::from_u128(3);
        let sci2 = Uuid::from_u128(4);
        let dna = Uuid::from_u128(5);
        let cn = Uuid::from_u128(6);
        let acn = Uuid::from_u128(7);
        let disc = Uuid::from_u128(8);
        let distsys = Uuid::from_u128(9);
        let inlp = Uuid::from_u128(10);
        let robotics = Uuid::from_u128(11);
        let peu = Uuid::from_u128(12);

        let mut aliases: HashMap<String, Vec<Uuid>> = HashMap::new();
        aliases.insert("cso".to_string(), vec![cso]);
        aliases.insert("dsa".to_string(), vec![dsa_digital, dsa_ds]);
        aliases.insert("ds".to_string(), vec![disc, distsys]);
        aliases.insert("acn".to_string(), vec![acn]);
        aliases.insert("inlp".to_string(), vec![inlp]);

        let mut codes = HashMap::new();
        codes.insert("cs1.201".to_string(), dsa_ds);

        let course_data = vec![
            ("Computer Systems Organisation", cso, "spring"),
            ("Digital Signal Analysis", dsa_digital, "spring"),
            ("Data Structures and Algorithms", dsa_ds, "spring"),
            ("Science 2", sci2, "spring"),
            ("Data and Applications", dna, "monsoon"),
            ("Computer Networks", cn, "both"),
            ("Advanced Computer Networks", acn, "monsoon"),
            ("Discrete Structures", disc, "monsoon"),
            ("Distributed Systems", distsys, "both"),
            ("Introduction to NLP", inlp, "monsoon"),
            ("Robotics: Dynamics and Control", robotics, "monsoon"),
            ("Physics of Early Universe", peu, "monsoon"),
        ];

        let mut names: Vec<(String, Vec<String>, Uuid)> = course_data
            .iter()
            .map(|(raw, id, _)| {
                let norm = normalize(raw);
                let words = contentwords(&norm).into_iter().map(String::from).collect();
                (norm, words, *id)
            })
            .collect();
        names.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let seasons = course_data
            .iter()
            .map(|(_, id, sem)| (*id, sem.to_string()))
            .collect();

        CourseRegistry { aliases, codes, names, seasons }
    }

    #[test]
    fn normalize_pipeline() {
        // punctuation, articles, ampersand, parentheticals, roman numerals
        assert_eq!(normalize("Robotics: Dynamics and Control"), "robotics dynamics and control");
        assert_eq!(normalize("Physics of the Early Universe"), "physics of early universe");
        assert_eq!(normalize("Data & Applications"), "data and applications");
        assert_eq!(normalize("Intro to Algorithms Engineering(H2)"), "intro to algorithms engineering");
        assert_eq!(normalize("Science, Technology and Society"), "science technology and society");
        assert_eq!(normalize("Science II"), "science 2");
    }

    #[test]
    fn season_extract() {
        assert_eq!(CourseRegistry::extractseason("sem-1/x/f.pdf"), Some("monsoon"));
        assert_eq!(CourseRegistry::extractseason("Semester 2/f.pdf"), Some("spring"));
        assert_eq!(CourseRegistry::extractseason("monsoon_26/f.pdf"), Some("monsoon"));
        assert_eq!(CourseRegistry::extractseason("random/f.pdf"), None);
    }

    #[test]
    fn name_match() {
        let reg = registry();
        // exact substring
        assert_eq!(reg.resolve("Computer-Systems-Organisation/end.pdf"), Some(Uuid::from_u128(1)));
        // longest wins: ACN before CN
        assert_ne!(
            reg.resolve("Advanced-Computer-Networks/notes.pdf"),
            reg.resolve("Computer-Networks/notes.pdf"),
        );
        // ampersand + roman numeral
        assert_eq!(reg.resolve("DA-Data & Applications/end.pdf"), Some(Uuid::from_u128(5)));
        assert_eq!(reg.resolve("SCI2-Science-II/end.pdf"), Some(Uuid::from_u128(4)));
    }

    #[test]
    fn alias_and_collision() {
        let reg = registry();
        // unique alias
        assert_eq!(reg.resolve("sem2/CSO/A1.pdf"), Some(Uuid::from_u128(1)));
        // collision resolved by season: sem-1 = monsoon → disc (monsoon) beats distsys (both)
        assert_eq!(reg.resolve("sem-1/ds/notes.pdf"), Some(Uuid::from_u128(8)));
        // collision with both same season → tie → None
        assert_eq!(reg.resolve("sem-2/dsa/a1.pdf"), None);
        // no match
        assert_eq!(reg.resolve("random/file.pdf"), None);
    }

    #[test]
    fn code_match() {
        let reg = registry();
        assert_eq!(reg.resolve("CS1.201 Data Structures/end.pdf"), Some(Uuid::from_u128(3)));
    }

    #[test]
    fn fuzzy_match() {
        let reg = registry();
        // prefix: "intro" → "introduction"
        assert_eq!(reg.resolve("Intro-to-NLP/notes.pdf"), Some(Uuid::from_u128(10)));
        // plural: "algorithm" → "algorithms"
        assert_eq!(reg.resolve("Data-Structures-and-Algorithm/hw.pdf"), Some(Uuid::from_u128(3)));
        // article strip: "the" dropped
        assert_eq!(reg.resolve("Physics-of-the-Early-Universe/notes.pdf"), Some(Uuid::from_u128(12)));
        // colon strip + plural: "Controls" → "Control"
        assert_eq!(reg.resolve("Robotics-Dynamics-And-Controls/a1.pdf"), Some(Uuid::from_u128(11)));
    }
}
