use crate::kota_code::llm::ToolDyn;

pub struct ToolRegistry {
    tools: Vec<Box<dyn ToolDyn>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Add a tool to the registry
    pub fn add(&mut self, tool: Box<dyn ToolDyn>) {
        self.tools.push(tool);
    }

    /// Remove a tool by index
    pub fn remove(&mut self, index: usize) {
        if index < self.tools.len() {
            self.tools.remove(index);
        }
    }

    /// Get all tools by taking ownership (empties the registry)
    pub fn take_all(&mut self) -> Vec<Box<dyn ToolDyn>> {
        std::mem::take(&mut self.tools)
    }

    /// Get all tools as a reference
    pub fn get_all(&self) -> &[Box<dyn ToolDyn>] {
        &self.tools
    }

    /// Get all tools as a mutable reference
    pub fn get_all_mut(&mut self) -> &mut Vec<Box<dyn ToolDyn>> {
        &mut self.tools
    }

    /// Get the number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Clear all tools from the registry
    pub fn clear(&mut self) {
        self.tools.clear();
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
