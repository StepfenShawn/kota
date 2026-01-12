use kota::tools::write_file::{WriteFileTool, WriteFileArgs};
use kota::tools::FileToolError;
use rig::tool::Tool;
use super::create_temp_dir;
use std::fs;

#[tokio::test]
async fn test_write_new_file() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("new_file.txt").to_string_lossy().to_string();
    let content = "Hello, world!\nThis is new content.";
    
    let tool = WriteFileTool;
    let args = WriteFileArgs { 
        file_path: file_path.clone(),
        content: content.to_string(),
    };
    
    let result: Result<_, FileToolError> = tool.call(args).await;
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.file_path, file_path);
    assert_eq!(output.bytes_written, content.len() as u64);
    assert!(output.message.contains("Successfully wrote"));
    
    // Verify file was actually written
    let written_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(written_content, content);
}

#[tokio::test]
async fn test_overwrite_existing_file() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("existing.txt").to_string_lossy().to_string();
    
    // Create initial file
    fs::write(&file_path, "Original content").unwrap();
    
    let new_content = "New content that replaces the old";
    let tool = WriteFileTool;
    let args = WriteFileArgs { 
        file_path: file_path.clone(),
        content: new_content.to_string(),
    };
    
    let result: Result<_, FileToolError> = tool.call(args).await;
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.bytes_written, new_content.len() as u64);
    
    // Verify file was overwritten
    let written_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(written_content, new_content);
}