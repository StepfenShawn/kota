// Integration tests for Kota tools
// This file runs all tool tests to ensure they work correctly

mod tools;

// Re-export all test modules to ensure they run
pub use tools::*;

#[cfg(test)]
mod integration {
    use super::tools::*;
    
    #[tokio::test]
    async fn test_all_tools_can_be_instantiated() {
        // Test that all tools can be created without errors
        use kota::tools::*;
        
        let _read_tool = WrappedReadFileTool::new();
        let _write_tool = WrappedWriteFileTool::new();
        let _edit_tool = WrappedEditFileTool::new();
        let _delete_tool = WrappedDeleteFileTool::new();
        let _create_dir_tool = WrappedCreateDirectoryTool::new();
        let _execute_tool = WrappedExecuteBashCommandTool::new();
        let _scan_tool = WrappedScanCodebaseTool::new();
        let _grep_tool = WrappedGrepSearchTool::new();
        
        // If we reach here, all tools can be instantiated
        assert!(true);
    }
}