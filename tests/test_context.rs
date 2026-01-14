use kota::context::ContextManager;
use rig::completion::Message;
use tempfile::TempDir;

#[test]
fn test_context_manager_basic_operations() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = ContextManager::new(temp_dir.path(), "test_session".to_string()).unwrap();

    // 测试添加消息
    manager.add_message(Message::user("Hello"));
    manager.add_message(Message::assistant("Hi there!"));

    assert_eq!(manager.get_messages().len(), 2);

    // 测试保存和加载
    manager.save().unwrap();

    let mut new_manager = ContextManager::new(temp_dir.path(), "test_session".to_string()).unwrap();
    assert!(new_manager.load().unwrap());
    assert_eq!(new_manager.get_messages().len(), 2);
}

#[test]
fn test_context_manager_max_messages() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = ContextManager::new(temp_dir.path(), "test_session".to_string())
        .unwrap()
        .with_max_messages(2);

    // 添加3条消息，应该只保留最后2条
    manager.add_message(Message::user("Message 1"));
    manager.add_message(Message::user("Message 2"));
    manager.add_message(Message::user("Message 3"));

    assert_eq!(manager.get_messages().len(), 2);
}
