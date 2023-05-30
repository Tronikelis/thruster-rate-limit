# thruster-rate-limit

A super simple rate limiting middleware for the [thruster](https://github.com/thruster-rs/Thruster) web framework.

Currently supports only the hyper backend of thruster, basically `hyper_server` feature must be enabled!




## Table of Contents

- [thruster-rate-limit](#thruster-rate-limit)
  - [Table of Contents](#table-of-contents)
  - [Simple example](#simple-example)
  - [Options](#options)
  - [Configuration](#configuration)
  - [Stores](#stores)
  - [Different routes, different settings](#different-routes-different-settings)





## Simple example

```rust
struct ServerState {
    rate_limiter: RateLimiter<MapStore>,
}

#[context_state]
struct RequestState(RateLimiter<MapStore>, Box<RateLimiterConf>);
type Ctx = TypedHyperContext<RequestState>;

struct RateLimiterConf;
impl Configuration<RequestState> for RateLimiterConf {}

#[middleware_fn]
async fn root(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body(BODY_STR);
    return Ok(context);
}

fn generate_context(request: HyperRequest, state: &ServerState, _path: &str) -> Ctx {
    return Ctx::new(
        request,
        RequestState(state.rate_limiter.clone(), Box::new(RateLimiterConf)),
    );
}

#[tokio::test]
async fn hello_world() {
    let rate_limiter = RateLimiter::default();

    let app = App::<HyperRequest, Ctx, ServerState>::create(
        generate_context,
        ServerState { rate_limiter },
    )
    .middleware("/", m![rate_limit_middleware])
    .get(ROUTE, m![root])
    .commit();

    let response = Testable::get(&app, ROUTE, vec![])
        .await
        .unwrap()
        .expect_status(200, "OK");

    assert_eq!(response.body_string(), BODY_STR);
}
```



## Options

```rust
#[derive(Clone)]
pub struct Options {
    pub max: usize,
    pub per_s: usize,
}

#[derive(Clone)]
pub struct RateLimiter<S: Store + Clone> {
    pub options: Options,
    pub routes: Vec<(String, Options)>,
    pub store: S,
}
```

- `routes`: apply different options to different routes [more info](#different-routes-different-settings)
- `max`: maximum amount of requests allowed `per_s`
- `per_s`: when does `max` reset
- `store`: anything that implements the `Store` trait, [2 stores](#stores) are provided by the library




## Configuration

This is currently pretty basic, but you can extend the functionality of the rate limiter based on your needs by implementing the `Configuration` trait

```rust
pub trait Configuration<S: Send> {
    fn should_limit(&self, _context: &TypedHyperContext<S>) -> bool {
        return true;
    }
    fn get_key(&self, context: &TypedHyperContext<S>) -> String {
        if let Some(request) = context.hyper_request.as_ref() {
            if let Some(ip) = request.ip {
                return ip.to_string();
            }
        }

        return "".to_string();
    }
}
```



## Stores

Simple in-memory store:
```rust
#[derive(Clone)]
pub struct MapStore {
    hash_map: Arc<Mutex<HashMap<String, MapValue>>>,
}
```

[needs `redis_store` feature] Redis store:
```rust
#[derive(Clone)]
pub struct RedisStore {
    connection_manager: ConnectionManager,
}
```

## Different routes, different settings

Simply put, this is useful when, for example, for a login route you want to have only 5 requests per minute.

```rust
let rate_limiter = RateLimiter::new(Options::new(1, 100), MapStore::new())
    .override_routes(vec![
        // Options::new(max, per_s)
        ("/user/login".to_string(), Options::new(5, 60)),
        // limit some expensive route for example, reset every 2 minutes
        ("/user/:id/calculate".to_string(), Options::new(20, 60 * 2)),
    ]);
```
