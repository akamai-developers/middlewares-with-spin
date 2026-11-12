use async_compression::futures::bufread::GzipEncoder;
use futures::{AsyncReadExt, SinkExt, TryStreamExt};
use spin_sdk::http::body::IncomingBodyExt;
use spin_sdk::http::{box_body, BoxBody, Request, Response};
use spin_sdk::http_service;

#[http_service]
async fn handle_gzip(req: Request) -> anyhow::Result<Response<BoxBody>> {
    let min_size: usize = spin_sdk::variables::get("gzip_min_size")
        .await?
        .parse()
        .unwrap_or(1024);

    let accepts_gzip = req
        .headers()
        .get("accept-encoding")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("gzip"));

    // pass the request along the pipeline
    let res = spin_sdk::http::next(req).await?;

    let too_small = res
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .is_some_and(|len| len < min_size);

    if !accepts_gzip || too_small {
        return Ok(res.map(box_body));
    }

    let (mut parts, body) = res.into_parts();

    // read the response body as a stream and gzip it chunk by chunk
    let reader = body.stream().map_err(std::io::Error::other).into_async_read();
    let mut encoder = GzipEncoder::new(reader);

    let (mut tx, compressed) = spin_sdk::http::body::stream::<Vec<u8>>();
    spin_sdk::wasip3::spawn(async move {
        let mut buf = vec![0u8; 8 * 1024];
        loop {
            match encoder.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    parts.headers.remove("content-length");
    parts.headers.insert("content-encoding", "gzip".parse()?);
    parts.headers.insert("vary", "accept-encoding".parse()?);
    Ok(Response::from_parts(parts, box_body(compressed)))
}
