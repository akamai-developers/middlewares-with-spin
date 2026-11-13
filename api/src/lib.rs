use anyhow::Context;
use spin_sdk::http::body::IncomingBodyExt;
use spin_sdk::http::{EmptyBody, IntoResponse, Method, Request, Response};
use spin_sdk::{http_service, variables};
use std::time::{SystemTime, UNIX_EPOCH};

#[http_service]
async fn handle_api(req: Request) -> anyhow::Result<impl IntoResponse> {
    let body = match (req.method(), req.uri().path()) {
        (&Method::GET, "/api" | "/api/") => serde_json::json!({
            "message": "Hello from Rust on Spin",
            "at": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        })
        .to_string(),
        (&Method::GET, "/api/posts") => {
            let upstream_origin = variables::get("sample_data_origin")
                .await
                .context("Invalid App Configuration")?;
            let upstream_req_url = format!("{}/posts", upstream_origin);
            let upstream = Request::get(upstream_req_url).body(EmptyBody::new())?;
            let res = spin_sdk::http::send(upstream).await?;
            let posts = res.into_body().bytes().await?;
            String::from_utf8(posts.to_vec())?
        }
        _ => return Ok(Response::builder().status(404).body(String::new())?),
    };

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("content-length", body.len())
        .body(body)?)
}
