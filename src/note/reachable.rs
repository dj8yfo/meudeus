use std::collections::HashSet;

use sqlx::Result as SqlxResult;

use crate::{
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
};

use super::Note;

impl super::Note {
    pub async fn reachable_notes(
        &self,
        db: SqliteAsyncHandle,
        md_static: MarkdownStatic,
    ) -> SqlxResult<Vec<Self>> {
        let mut reachable_all: HashSet<Note> = HashSet::new();
        let mut current_layer: HashSet<Note> = HashSet::new();
        current_layer.insert(self.clone());

        loop {
            let mut next_layer: HashSet<Note> = HashSet::new();

            let lock = db.lock().await;
            for note in &current_layer {
                let forward_links = lock.find_links_from(&note.name(), md_static).await?;
                next_layer.extend(forward_links);
            }
            reachable_all.extend(current_layer.drain());
            let diff: HashSet<_> = next_layer.difference(&reachable_all).collect();
            if diff.is_empty() {
                break;
            }

            current_layer = next_layer;
        }
        let all_vec: Vec<_> = reachable_all.into_iter().collect();
        Ok(all_vec)
    }
}
