use super::{create_temp_dir, create_test_file};
use kota::tools::edit_file::{EditFileArgs, EditFileTool};
use rig::tool::Tool;
use std::fs;

#[tokio::test]
async fn test_edit_file_simple_replacement() {
    let temp_dir = create_temp_dir();
    let file_path = create_test_file(temp_dir.path(), "test.txt", "line 1\nline 2\nline 3\n");

    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: file_path.clone(),
        patch: "--- a/test.txt\n+++ a/test.txt\n@@ -1,3 +1,3 @@\n line 1\n-line 2\n+modified line 2\n line 3\n".to_string(),
    };

    let result = tool.call(args).await.unwrap();

    assert!(result.success);
    assert_eq!(result.lines_added, 1);
    assert_eq!(result.lines_removed, 1);
    assert_eq!(result.file_path, file_path);

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "line 1\nmodified line 2\nline 3");
}

#[tokio::test]
async fn test_edit_file_add_lines() {
    let temp_dir = create_temp_dir();
    let file_path = create_test_file(temp_dir.path(), "test.txt", "line 1\nline 2\n");

    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: file_path.clone(),
        patch:
            "--- a/test.txt\n+++ b/test.txt\n@@ -1,2 +1,4 @@\n line 1\n line 2\n+line 3\n+line 4\n"
                .to_string(),
    };

    let result = tool.call(args).await.unwrap();

    assert!(result.success);
    assert_eq!(result.lines_added, 2);
    assert_eq!(result.lines_removed, 0);

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "line 1\nline 2\nline 3\nline 4");
}

#[tokio::test]
async fn test_edit_file_remove_lines() {
    let temp_dir = create_temp_dir();
    let file_path = create_test_file(
        temp_dir.path(),
        "test.txt",
        "line 1\nline 2\nline 3\nline 4\n",
    );

    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: file_path.clone(),
        patch:
            "--- a/test.txt\n+++ b/test.txt\n@@ -1,4 +1,2 @@\n line 1\n-line 2\n-line 3\n line 4\n"
                .to_string(),
    };

    let result = tool.call(args).await.unwrap();

    assert!(result.success);
    assert_eq!(result.lines_added, 0);
    assert_eq!(result.lines_removed, 2);

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "line 1\nline 4");
}

#[tokio::test]
async fn test_edit_file_complex_patch() {
    let temp_dir = create_temp_dir();
    let file_path = create_test_file(
        temp_dir.path(),
        "code.rs",
        "fn main() {\n    println!(\"Hello\");\n}\n",
    );

    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: file_path.clone(),
        patch: "--- a/code.rs\n+++ b/code.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    // Added comment\n     println!(\"Hello\");\n+    println!(\"World\");\n }".to_string(),
    };

    let result = tool.call(args).await.unwrap();

    assert!(result.success);
    assert_eq!(result.lines_added, 2);
    assert_eq!(result.lines_removed, 0);

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(
        content,
        "fn main() {\n    // Added comment\n    println!(\"Hello\");\n    println!(\"World\");\n}"
    );
}

#[tokio::test]
async fn test_edit_file_nonexistent_file() {
    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: "nonexistent.txt".to_string(),
        patch: "--- a/nonexistent.txt\n+++ b/nonexistent.txt\n@@ -1,1 +1,1 @@\n-old\n+new\n"
            .to_string(),
    };

    let result = tool.call(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_edit_file_directory_instead_of_file() {
    let temp_dir = create_temp_dir();
    let dir_path = temp_dir.path().join("subdir");
    fs::create_dir(&dir_path).unwrap();

    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: dir_path.to_string_lossy().to_string(),
        patch: "@@ -1,1 +1,1 @@\n-old\n+new\n".to_string(),
    };

    let result = tool.call(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_edit_file_invalid_patch_format() {
    let temp_dir = create_temp_dir();
    let file_path = create_test_file(temp_dir.path(), "test.txt", "line 1\nline 2\n");

    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: file_path.clone(),
        patch: "invalid patch format".to_string(),
    };

    let result = tool.call(args).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_edit_file_empty_file() {
    let temp_dir = create_temp_dir();
    let file_path = create_test_file(temp_dir.path(), "empty.txt", "");

    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: file_path.clone(),
        patch: "--- a/empty.txt\n+++ b/empty.txt\n@@ -0,0 +1,1 @@\n+new line\n".to_string(),
    };

    let result = tool.call(args).await.unwrap();

    assert!(result.success);
    assert_eq!(result.lines_added, 1);
    assert_eq!(result.lines_removed, 0);

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "new line");
}

#[tokio::test]
async fn test_edit_file_single_line_replacement() {
    let temp_dir = create_temp_dir();
    let file_path = create_test_file(temp_dir.path(), "single.txt", "old content\n");

    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: file_path.clone(),
        patch: "--- a/single.txt\n+++ b/single.txt\n@@ -1,1 +1,1 @@\n-old content\n+new content\n"
            .to_string(),
    };

    let result = tool.call(args).await.unwrap();

    assert!(result.success);
    assert_eq!(result.lines_added, 1);
    assert_eq!(result.lines_removed, 1);

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "new content");
}

#[tokio::test]
async fn test_edit_file_multiline_context() {
    let temp_dir = create_temp_dir();
    let file_path = create_test_file(
        temp_dir.path(),
        "context.txt",
        "line 1\nline 2\nline 3\nline 4\nline 5\n",
    );

    let tool = EditFileTool;
    let args = EditFileArgs {
        file_path: file_path.clone(),
        patch: "--- a/context.txt\n+++ b/context.txt\n@@ -2,3 +2,3 @@\n line 2\n-line 3\n+modified line 3\n line 4\n".to_string(),
    };

    let result = tool.call(args).await.unwrap();

    assert!(result.success);
    assert_eq!(result.lines_added, 1);
    assert_eq!(result.lines_removed, 1);

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "line 1\nline 2\nmodified line 3\nline 4\nline 5\n");
}

#[tokio::test]
async fn test_edit_file_tool_definition() {
    let tool = EditFileTool;
    let definition = tool.definition("test prompt".to_string()).await;

    assert_eq!(definition.name, "edit_file");
    assert!(definition.description.contains("unified diff patch"));
    assert!(definition.parameters.get("properties").is_some());
}
