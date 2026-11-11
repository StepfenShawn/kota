use std::net::SocketAddr;
use std::time::Duration;

use kota::kota_code::llm::{CompletionResponse, Message, OpenAiClient};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::time::timeout;

// HTTP chunk framing, rather than sleeps or individual TCP writes, defines the
// fragmentation. A one-byte body chunk cuts every two-, three- and four-byte
// UTF-8 character, even when the entire response arrives in one socket read.
async fn serve_chunked_sse(body: &str, chunk_size: usize) -> (SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mut wire = b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
    for chunk in body.as_bytes().chunks(chunk_size) {
        wire.extend_from_slice(format!("{:x}\r\n", chunk.len()).as_bytes());
        wire.extend_from_slice(chunk);
        wire.extend_from_slice(b"\r\n");
    }
    wire.extend_from_slice(b"0\r\n\r\n");

    let server = tokio::spawn(async move {
        timeout(Duration::from_secs(10), async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buf = [0; 4096];
            let header_end = loop {
                let count = socket.read(&mut buf).await.unwrap();
                assert_ne!(count, 0, "request ended before its headers");
                request.extend_from_slice(&buf[..count]);
                assert!(request.len() <= 32 * 1024, "fixture request too large");
                if let Some(at) = request.windows(4).position(|s| s == b"\r\n\r\n") {
                    break at + 4;
                }
            };
            let headers = std::str::from_utf8(&request[..header_end]).unwrap();
            let length: usize = headers
                .lines()
                .filter_map(|line| line.split_once(':'))
                .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                .expect("fixture expects a fixed-length request")
                .1
                .trim()
                .parse()
                .unwrap();
            assert!(header_end + length <= 32 * 1024);
            while request.len() < header_end + length {
                let count = socket.read(&mut buf).await.unwrap();
                assert_ne!(count, 0, "request ended before its body");
                request.extend_from_slice(&buf[..count]);
            }
            socket.write_all(&wire).await.unwrap();
            socket.shutdown().await.unwrap();
        })
        .await
        .expect("loopback fixture timed out");
    });
    (addr, server)
}

fn event(value: serde_json::Value) -> String {
    format!("data: {value}\n\n")
}

fn text_fixture() -> String {
    event(json!({"choices": [{"delta": {
        "content": "café 你好 🦀", "reasoning_content": "检查 🦀"
    }}]}))
        + &event(json!({"choices": [{"delta": {}, "finish_reason": "stop"}]}))
        + &event(json!({"choices": [], "usage": {
            "prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5
        }}))
        + "data: [DONE]\n\n"
}

fn tool_fixture() -> String {
    event(json!({"choices": [{"delta": {"tool_calls": [{
        "index": 0, "id": "call_unicode", "type": "function",
        "function": {"name": "read_file", "arguments": "{\"file_path\":\"资"}
    }]}}]}))
        + &event(json!({"choices": [{"delta": {"tool_calls": [{
            "index": 0, "function": {"arguments": "料/🦀.txt\",\"label\":\"café\"}"}
        }]}, "finish_reason": "tool_calls"}]}))
        + "data: [DONE]\n\n"
}

async fn stream_fixture(body: &str, chunk_size: usize) -> (CompletionResponse, String, String) {
    let (addr, server) = serve_chunked_sse(body, chunk_size).await;
    let client = OpenAiClient::new(
        "test-key".to_string(),
        "test-model".to_string(),
        format!("http://{addr}"),
        None,
    )
    .unwrap();
    let mut text = String::new();
    let mut reasoning = String::new();
    let response = timeout(
        Duration::from_secs(10),
        client.stream_completion(
            &[Message::user("hi")],
            &[],
            |token| text.push_str(token),
            |token| reasoning.push_str(token),
        ),
    )
    .await
    .expect("stream timed out")
    .unwrap();
    server.await.unwrap();
    (response, text, reasoning)
}

fn assert_text(response: CompletionResponse, text: String, reasoning: String) {
    assert_eq!(text, "café 你好 🦀", "token callback must preserve UTF-8");
    assert_eq!(response.choices[0].message.text_content(), text);
    assert_eq!(
        reasoning, "检查 🦀",
        "reasoning callback must preserve UTF-8"
    );
    assert_eq!(response.choices[0].finish_reason.as_deref(), Some("stop"));
    assert_eq!(response.usage.total_tokens, 5);
}

fn assert_tool(response: CompletionResponse) {
    let choice = &response.choices[0];
    assert_eq!(choice.finish_reason.as_deref(), Some("tool_calls"));
    let calls = choice.message.tool_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].id, "call_unicode");
    assert_eq!(calls[0].call_type, "function");
    assert_eq!(calls[0].function.name, "read_file");
    assert_eq!(
        calls[0].function.arguments, "{\"file_path\":\"资料/🦀.txt\",\"label\":\"café\"}",
        "tool arguments must preserve UTF-8 across transport and delta boundaries"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&calls[0].function.arguments).unwrap(),
        json!({"file_path": "资料/🦀.txt", "label": "café"})
    );
}

#[tokio::test]
async fn test_chunked_fixture_really_splits_utf8_bytes() {
    let body = "é你🦀";
    let (addr, server) = serve_chunked_sse(body, 1).await;
    let mut response = reqwest::Client::new()
        .post(format!("http://{addr}/chat/completions"))
        .body("{}")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .unwrap();
    let mut received = Vec::new();
    let mut split_codepoint = false;
    while let Some(chunk) = response.chunk().await.unwrap() {
        split_codepoint |= std::str::from_utf8(&chunk).is_err();
        received.extend_from_slice(&chunk);
    }
    server.await.unwrap();
    assert_eq!(received, body.as_bytes());
    assert!(
        split_codepoint,
        "fixture must expose an incomplete UTF-8 chunk"
    );
}

#[tokio::test]
async fn test_stream_completion_unicode_in_whole_http_chunks() {
    let body = text_fixture();
    let (response, text, reasoning) = stream_fixture(&body, body.len()).await;
    assert_text(response, text, reasoning);
    let body = tool_fixture();
    let (response, _, _) = stream_fixture(&body, body.len()).await;
    assert_tool(response);
}

// Regression cases requested in #6. The whole-chunk control above uses the
// identical payloads; only HTTP body chunk boundaries differ.
#[tokio::test]
async fn test_stream_completion_preserves_fragmented_utf8_text() {
    let (response, text, reasoning) = stream_fixture(&text_fixture(), 1).await;
    assert_text(response, text, reasoning);
}

#[tokio::test]
async fn test_stream_completion_preserves_fragmented_utf8_tool_arguments() {
    let (response, _, _) = stream_fixture(&tool_fixture(), 1).await;
    assert_tool(response);
}
