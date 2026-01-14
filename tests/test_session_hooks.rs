use kota::hooks::SessionIdHook;
use rig::agent::CancelSignal;
use rig::agent::StreamingPromptHook;
use rig::completion::Message;

#[tokio::test]
async fn test_session_hook_creation() {
    let hook = SessionIdHook::new("test_session".to_string());
    assert_eq!(hook.session_id, "test_session");
    assert!(hook.enable_logging);
}

#[tokio::test]
async fn test_session_hook_with_logging_disabled() {
    let hook = SessionIdHook::new("test_session".to_string()).with_logging(false);
    assert!(!hook.enable_logging);
}

#[tokio::test]
async fn test_session_hook_clone() {
    let hook1 = SessionIdHook::new("test_session".to_string()).with_logging(false);
    let hook2 = hook1.clone();

    assert_eq!(hook1.session_id, hook2.session_id);
    assert_eq!(hook1.enable_logging, hook2.enable_logging);
}
