use std::collections::HashMap;

use egui::Pos2;
use slotmap::SlotMap;
use crate::{Task, TaskId};

pub struct TaskGraph {
    tasks: SlotMap<TaskId, Task>,
    dependencies: HashMap<TaskId, Vec<TaskId>>,
}

impl TaskGraph {

    pub fn insert(&mut self, task: Task) -> TaskId {
        self.tasks.insert(task)
    }

    pub fn remove(&mut self, task_id: TaskId) -> Option<Task> {
        // Remove from graph
        let removed = self.tasks.remove(task_id);
        // Remove task dependencies entry
        self.dependencies.remove(&task_id);
        // Remove task id from all tasks that had dependencies on it
        for (_, task_deps) in &mut self.dependencies {
            task_deps.retain(|tid| *tid != task_id);
        }
        removed
    }

    pub fn contains_dependency(&mut self, parent_id: TaskId, child_id: TaskId) -> bool {
        if
            !self.tasks.contains_key(parent_id) ||
            !self.tasks.contains_key(child_id) ||
            !self.dependencies.contains_key(&parent_id)
        {
            return false
        }
        let parent_deps = self.dependencies.entry(parent_id).or_default();
        parent_deps.contains(&child_id)
    }

    pub fn add_dependency(&mut self, parent_id: TaskId, child_id: TaskId) -> bool {
        if !self.tasks.contains_key(parent_id) || !self.tasks.contains_key(child_id) {
            return false
        }
        let parent_deps = self.dependencies.entry(parent_id).or_default();
        parent_deps.push(child_id);
        false
    }

    pub fn remove_dependency(&mut self, parent_id: TaskId, child_id: TaskId) -> bool {
        if !self.tasks.contains_key(parent_id) || !self.tasks.contains_key(child_id) {
            return false
        }
        let parent_deps = self.dependencies.entry(parent_id).or_default();
        parent_deps.retain(|task_id| *task_id != child_id);
        false
    }

    /// Gets task with specified id
    pub fn get(&self, id: TaskId) -> Option<&Task> {
        self.tasks.get(id)
    }

    /// Gets first task that contains the given point.
    /// None if not found.
    pub fn get_at_pos(&self, pos: Pos2) -> Option<(TaskId, &Task)> {
        self.tasks.iter()
            .find(|(_, task)| task.rect().contains(pos))
    }

    pub fn get_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        self.tasks.get_mut(id)
    }

    pub fn iter(&self) -> impl Iterator<Item=(TaskId, &Task)> {
        self.tasks.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=(TaskId, &mut Task)> {
        self.tasks.iter_mut()
    }

    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(TaskId, &mut Task) -> bool,
    {
        self.tasks.retain(predicate);
    }

    pub fn dependencies(&self) -> impl Iterator<Item=(TaskId, &[TaskId])> {
        self.dependencies.iter()
            .map(|(task_id, deps)| (*task_id, deps.as_slice()))
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self {
            tasks: SlotMap::default(),
            dependencies: HashMap::default(), 
        }
    }
}


