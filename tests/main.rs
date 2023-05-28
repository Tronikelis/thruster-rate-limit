#![allow(clippy::needless_return)]

use std::sync::Arc;
use thruster::{
    context::typed_hyper_context::TypedHyperContext, context_state, m, middleware_fn, testing, App,
    HyperRequest, MiddlewareNext, MiddlewareResult,
};
use thruster_jab::{provide, JabDI};

use thruster_rate_limit::{
    rate_limit_middleware, stores::map::MapStore, Configuration, RateLimiter,
};

struct ServerState {
    rate_limiter: RateLimiter<MapStore>,
    di: Arc<JabDI>,
}

#[context_state]
struct RequestState(Arc<JabDI>, RateLimiter<MapStore>);
type Ctx = TypedHyperContext<RequestState>;

#[middleware_fn]
async fn root(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body("foo");
    return Ok(context);
}

fn generate_context(request: HyperRequest, state: &ServerState, _path: &str) -> Ctx {
    return Ctx::new(
        request,
        RequestState(state.di.clone(), state.rate_limiter.clone()),
    );
}

#[tokio::test]
async fn hello_world() {
    let mut di = JabDI::default();

    let rate_limiter = RateLimiter {
        max: 100,
        per_s: 60,
        store: MapStore::new(),
    };

    struct RateLimiterConfiguration;
    impl Configuration<RequestState> for RateLimiterConfiguration {}

    provide!(
        di,
        dyn Configuration<RequestState> + Send + Sync,
        RateLimiterConfiguration
    );

    let mut app = App::<HyperRequest, Ctx, ServerState>::create(
        generate_context,
        ServerState {
            rate_limiter,
            di: Arc::new(di),
        },
    )
    .middleware("/", m![rate_limit_middleware])
    .get("/", m![root]);

    let result = testing::get(app, "/");
}
