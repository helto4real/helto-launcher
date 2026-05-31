use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_FAVORITES: usize = 5;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct Favorites {
    #[serde(default)]
    pub items: Vec<String>,
}

impl Favorites {
    pub fn contains(&self, app_id: &str) -> bool {
        self.items.iter().any(|item| item == app_id)
    }

    pub fn can_add(&self, app_id: &str) -> bool {
        self.contains(app_id) || self.items.len() < MAX_FAVORITES
    }

    pub fn toggle(&mut self, app_id: &str) -> bool {
        if self.remove(app_id) {
            return false;
        }

        self.add(app_id)
    }

    pub fn add(&mut self, app_id: &str) -> bool {
        if self.contains(app_id) || self.items.len() >= MAX_FAVORITES {
            return false;
        }
        self.items.push(app_id.to_string());
        true
    }

    pub fn remove(&mut self, app_id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|item| item != app_id);
        before != self.items.len()
    }

    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        if from >= self.items.len() || to >= self.items.len() || from == to {
            return false;
        }

        let item = self.items.remove(from);
        self.items.insert(to, item);
        true
    }

    pub fn visible_items<'a>(&'a self, available_ids: &HashSet<&str>) -> Vec<&'a str> {
        self.items
            .iter()
            .filter(|id| available_ids.contains(id.as_str()))
            .map(String::as_str)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforces_max_favorites() {
        let mut favorites = Favorites::default();
        for id in ["a", "b", "c", "d", "e"] {
            assert!(favorites.add(id));
        }

        assert!(!favorites.add("f"));
        assert_eq!(favorites.items.len(), MAX_FAVORITES);
    }

    #[test]
    fn toggles_and_reorders() {
        let mut favorites = Favorites::default();
        assert!(favorites.toggle("a"));
        assert!(favorites.toggle("b"));
        assert!(!favorites.toggle("a"));
        assert_eq!(favorites.items, ["b"]);

        favorites.add("c");
        favorites.add("d");
        assert!(favorites.reorder(2, 0));
        assert_eq!(favorites.items, ["d", "b", "c"]);
    }

    #[test]
    fn hides_unavailable_without_removing() {
        let favorites = Favorites {
            items: vec!["a".into(), "stale".into(), "b".into()],
        };
        let available = HashSet::from(["a", "b"]);

        assert_eq!(favorites.visible_items(&available), ["a", "b"]);
        assert_eq!(favorites.items[1], "stale");
    }
}
