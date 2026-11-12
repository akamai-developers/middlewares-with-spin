use spin_sdk::http::{box_body, BoxBody, EmptyBody, Request, Response};
use spin_sdk::http_service;

#[http_service]
async fn handle_auth(req: Request) -> anyhow::Result<Response<BoxBody>> {
    let token = spin_sdk::variables::get("auth_token").await?;
    let expected = format!("Bearer {token}");

    let presented = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    if !constant_time_eq(presented.as_bytes(), expected.as_bytes()) {
        return Ok(Response::builder()
            .status(401)
            .body(box_body(EmptyBody::new()))?);
    }
    // pass the request along the pipeline
    let res = spin_sdk::http::next(req).await?;
    Ok(res.map(box_body))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}
