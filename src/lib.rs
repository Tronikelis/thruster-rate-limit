use thruster::{
    context::typed_hyper_context::TypedHyperContext, errors::ThrusterError, middleware_fn,
    ContextState, MiddlewareNext, MiddlewareResult,
};

pub mod stores;
use stores::Store;

#[derive(Clone)]
pub struct RateLimiter<T: Store + Clone + Sync> {
    pub max: usize,
    pub per_s: usize,
    pub store: T,
}

#[middleware_fn]
pub async fn rate_limit_middleware<
    T: Send + ContextState<RateLimiter<G>>,
    G: 'static + Store + Send + Sync + Clone,
>(
    mut context: TypedHyperContext<T>,
    next: MiddlewareNext<TypedHyperContext<T>>,
) -> MiddlewareResult<TypedHyperContext<T>> {
    let rate_limiter: &mut RateLimiter<G> = context.extra.get_mut();
    let RateLimiter { store, max, per_s } = rate_limiter;

    let key = "rate_limit:".to_string()
        + &context
            .hyper_request
            .as_ref()
            .unwrap()
            .ip
            .unwrap()
            .to_string();

    let current_count: Option<usize> = store.get(&key).await.unwrap();

    let current_count = current_count.unwrap_or(0);
    let new_count = current_count + 1;

    if new_count > *max {
        context.status(429);
        return Err(ThrusterError {
            cause: None,
            context,
            message: "Rate limit exceeded".to_string(),
        });
    }

    store.set(&key, new_count, *per_s).await.unwrap();

    return next(context).await;
}