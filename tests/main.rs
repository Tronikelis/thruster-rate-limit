#![allow(clippy::needless_return)]

use thruster::{
    context::typed_hyper_context::TypedHyperContext, context_state, m, middleware_fn, App,
    HyperRequest, MiddlewareNext, MiddlewareResult, Testable,
};

use thruster_rate_limit::{
    rate_limit_middleware, stores::map::MapStore, Configuration, RateLimiter,
};

const BODY_STR: &str = "foo";
const ROUTE: &str = "/foo";

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

fn create_app(server_state: ServerState) -> App<HyperRequest, Ctx, ServerState> {
    return App::<HyperRequest, Ctx, ServerState>::create(generate_context, server_state)
        .middleware("/", m![rate_limit_middleware]);
}

#[tokio::test]
async fn hello_world() {
    let rate_limiter = RateLimiter {
        max: 100,
        per_s: 60,
        routes: vec![],
        store: MapStore::new(),
    };

    let app = create_app(ServerState { rate_limiter })
        .get(ROUTE, m![root])
        .commit();

    let response = Testable::get(&app, ROUTE, vec![])
        .await
        .unwrap()
        .expect_status(200, "OK");

    assert_eq!(response.body_string(), BODY_STR);
}

#[tokio::test]
async fn simple_block() {
    let rate_limiter = RateLimiter {
        max: 10,
        per_s: 100,
        routes: vec![],
        store: MapStore::new(),
    };

    let app = create_app(ServerState { rate_limiter })
        .get(ROUTE, m![root])
        .commit();

    for _ in 0..10 {
        Testable::get(&app, ROUTE, vec![])
            .await
            .unwrap()
            .expect_status(200, "OK");
    }

    Testable::get(&app, ROUTE, vec![])
        .await
        .unwrap()
        .expect_status(429, "OK");
}

#[tokio::test]
async fn routes_option() {
    let rate_limiter = RateLimiter {
        max: 1,
        per_s: 100,
        routes: vec![("/foo".to_string(), 10), ("/user/:id".to_string(), 10)],
        store: MapStore::new(),
    };

    let app = create_app(ServerState { rate_limiter })
        .get("/", m![root])
        .get("/foo", m![root])
        .get("/user/:id", m![root])
        .commit();

    for i in 0..11 {
        Testable::get(&app, "/foo?q=q", vec![])
            .await
            .unwrap()
            .expect_status(if i == 10 { 429 } else { 200 }, "OK");
    }

    for i in 0..11 {
        Testable::get(&app, "/user/0", vec![])
            .await
            .unwrap()
            .expect_status(if i == 10 { 429 } else { 200 }, "OK");
    }
}
