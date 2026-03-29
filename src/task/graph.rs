use egui::Pos2;
use slotmap::SlotMap;
use thiserror::Error;
use crate::{Task, TaskId};

/// A directed acyclic graph (DAG) of tasks.
/// Tasks can have dependencies on other tasks, but task cycles are disallowed.
pub struct TaskGraph {
    nodes: SlotMap<TaskId, TaskNode>,
}

impl TaskGraph {

    /// Inserts a new task, and returns its id
    pub fn insert(&mut self, task: Task) -> TaskId {
        self.nodes.insert(TaskNode::new(task))
    }

    /// Removes a task by id. Returns task if found.
    pub fn remove(&mut self, task_id: TaskId) -> Option<Task> {
        // Removes node
        let removed = self.nodes.remove(task_id)?;
        // Removes from children's list of parents 
        for child_id in removed.children {
            let child = self.nodes.get_mut(child_id).unwrap();
            let parent_idx = child.parents.iter()
                .position(|parent_id| *parent_id == task_id)
                .unwrap();
            child.parents.swap_remove(parent_idx);
        }
        // Removes from parent's list of children
        for parent_id in removed.parents {
            let parent = self.nodes.get_mut(parent_id).unwrap();
            let child_idx = parent.children.iter()
                .position(|child_id| *child_id == task_id)
                .unwrap();
            parent.children.swap_remove(child_idx);
        }
        // Returns only the task
        Some(removed.task)
    }

    pub fn contains_key(&self, task_id: TaskId) -> bool {
        self.nodes.contains_key(task_id)
    }

    /// Adds a direct child dependency to a parent task.
    /// Returns true if a dependency was added, and false if there was no change.
    pub fn add_dependency(&mut self, parent_id: TaskId, child_id: TaskId) -> Result<bool, GraphError> {
        if !self.contains_key(parent_id) || !self.contains_key(child_id) {
            return Err(GraphError::TaskNotFound);
        }
        if self.has_path_to(child_id, parent_id) {
            return Err(GraphError::CycleDetected);
        }
        let [parent, child] = self.nodes.get_disjoint_mut([parent_id, child_id]).unwrap();
        if parent.children.contains(&child_id) {
            return Ok(false);
        }
        parent.children.push(child_id);
        child.parents.push(parent_id);
        Ok(true)
    }

    /// Removes a direct child dependency from the parent task
    /// Returns true if a dependency was removed, and false if there was no change.
    pub fn remove_dependency(&mut self, parent_id: TaskId, child_id: TaskId) -> Result<bool, GraphError> {
        let Some([parent, child]) = self.nodes.get_disjoint_mut([parent_id, child_id]) else {
            return Err(GraphError::TaskNotFound);
        };
        let Some(parent_child_idx) = parent.children.iter().position(|cid| *cid == child_id) else {
            return Ok(false);
        };
        let child_parent_idx = child.parents.iter().position(|pid| *pid == parent_id).unwrap();
        parent.children.swap_remove(parent_child_idx);
        child.parents.swap_remove(child_parent_idx);
        Ok(true)
    }

    // Returns true if task_a has a path to task_b.
    // Assumes that both tasks exist.
    // Panics if they don't.
    fn has_path_to(&self, task_a_id: TaskId, task_b_id: TaskId) -> bool {
        if task_a_id == task_b_id { return true }
        let task_a = self.nodes.get(task_a_id).unwrap();
        for child_id in task_a.children.iter().copied() {
            let has_path = self.has_path_to(child_id, task_b_id);
            if has_path { return true }
        }
        false
    }

    /// Gets node with specified id
    pub fn get(&self, id: TaskId) -> Option<&TaskNode> {
        self.nodes.get(id)
    }

    /// Gets node with specified id
    pub fn get_mut(&mut self, id: TaskId) -> Option<&mut TaskNode> {
        self.nodes.get_mut(id)
    }

    /// Gets first task that contains the given point.
    /// None if not found.
    pub fn get_at_pos(&self, pos: Pos2) -> Option<(TaskId, &TaskNode)> {
        self.nodes.iter()
            .find(|(_, node)| node.task.rect().contains(pos))
    }

    /// Iterator over all task nodes
    pub fn iter(&self) -> impl Iterator<Item=(TaskId, &TaskNode)> {
        self.nodes.iter()
    }

    /// Iterator over all task nodes
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(TaskId, &mut TaskNode)> {
        self.nodes.iter_mut()
    }

    /// Retains only the elements specified by the predicate
    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(TaskId, &mut TaskNode) -> bool,
    {
        self.nodes.retain(predicate);
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self {
            nodes: SlotMap::default(),
        }
    }
}


#[derive(Debug)]
pub struct TaskNode {
    pub task: Task,
    children: Vec<TaskId>,
    parents: Vec<TaskId>,
}

impl TaskNode {
    pub fn children(&self) -> &[TaskId] {
        &self.children
    }
    pub fn parents(&self) -> &[TaskId] {
        &self.parents
    }
}

impl TaskNode {
    fn new(task: Task) -> Self {
        Self {
            task,
            children: vec![],
            parents: vec![],
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

