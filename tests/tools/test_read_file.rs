use super::{create_temp_dir, create_test_file};
use kota::tools::read_file::{ReadFileArgs, ReadFileTool};
use kota::tools::FileToolError;
use rig::tool::Tool;

#[tokio::test]
async fn test_read_existing_file() {
    let temp_dir = create_temp_dir();
    let content = "Hello, world!\nThis is a test file.";
    let file_path = create_test_file(temp_dir.path(), "test.txt", content);

    let tool = ReadFileTool;
    let args = ReadFileArgs {
        file_path: file_path.clone(),
    };

    let result: Result<_, FileToolError> = tool.call(args).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.content, content);
    assert_eq!(output.file_path, file_path);
    assert_eq!(output.size_bytes, content.len() as u64);
    assert!(output.message.contains("Successfully read"));
}

#[tokio::test]
async fn test_read_nonexistent_file() {
    let tool = ReadFileTool;
    let args = ReadFileArgs {
        file_path: "nonexistent_file.txt".to_string(),
    };

    let result: Result<_, FileToolError> = tool.call(args).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        FileToolError::FileNotFound(path) => {
            assert_eq!(path, "nonexistent_file.txt");
        }
        _ => panic!("Expected FileNotFound error"),
    }
}
