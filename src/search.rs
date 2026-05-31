use crate::desktop::DesktopApp;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult<'a> {
    pub app: &'a DesktopApp,
    pub score: i32,
}

pub fn filter_and_rank<'a>(
    apps: &'a [DesktopApp],
    query: &str,
    launch_counts: &BTreeMap<String, u64>,
) -> Vec<SearchResult<'a>> {
    let query = query.trim().to_lowercase();

    let mut results: Vec<_> = if query.is_empty() {
        apps.iter()
            .map(|app| SearchResult { app, score: 0 })
            .collect()
    } else {
        apps.iter()
            .filter_map(|app| rank_app(app, &query).map(|score| SearchResult { app, score }))
            .collect()
    };

    if query.is_empty() {
        results.sort_by(|left, right| {
            let left_count = launch_counts.get(&left.app.id).copied().unwrap_or_default();
            let right_count = launch_counts
                .get(&right.app.id)
                .copied()
                .unwrap_or_default();
            right_count.cmp(&left_count).then_with(|| {
                left.app
                    .name
                    .to_lowercase()
                    .cmp(&right.app.name.to_lowercase())
            })
        });
    } else {
        results.sort_by(|left, right| {
            left.score.cmp(&right.score).then_with(|| {
                left.app
                    .name
                    .to_lowercase()
                    .cmp(&right.app.name.to_lowercase())
            })
        });
    }

    results
}

fn rank_app(app: &DesktopApp, query: &str) -> Option<i32> {
    let name = app.name.to_lowercase();
    if name == query {
        return Some(0);
    }
    if name.starts_with(query) {
        return Some(1);
    }
    if name.contains(query) {
        return Some(2);
    }
    if app
        .keywords
        .iter()
        .any(|keyword| keyword.to_lowercase().contains(query))
    {
        return Some(3);
    }
    if app
        .generic_name
        .as_deref()
        .is_some_and(|value| value.to_lowercase().contains(query))
        || app
            .comment
            .as_deref()
            .is_some_and(|value| value.to_lowercase().contains(query))
    {
        return Some(4);
    }
    if fuzzy_match(&name, query) {
        return Some(5);
    }
    None
}

fn fuzzy_match(value: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let mut chars = value.chars();
    query
        .chars()
        .all(|needle| chars.by_ref().any(|candidate| candidate == needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::DesktopApp;
    use crate::exec::CommandLine;
    use std::path::PathBuf;

    fn app(id: &str, name: &str, keywords: &[&str], generic: Option<&str>) -> DesktopApp {
        DesktopApp {
            id: id.into(),
            path: PathBuf::from(id),
            name: name.into(),
            generic_name: generic.map(str::to_string),
            comment: Some("Example app".into()),
            exec: name.into(),
            command: CommandLine {
                program: name.into(),
                args: Vec::new(),
            },
            icon: None,
            terminal: false,
            categories: Vec::new(),
            keywords: keywords.iter().map(|value| value.to_string()).collect(),
        }
    }

    #[test]
    fn ranks_stronger_name_matches_first() {
        let apps = vec![
            app("contains.desktop", "Best Firefox", &[], None),
            app("prefix.desktop", "Fire Control", &[], None),
            app("keyword.desktop", "Browser", &["fire"], None),
        ];

        let results = filter_and_rank(&apps, "fire", &BTreeMap::new());
        let ids: Vec<_> = results
            .iter()
            .map(|result| result.app.id.as_str())
            .collect();

        assert_eq!(
            ids,
            ["prefix.desktop", "contains.desktop", "keyword.desktop"]
        );
    }

    #[test]
    fn empty_query_sorts_by_launch_count_then_name() {
        let apps = vec![
            app("b.desktop", "Beta", &[], None),
            app("a.desktop", "Alpha", &[], None),
            app("c.desktop", "Gamma", &[], None),
        ];
        let counts = BTreeMap::from([("c.desktop".into(), 4), ("a.desktop".into(), 4)]);

        let results = filter_and_rank(&apps, "", &counts);
        let ids: Vec<_> = results
            .iter()
            .map(|result| result.app.id.as_str())
            .collect();

        assert_eq!(ids, ["a.desktop", "c.desktop", "b.desktop"]);
    }
}
