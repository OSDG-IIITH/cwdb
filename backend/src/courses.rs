use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// in-memory course registry for path-to-course matching
#[derive(Clone, Debug)]
pub struct CourseRegistry {
    aliases: HashMap<String, Option<Uuid>>,
    names: Vec<(String, Uuid)>,
}

impl CourseRegistry {
    pub async fn load(db: &PgPool) -> Result<Self, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, String, Vec<String>)>(
            "SELECT id, name, aliases FROM courses",
        )
        .fetch_all(db)
        .await?;

        let mut aliases: HashMap<String, Option<Uuid>> = HashMap::new();
        let mut names = Vec::new();

        for (id, name, course_aliases) in &rows {
            names.push((normalize(name), *id));

            for alias in course_aliases {
                let key = alias.to_lowercase();
                aliases
                    .entry(key)
                    .and_modify(|existing| {
                        if *existing != Some(*id) {
                            *existing = None; // collision
                        }
                    })
                    .or_insert(Some(*id));
            }
        }

        // sort longest name first so "advanced computer networks" matches before "computer networks"
        names.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        tracing::info!("loaded {} courses ({} aliases)", names.len(), aliases.len());

        Ok(Self { aliases, names })
    }

    // FIXME: on multiple matches, currently takes first match — revisit with priority/scoring later
    pub fn resolve(&self, path: &str) -> Option<Uuid> {
        for seg in path.split('/') {
            let norm = normalize(seg);
            if norm.is_empty() {
                continue;
            }
            for (name, id) in &self.names {
                if norm.contains(name.as_str()) {
                    return Some(*id);
                }
            }
        }

        for seg in path.split(&['/', '-', '_'][..]) {
            let key = seg.to_lowercase();
            if let Some(Some(id)) = self.aliases.get(&key) {
                return Some(*id);
            }
        }

        None
    }

    pub fn name(&self, id: Uuid) -> Option<&str> {
        self.names
            .iter()
            .find(|(_, cid)| *cid == id)
            .map(|(name, _)| name.as_str())
    }
}

fn normalize(s: &str) -> String {
    let s = s.to_lowercase().replace(['-', '_'], " ").replace('&', "and");
    let words: Vec<String> = s
        .split_whitespace()
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


/// unit tests
#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> CourseRegistry {
        let id_cso = Uuid::new_v4();
        let id_dsa1 = Uuid::new_v4();
        let id_dsa2 = Uuid::new_v4();
        let id_sci2 = Uuid::new_v4();
        let id_dna = Uuid::new_v4();
        let id_cn = Uuid::new_v4();
        let id_acn = Uuid::new_v4();

        let mut aliases = HashMap::new();
        aliases.insert("cso".to_string(), Some(id_cso));
        aliases.insert("dsa".to_string(), None);
        aliases.insert("cn".to_string(), Some(id_cn));
        aliases.insert("acn".to_string(), Some(id_acn));

        let mut names = vec![
            (normalize("Computer Systems Organisation"), id_cso),
            (normalize("Digital Signal Analysis"), id_dsa1),
            (normalize("Data Structures & Algorithms"), id_dsa2),
            (normalize("Science 2"), id_sci2),
            (normalize("Data & Applications"), id_dna),
            (normalize("Computer Networks"), id_cn),
            (normalize("Advanced Computer Networks"), id_acn),
        ];
        names.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        CourseRegistry { aliases, names }
    }

    #[test]
    fn aliasmatch() {
        let reg = registry();
        let id = reg.resolve("resources/sem2/CSO/CSO A1.pdf");
        assert!(id.is_some());
    }

    #[test]
    fn collisionskip() {
        let reg = registry();
        let id = reg.resolve("resources/sem2/DSA/Assignments/DSA Assignment 1.pdf");
        assert!(id.is_none());
    }

    #[test]
    fn substringmatch() {
        let reg = registry();
        let id = reg.resolve("Computer-Systems-Organisation/endsem.pdf");
        assert!(id.is_some());
    }

    #[test]
    fn romannumeral() {
        let reg = registry();
        let id = reg.resolve("Year2/CS/SCI2-Science-II/SCI2_end_2022_S_key.pdf");
        assert!(id.is_some());
    }

    #[test]
    fn normalization() {
        let reg = registry();
        let id = reg.resolve("Year2/CS/DA-Data & Applications/DA_end_2023_M.pdf");
        assert_eq!(id, reg.resolve("Data-and-Applications/slides.pdf"));
        assert!(id.is_some());
    }

    #[test]
    fn longestmatch() {
        let reg = registry();
        let id_cn = reg.resolve("sem4/Computer-Networks/notes.pdf");
        let id_acn = reg.resolve("sem6/Advanced-Computer-Networks/notes.pdf");
        assert!(id_cn.is_some());
        assert!(id_acn.is_some());
        assert_ne!(id_cn, id_acn);
    }

    #[test]
    fn none() {
        let reg = registry();
        assert!(reg.resolve("random/file.pdf").is_none());
    }
}
