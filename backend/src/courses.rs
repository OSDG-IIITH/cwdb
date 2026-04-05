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

    /// look up course id by normalized name
    fn resolvebyname(&self, name: &str) -> Option<Uuid> {
        let norm = normalize(name);
        self.names.iter().find(|(n, _, _)| *n == norm).map(|(_, _, id)| *id)
    }

    pub fn resolve(&self, path: &str, repo: &str, source_aliases: &HashMap<String, String>) -> Option<Uuid> {
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

            // tokenize filename stem on all non-alphanumeric boundaries
            // catches aliases embedded in filenames like "OSN Sep 24.pdf"
            if let Some(filename) = path.split('/').last() {
                let stem = match filename.rfind('.') {
                    Some(i) => &filename[..i],
                    None => filename,
                };
                for token in stem.split(|c: char| !c.is_alphanumeric()) {
                    let key = token.to_lowercase();
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

            // alias match on repo name
            for token in repo.split(|c: char| !c.is_alphanumeric()) {
                let key = token.to_lowercase();
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
            _ => {
                if source_aliases.is_empty() {
                    return None;
                }
                self.tiebreak(path, &results, source_aliases)
            }
        }
    }

    // tiebreaker after alias collision
    fn tiebreak(
        &self,
        path: &str,
        tied: &[(Uuid, f32)],
        source_aliases: &HashMap<String, String>,
    ) -> Option<Uuid> {
        let top = tied[0].1;
        let tied_ids: Vec<Uuid> = tied.iter()
            .take_while(|(_, s)| (s - top).abs() <= f32::EPSILON)
            .map(|(id, _)| *id)
            .collect();

        let path_tokens: Vec<String> = path
            .split(&['/', '-', '_'][..])
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        for token in &path_tokens {
            if let Some(course_name) = source_aliases.get(token.as_str()) {
                if let Some(id) = self.resolvebyname(course_name) {
                    if tied_ids.contains(&id) {
                        return Some(id);
                    }
                }
            }
        }

        for (pattern, course_name) in source_aliases {
            let pattern_segs: Vec<&str> = pattern
                .split(&['/', '-', '_'][..])
                .filter(|s| !s.is_empty())
                .collect();
            if pattern_segs.is_empty() {
                continue;
            }
            if segmentaligned(&path_tokens, &pattern_segs) {
                if let Some(id) = self.resolvebyname(course_name) {
                    if tied_ids.contains(&id) {
                        return Some(id);
                    }
                }
            }
        }

        None
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

fn segmentaligned(path_tokens: &[String], pattern_segs: &[&str]) -> bool {
    if pattern_segs.len() > path_tokens.len() {
        return false;
    }
    path_tokens
        .windows(pattern_segs.len())
        .any(|w| w.iter().zip(pattern_segs).all(|(a, b)| a == &b.to_lowercase()))
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
        let mdl = Uuid::from_u128(13);
        let osn = Uuid::from_u128(14);

        let mut aliases: HashMap<String, Vec<Uuid>> = HashMap::new();
        aliases.insert("cso".to_string(), vec![cso]);
        aliases.insert("dsa".to_string(), vec![dsa_digital, dsa_ds]);
        aliases.insert("ds".to_string(), vec![disc, distsys]);
        aliases.insert("acn".to_string(), vec![acn]);
        aliases.insert("inlp".to_string(), vec![inlp]);
        aliases.insert("mdl".to_string(), vec![mdl]);
        aliases.insert("osn".to_string(), vec![osn]);

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
            ("Machine Data Learning", mdl, "monsoon"),
            ("Operating Systems and Networks", osn, "monsoon"),
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

    fn noaliases() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn normalize_pipeline() {
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
        let e = noaliases();
        assert_eq!(reg.resolve("Computer-Systems-Organisation/end.pdf", "", &e), Some(Uuid::from_u128(1)));
        assert_ne!(
            reg.resolve("Advanced-Computer-Networks/notes.pdf", "", &e),
            reg.resolve("Computer-Networks/notes.pdf", "", &e),
        );
        assert_eq!(reg.resolve("DA-Data & Applications/end.pdf", "", &e), Some(Uuid::from_u128(5)));
        assert_eq!(reg.resolve("SCI2-Science-II/end.pdf", "", &e), Some(Uuid::from_u128(4)));
    }

    #[test]
    fn alias_and_collision() {
        let reg = registry();
        let e = noaliases();
        assert_eq!(reg.resolve("sem2/CSO/A1.pdf", "", &e), Some(Uuid::from_u128(1)));
        assert_eq!(reg.resolve("sem-1/ds/notes.pdf", "", &e), Some(Uuid::from_u128(8)));
        assert_eq!(reg.resolve("sem-2/dsa/a1.pdf", "", &e), None);
        assert_eq!(reg.resolve("random/file.pdf", "", &e), None);
    }

    #[test]
    fn code_match() {
        let reg = registry();
        assert_eq!(reg.resolve("CS1.201 Data Structures/end.pdf", "", &noaliases()), Some(Uuid::from_u128(3)));
    }

    #[test]
    fn fuzzy_match() {
        let reg = registry();
        let e = noaliases();
        assert_eq!(reg.resolve("Intro-to-NLP/notes.pdf", "", &e), Some(Uuid::from_u128(10)));
        assert_eq!(reg.resolve("Data-Structures-and-Algorithm/hw.pdf", "", &e), Some(Uuid::from_u128(3)));
        assert_eq!(reg.resolve("Physics-of-the-Early-Universe/notes.pdf", "", &e), Some(Uuid::from_u128(12)));
        assert_eq!(reg.resolve("Robotics-Dynamics-And-Controls/a1.pdf", "", &e), Some(Uuid::from_u128(11)));
    }

    #[test]
    fn filename_token_match() {
        let reg = registry();
        assert_eq!(reg.resolve("user/repo/sem-1/OSN Sep 24.pdf", "", &noaliases()), Some(Uuid::from_u128(14)));
    }

    #[test]
    fn repo_name_match() {
        let reg = registry();
        let e = noaliases();
        assert_eq!(reg.resolve("sem-1/lecture.pdf", "MDL-lecs", &e), Some(Uuid::from_u128(13)));
        assert_eq!(reg.resolve("lecture1.pdf", "MDL-lecs", &e), Some(Uuid::from_u128(13)));
    }

    #[test]
    fn source_alias_direct_tiebreak() {
        let reg = registry();
        let mut sa = HashMap::new();
        sa.insert("dsa".to_string(), "Data Structures and Algorithms".to_string());
        assert_eq!(reg.resolve("sem-2/dsa/a1.pdf", "", &sa), Some(Uuid::from_u128(3)));

        let mut sa2 = HashMap::new();
        sa2.insert("dsa".to_string(), "Digital Signal Analysis".to_string());
        assert_eq!(reg.resolve("sem-2/dsa/a1.pdf", "", &sa2), Some(Uuid::from_u128(2)));
    }

    #[test]
    fn source_alias_segment_tiebreak() {
        let reg = registry();
        let mut sa = HashMap::new();
        sa.insert("sem2/dsa".to_string(), "Data Structures and Algorithms".to_string());
        sa.insert("sem4/dsa".to_string(), "Digital Signal Analysis".to_string());

        assert_eq!(reg.resolve("stuff/sem2/dsa/notes.pdf", "", &sa), Some(Uuid::from_u128(3)));
        assert_eq!(reg.resolve("stuff/sem4/dsa/notes.pdf", "", &sa), Some(Uuid::from_u128(2)));
    }

    #[test]
    fn source_alias_no_match_still_none() {
        let reg = registry();
        let mut sa = HashMap::new();
        sa.insert("foo".to_string(), "Computer Networks".to_string());
        assert_eq!(reg.resolve("sem-2/dsa/a1.pdf", "", &sa), None);
    }

    #[test]
    fn segment_aligned_check() {
        let tokens = vec!["stuff".into(), "sem2".into(), "dsa".into(), "notes.pdf".into()];
        assert!(segmentaligned(&tokens, &["sem2", "dsa"]));
        assert!(segmentaligned(&tokens, &["dsa"]));
        assert!(!segmentaligned(&tokens, &["sem2", "notes.pdf"]));
        assert!(!segmentaligned(&tokens, &["sem4", "dsa"]));
    }
}
