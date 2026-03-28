use std::collections::{HashMap, HashSet};

use egui::Pos2;
use slotmap::SlotMap;
use thiserror::Error;
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

    pub fn add_dependency(&mut self, parent_id: TaskId, child_id: TaskId) -> Result<(), GraphError> {
        if !self.tasks.contains_key(parent_id) || !self.tasks.contains_key(child_id) {
            return Err(GraphError::TaskNotFound);
        }
        if self.has_path_to(child_id, parent_id) {
            return Err(GraphError::CycleDetected);
        }
        let parent_deps = self.dependencies.entry(parent_id).or_default();
        parent_deps.push(child_id);
        Ok(())
    }
    
    pub fn has_path_to(&self, task_a: TaskId, task_b: TaskId) -> bool {
        let mut visited = HashSet::new();
        self._has_path_to(task_a, task_b, &mut visited)
    }

    fn _has_path_to(
        &self,
        task_a: TaskId,
        task_b: TaskId,
        visited: &mut HashSet<TaskId>,
    ) -> bool {
        if task_a == task_b { return true }
        visited.insert(task_a);
        let Some(deps) = self.dependencies.get(&task_a) else { return false };
        for dep in deps.iter().copied() {
            if visited.contains(&dep) { continue };
            let has_path = self._has_path_to(dep, task_b, visited);
            if has_path { return true }
        }
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

#[derive(Error, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GraphError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Cycle detected")]
    CycleDetected, 
}
