# middlewares-in-spin

This sample shows HTTP middleware in Spin 4.3: a small Rust API on `/api/...` is protected by an `auth` middleware and its responses are compressed by a `gzip` middleware. Both middlewares are standalone WebAssembly Components wired up in `spin.toml`, so the API itself knows nothing about authentication or compression.

## Prerequisites

- Spin CLI (version `4.3.0` or newer)
- Rust with the `wasm32-wasip2` target (`rustup target add wasm32-wasip2`)

## Build And Run

```bash
spin build
spin up
```

The application listens on `http://localhost:3000`.

## Test It

A request without a token is rejected by the `auth` middleware with a `401`:

```bash
curl -i localhost:3000/api/
```

With the token, the API answers with a `200` and a small JSON object:

```bash
curl -i -H 'Authorization: Bearer 6f1c2e9a-4b7d-4c3e-9a1f-2d8e5b0c7a64' localhost:3000/api/
```

Asking for gzip on the larger `/api/posts` response makes the `gzip` middleware compress it (`content-encoding: gzip`):

```bash
curl -s -D - -o /dev/null -H 'Accept-Encoding: gzip' -H 'Authorization: Bearer 6f1c2e9a-4b7d-4c3e-9a1f-2d8e5b0c7a64' localhost:3000/api/posts
```

The token is the default value of the `auth_token` variable in `spin.toml`.
