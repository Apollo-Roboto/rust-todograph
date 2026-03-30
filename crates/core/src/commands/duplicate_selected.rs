use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::MindTask;
use crate::commands::Command;
use crate::editor::EditorState;

#[derive(Debug, Clone)]
pub struct DuplicateSelectedCommand {
    duplicated_items: Option<Vec<MindTask>>,
    previously_active: Option<Option<u32>>,
    original_selection: Option<HashSet<u32>>,
    creation_time: Option<DateTime<Utc>>,
}
impl DuplicateSelectedCommand {
    pub fn new() -> Self {
        Self {
            duplicated_items: None,
            previously_active: None,
            original_selection: None,
            creation_time: None,
        }
    }
}
impl Display for DuplicateSelectedCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duplicate selection")
    }
}
impl Command for DuplicateSelectedCommand {
    fn execute(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if self.duplicated_items.is_none() {
            let mut items_to_duplicate: Vec<&mut MindTask> = editor
                .graph
                .tasks
                .iter_mut()
                .filter(|t| t.selected == true)
                .collect();

            self.original_selection = Some(items_to_duplicate.iter().map(|t| t.id).collect());

            let mut active_task_index = None;

            for (i, item) in &mut items_to_duplicate.iter_mut().enumerate() {
                item.selected = false;
                if Some(item.id) == editor.active_task {
                    self.previously_active = Some(editor.active_task);
                    active_task_index = Some(i);
                }
            }

            let mut cloned_items: Vec<MindTask> = items_to_duplicate
                .iter()
                .by_ref()
                .clone()
                .map(|t| (*t).clone())
                .collect();

            // reuse existing time for redo
            let now = match self.creation_time {
                Some(time) => time,
                None => {
                    let now = Utc::now();
                    self.creation_time = Some(now);
                    now
                }
            };

            let mut old_new_id_mapping: HashMap<u32, u32> = HashMap::new();

            for item in &mut cloned_items {
                let original_id = item.id;
                let new_id = editor.graph.generate_id();

                old_new_id_mapping.insert(original_id, new_id);

                item.id = new_id;
                item.creation_date = now;
                item.selected = true;
            }

            // update the parents to the new ids
            for item in &mut cloned_items {
                if let Some(old_parent_id) = item.parent {
                    let Some(new_parent_id) = old_new_id_mapping.get(&old_parent_id) else {
                        continue;
                    };
                    item.parent = Some(*new_parent_id);
                } else {
                    item.parent = None;
                }
            }

            // update the dependencies to the new ids, depdendencies could be outside of the selection
            for item in &mut cloned_items {
                item.depends_on = item
                    .depends_on
                    .iter()
                    .map(|&old_dep_id| {
                        old_new_id_mapping
                            .get(&old_dep_id)
                            .copied()
                            .unwrap_or(old_dep_id)
                    })
                    .collect();
            }

            if let Some(active_index) = active_task_index
                && let Some(new_active_task) = cloned_items.get(active_index)
            {
                editor.active_task = Some(new_active_task.id);
            }

            self.duplicated_items = Some(cloned_items);
        }

        if let Some(items_to_duplicate) = self.duplicated_items.as_ref() {
            for item in items_to_duplicate {
                editor.graph.create_task(item.clone());
            }
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut EditorState) -> Result<(), String> {
        if let Some(items) = self.duplicated_items.as_ref() {
            for item in items {
                editor.graph.delete_task(item.id);
            }
        }
        if let Some(previously_active) = self.previously_active {
            editor.active_task = previously_active;
        }

        if let Some(original_selection) = self.original_selection.as_ref() {
            editor
                .graph
                .tasks
                .iter_mut()
                .for_each(|t| t.selected = original_selection.contains(&t.id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use serde_json::{Value, json};

    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_undo_redo() {
        let mut state = EditorState::default();

        let id1 = state.graph.generate_id();
        let id2 = state.graph.generate_id();
        let id3 = state.graph.generate_id();
        let id4 = state.graph.generate_id();
        let id5 = state.graph.generate_id();

        state.graph.tasks.push(crate::MindTask {
            id: id1,
            title: String::from("find me id1"),
            selected: true,
            ..Default::default()
        });
        state.graph.tasks.push(crate::MindTask {
            id: id2,
            title: String::from("find me id2"),
            parent: Some(id1),
            selected: true,
            ..Default::default()
        });
        state.graph.tasks.push(crate::MindTask {
            id: id3,
            title: String::from("find me id3"),
            parent: Some(id1),
            selected: true,
            depends_on: HashSet::from_iter(vec![id1, id2]),
            ..Default::default()
        });
        state.graph.tasks.push(crate::MindTask {
            id: id4,
            title: String::from("find me id4"),
            selected: true,
            ..Default::default()
        });
        state.graph.tasks.push(crate::MindTask {
            id: id5,
            ..Default::default()
        });

        state.active_task = Some(id2);

        let mut cmd = DuplicateSelectedCommand::new();

        let state_before_execute = state.clone();

        cmd.execute(&mut state).unwrap();

        assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );

        fn get_original_item(state: &EditorState, id: u32) -> &MindTask {
            state.graph.tasks.iter().find(|t| t.id == id).unwrap()
        }
        fn get_duplicated_item<'a, 'b>(state: &'a EditorState, title: &'b str, id: u32,) -> &'a MindTask {
            state.graph.tasks.iter().find(|t| t.title == title.to_string() && t.id != id).unwrap()
        }

        let original_id1 = get_original_item(&state, id1);
        let original_id2 = get_original_item(&state, id2);
        let original_id3 = get_original_item(&state, id3);
        let original_id4 = get_original_item(&state, id4);
        let duplicate_of_id1 = get_duplicated_item(&state, "find me id1", id1);
        let duplicate_of_id2 = get_duplicated_item(&state, "find me id2", id2);
        let duplicate_of_id3 = get_duplicated_item(&state, "find me id3", id3);
        let duplicate_of_id4 = get_duplicated_item(&state, "find me id4", id4);

        // utility function for comparing results
        fn sorted_vec(vec: Vec<u32>) -> Vec<u32> {
            let mut v: Vec<u32> = vec.iter().copied().collect();
            v.sort();
            v
        }
        fn sorted_hashset(set: &HashSet<u32>) -> Vec<u32> {
            let mut v: Vec<u32> = set.iter().copied().collect();
            v.sort();
            v
        }

        let mut expectation: Vec<(&str, Value)> = Vec::new();
        expectation.push(("duplicate_of_id1_parent", Value::Null));
        expectation.push(("duplicate_of_id2_parent", json!(Some(duplicate_of_id1.id))));
        expectation.push(("duplicate_of_id3_parent", json!(Some(duplicate_of_id1.id))));
        expectation.push(("duplicate_of_id4_parent", Value::Null));
        expectation.push(("duplicate_of_id1_selected", json!(true)));
        expectation.push(("duplicate_of_id2_selected", json!(true)));
        expectation.push(("duplicate_of_id3_selected", json!(true)));
        expectation.push(("duplicate_of_id4_selected", json!(true)));
        expectation.push(("duplicate_of_id3_depends_on", json!(sorted_vec(vec![duplicate_of_id1.id, duplicate_of_id2.id]))));
        expectation.push(("original_id1_selected", json!(false)));
        expectation.push(("original_id2_selected", json!(false)));
        expectation.push(("original_id3_selected", json!(false)));
        expectation.push(("original_id4_selected", json!(false)));
        expectation.push(("original_id3_depends_on", json!(sorted_vec(vec![id1, id2]))));
        expectation.push(("selection_count", json!(4)));
        expectation.push(("active_task", json!(Some(duplicate_of_id2.id))));

        let mut result: Vec<(&str, Value)> = Vec::new();
        result.push(("duplicate_of_id1_parent", json!(duplicate_of_id1.parent)));
        result.push(("duplicate_of_id2_parent", json!(duplicate_of_id2.parent)));
        result.push(("duplicate_of_id3_parent", json!(duplicate_of_id3.parent)));
        result.push(("duplicate_of_id4_parent", json!(duplicate_of_id4.parent)));
        result.push(("duplicate_of_id1_selected", json!(duplicate_of_id1.selected)));
        result.push(("duplicate_of_id2_selected", json!(duplicate_of_id2.selected)));
        result.push(("duplicate_of_id3_selected", json!(duplicate_of_id3.selected)));
        result.push(("duplicate_of_id4_selected", json!(duplicate_of_id4.selected)));
        result.push(("duplicate_of_id3_depends_on", json!(sorted_hashset(&duplicate_of_id3.depends_on))));
        result.push(("original_id1_selected", json!(original_id1.selected)));
        result.push(("original_id2_selected", json!(original_id2.selected)));
        result.push(("original_id3_selected", json!(original_id3.selected)));
        result.push(("original_id4_selected", json!(original_id4.selected)));
        result.push(("original_id3_depends_on", json!(sorted_hashset(&original_id3.depends_on))));
        result.push(("selection_count", json!(state.graph.tasks.iter().filter(|t| t.selected).count())));
        result.push(("active_task", json!(state.active_task)));

        pretty_assertions::assert_eq!(expectation, result);

        cmd.undo(&mut state).unwrap();
        pretty_assertions::assert_eq!(
            state_before_execute, state,
            "The state was supposed to be identical to before"
        );
        cmd.execute(&mut state).unwrap();
        pretty_assertions::assert_ne!(
            state_before_execute, state,
            "The state was supposed to change"
        );
    }
}
