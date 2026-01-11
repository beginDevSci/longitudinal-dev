use serde::{Deserialize, Serialize};

/// Generic undo/redo history for interaction state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History<T> {
    past: Vec<T>,
    present: T,
    future: Vec<T>,
}

impl<T: Clone> History<T> {
    /// Create a new history with the given initial state.
    pub fn new(initial: T) -> Self {
        Self {
            past: Vec::new(),
            present: initial,
            future: Vec::new(),
        }
    }

    /// Borrow the current state.
    pub fn present(&self) -> &T {
        &self.present
    }

    /// Apply a new state and push the previous one onto the undo stack.
    pub fn apply(&mut self, next: T) {
        self.past.push(self.present.clone());
        self.present = next;
        self.future.clear();
    }

    /// Undo to the previous state, if any.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.past.pop() {
            let current = std::mem::replace(&mut self.present, prev);
            self.future.push(current);
            true
        } else {
            false
        }
    }

    /// Redo to the next state, if any.
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.future.pop() {
            let current = std::mem::replace(&mut self.present, next);
            self.past.push(current);
            true
        } else {
            false
        }
    }
}

