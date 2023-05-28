#![allow(clippy::needless_return)]

use thruster::{
    context::typed_hyper_context::TypedHyperContext, context_state, m, middleware_fn, App,
    HyperRequest, MiddlewareNext, MiddlewareResult, Testable,
};

use thruster_rate_limit::{
    rate_limit_middleware, stores::map::MapStore, Configuration, RateLimiter,
};

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
    context.body("foo");
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
    let rate_limiter = RateLimiter {
        max: 100,
        per_s: 60,
        store: MapStore::new(),
    };

    let app = App::<HyperRequest, Ctx, ServerState>::create(
        generate_context,
        ServerState { rate_limiter },
    )
    .middleware("/", m![rate_limit_middleware])
    .get("/", m![root])
    .commit();

    let response = Testable::get(&app, "/", vec![])
        .await
        .unwrap()
        .expect_status(200, "OK");

    assert_eq!(response.body_string(), "foo");
}
