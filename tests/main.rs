#![allow(clippy::needless_return)]

use std::sync::Arc;
use thruster::{
    context::typed_hyper_context::TypedHyperContext, context_state, m, middleware_fn, testing, App,
    HyperRequest, MiddlewareNext, MiddlewareResult,
};
use thruster_jab::JabDI;

use thruster_rate_limit::{rate_limit_middleware, stores::map::MapStore, RateLimiter};

struct ServerState {
    di: Arc<JabDI>,
    rate_limiter: RateLimiter<MapStore>,
}

#[context_state]
struct RequestState(RateLimiter<MapStore>, Arc<JabDI>);
type Ctx = TypedHyperContext<RequestState>;

#[middleware_fn]
async fn root(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body("foo");
    return Ok(context);
}

fn generate_context(request: HyperRequest, state: &ServerState, _path: &str) -> Ctx {
    return Ctx::new(
        request,
        RequestState(state.rate_limiter.clone(), state.di.clone()),
    );
}

#[tokio::test]
async fn hello_world() {
    let rate_limiter = RateLimiter {
        max: 100,
        per_s: 60,
        store: MapStore::new(),
    };

    let di = Arc::new(JabDI::default());

    let mut app = App::<HyperRequest, Ctx, ServerState>::create(
        generate_context,
        ServerState { di, rate_limiter },
    )
    .middleware("/", m![rate_limit_middleware])
    .get("/", m![root]);

    let result = testing::get(app, "/");
}
