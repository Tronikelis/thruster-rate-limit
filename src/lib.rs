#![allow(clippy::needless_return)]

use thruster::{
    context::typed_hyper_context::TypedHyperContext, errors::ThrusterError, middleware_fn, Context,
    ContextState, MiddlewareNext, MiddlewareResult,
};

pub mod stores;
use stores::Store;

mod rate_limiter;
pub use rate_limiter::{Options, RateLimiter};

mod utils;

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

#[middleware_fn]
pub async fn rate_limit_middleware<
    T: Send + Sync + ContextState<RateLimiter<S>> + ContextState<Box<C>>,
    S: 'static + Store + Send + Sync + Clone,
    C: 'static + Configuration<T> + Sync,
>(
    mut context: TypedHyperContext<T>,
    next: MiddlewareNext<TypedHyperContext<T>>,
) -> MiddlewareResult<TypedHyperContext<T>> {
    #[allow(clippy::borrowed_box)]
    let configuration: &Box<_> = context.extra.get();

    if !configuration.should_limit(&context) {
        return next(context).await;
    }

    let rate_limiter: &RateLimiter<S> = context.extra.get();
    let RateLimiter { mut store, .. } = rate_limiter.clone();

    let (path, options) = match rate_limiter.matches_route(context.route()) {
        Some(x) => x,
        None => ("".to_string(), rate_limiter.options.clone()),
    };

    let key = format!("rate-limit:{}:{}", configuration.get_key(&context), path);

    let current_count: Option<usize> = store.get(&key).await.unwrap();

    let current_count = current_count.unwrap_or(0);
    let new_count = current_count + 1;

    if new_count > options.max {
        context.status(429);
        return Err(ThrusterError {
            cause: None,
            context,
            message: format!("Rate limit exceeded, please wait {} seconds", options.per_s),
        });
    }

    store.set(&key, new_count, options.per_s).await.unwrap();

    return next(context).await;
}
