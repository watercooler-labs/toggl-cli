use super::*;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};

// These tests assert response semantics, not latency. Emulated/native VM CI can
// be much slower than loopback on the developer's host, especially in parallel.
const TEST_TIMEOUT: Duration = Duration::from_secs(60);

struct Server {
    url: String,
    requests: JoinHandle<Vec<String>>,
}

impl Server {
    async fn start(
        count: usize,
        respond: impl Fn(&str) -> (u16, &'static str) + Send + 'static,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = tokio::spawn(async move {
            let mut requests = Vec::new();
            for _ in 0..count {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                let header_end = loop {
                    assert_ne!(socket.read_buf(&mut bytes).await.unwrap(), 0);
                    if let Some(end) = bytes.windows(4).position(|s| s == b"\r\n\r\n") {
                        break end + 4;
                    }
                };
                let headers = String::from_utf8_lossy(&bytes[..header_end]);
                let length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                while bytes.len() < header_end + length {
                    assert_ne!(socket.read_buf(&mut bytes).await.unwrap(), 0);
                }
                let request = String::from_utf8(bytes).unwrap();
                let (status, body) = respond(&request);
                let response = format!("HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n{body}", body.len());
                socket.write_all(response.as_bytes()).await.unwrap();
                requests.push(request);
            }
            requests
        });
        Self { url, requests }
    }

    fn client(&self) -> V9ApiClient {
        V9ApiClient {
            http_client: Client::builder()
                .no_proxy()
                .timeout(TEST_TIMEOUT)
                .build()
                .unwrap(),
            base_url: self.url.clone(),
        }
    }

    async fn finish(&mut self) -> Vec<String> {
        tokio::time::timeout(TEST_TIMEOUT, &mut self.requests)
            .await
            .unwrap()
            .unwrap()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.requests.abort();
    }
}

#[tokio::test]
async fn raw_http_errors_keep_status_and_body() {
    for status in [301, 401, 403, 404, 429, 500] {
        let mut server = Server::start(1, move |_| (status, "denied")).await;
        let error = server
            .client()
            .send_raw("/me", "GET", None)
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains(&format!("HTTP {status}")), "{error}");
        assert!(error.contains("denied"), "{error}");
        server.finish().await;
    }
}

#[tokio::test]
async fn raw_empty_and_non_json_responses_are_unambiguous() {
    for (status, body, expected) in [
        (204, "", "empty"),
        (200, "null", "null"),
        (200, "<html>error</html>", "error"),
        (404, "", "error"),
    ] {
        let mut server = Server::start(1, move |_| (status, body)).await;
        let result = server.client().send_raw("/me", "GET", None).await;
        match expected {
            "empty" => assert_eq!(result.unwrap(), None),
            "null" => assert_eq!(result.unwrap(), Some(Value::Null)),
            _ => assert!(result.is_err()),
        }
        server.finish().await;
    }
}

#[tokio::test]
async fn raw_requests_send_method_and_typed_json() {
    let mut server = Server::start(1, |_| (200, "{\"id\":42}")).await;
    let body = serde_json::json!({"billable":true,"rate":55.5,"name":"test"});
    assert_eq!(
        server
            .client()
            .send_raw("/projects", "post", Some(body.clone()))
            .await
            .unwrap(),
        Some(serde_json::json!({"id":42}))
    );
    let requests = server.finish().await;
    assert!(requests[0].starts_with("POST /projects HTTP/1.1\r\n"));
    let received: Value =
        serde_json::from_str(requests[0].split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(received, body);
}

#[tokio::test]
async fn typed_requests_check_status_before_deserializing() {
    const USER: &str = r#"{"api_token":"dummy","email":"test@example.com","timezone":"UTC","default_workspace_id":1}"#;
    let mut server = Server::start(1, |_| (401, USER)).await;
    let error = server.client().get_user().await.unwrap_err().to_string();
    assert!(error.contains("HTTP 401"));
    server.finish().await;
}

#[tokio::test]
async fn deletes_accept_empty_success_and_reject_http_errors() {
    for status in [204, 403] {
        let mut server = Server::start(1, move |_| (status, "")).await;
        assert_eq!(
            server.client().delete_time_entry(1, 42).await.is_ok(),
            status == 204
        );
        let requests = server.finish().await;
        assert!(requests[0].starts_with("DELETE /workspaces/1/time_entries/42 "));
    }
}

#[tokio::test]
async fn entity_fetch_failures_are_not_empty_successes() {
    for path in [
        "/me/time_entries",
        "/me/projects",
        "/me/tasks",
        "/me/clients",
        "/me/workspaces",
    ] {
        let mut server = Server::start(5, move |request| {
            if request.split_whitespace().nth(1) == Some(path) {
                (503, "unavailable")
            } else {
                (200, "[]")
            }
        })
        .await;
        assert!(server
            .client()
            .get_entities()
            .await
            .unwrap_err()
            .to_string()
            .contains("HTTP 503"));
        server.finish().await;
    }
}

#[tokio::test]
async fn filtered_fetch_failures_propagate_and_dates_are_encoded() {
    let mut server = Server::start(4, |request| {
        if request.starts_with("GET /me/time_entries?") {
            (429, "rate limited")
        } else {
            (200, "[]")
        }
    })
    .await;
    assert!(server
        .client()
        .get_time_entries_filtered(Some("2026-11-20&injected=1".into()), None)
        .await
        .is_err());
    let requests = server.finish().await;
    assert!(requests
        .iter()
        .any(|r| r.starts_with("GET /me/time_entries?start_date=2026-11-20%26injected%3D1 ")));
}

#[tokio::test]
async fn tasks_with_missing_projects_do_not_panic() {
    let mut server = Server::start(5, |request| {
        if request.starts_with("GET /me/tasks ") {
            (
                200,
                r#"[{"id":1,"name":"orphan","workspace_id":1,"project_id":99}]"#,
            )
        } else {
            (200, "[]")
        }
    })
    .await;
    assert!(server
        .client()
        .get_entities()
        .await
        .unwrap()
        .tasks
        .is_empty());
    server.finish().await;
}
