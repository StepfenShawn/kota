use kota::kota_code::llm::{Message, OpenAiClient};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn serve_sse(body: &'static str) -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 16 * 1024];
        let _ = socket.read(&mut buf).await.unwrap();

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
    });

    addr
}

#[tokio::test]
async fn test_stream_completion_content_and_usage() {
    let addr = serve_sse(concat!(
        "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\",\"content\":\"Hello\"}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\" world\"},\"finish_reason\":\"stop\"}]}\n\n",
        "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":3,\"completion_tokens\":2,\"total_tokens\":5}}\n\n",
        "data: [DONE]\n\n",
    ))
    .await;

    let client = OpenAiClient::new(
        "test-key".to_string(),
        "test-model".to_string(),
        format!("http://{}", addr),
        None,
    )
    .unwrap();

    let mut streamed = String::new();
    let response = client
        .stream_completion(&[Message::user("hi")], &[], |token| {
            streamed.push_str(token)
        })
        .await
        .unwrap();

    assert_eq!(streamed, "Hello world");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.text_content(), "Hello world");
    assert_eq!(response.usage.total_tokens, 5);
}

#[tokio::test]
async fn test_stream_completion_accumulates_tool_calls() {
    let addr = serve_sse(concat!(
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"read_file\",\"arguments\":\"{\\\"file\"}}]}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"_path\\\":\\\"a.txt\\\"}\"}}]},\"finish_reason\":\"tool_calls\"}]}\n\n",
        "data: [DONE]\n\n",
    ))
    .await;

    let client = OpenAiClient::new(
        "test-key".to_string(),
        "test-model".to_string(),
        format!("http://{}", addr),
        None,
    )
    .unwrap();

    let response = client
        .stream_completion(&[Message::user("read a.txt")], &[], |_| {})
        .await
        .unwrap();

    let message = &response.choices[0].message;
    assert_eq!(message.text_content(), "");
    let calls = message.tool_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].id, "call_1");
    assert_eq!(calls[0].call_type, "function");
    assert_eq!(calls[0].function.name, "read_file");
    assert_eq!(calls[0].function.arguments, "{\"file_path\":\"a.txt\"}");
}
