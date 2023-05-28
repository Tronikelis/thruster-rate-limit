#![allow(clippy::needless_return)]

use thruster::{
    context::typed_hyper_context::TypedHyperContext, errors::ThrusterError, middleware_fn, Context,
    ContextState, MiddlewareNext, MiddlewareResult,
};

pub mod stores;
use stores::Store;

#[derive(Clone)]
pub struct RateLimiter<S: Store + Clone> {
    pub max: usize,
    pub per_s: usize,
    pub store: S,
}

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
    let RateLimiter {
        mut store,
        max,
        per_s,
    } = rate_limiter.clone();

    let key = "rate-limit:".to_string() + &configuration.get_key(&context);

    let current_count: Option<usize> = store.get(&key).await.unwrap();

    let current_count = current_count.unwrap_or(0);
    let new_count = current_count + 1;

    if new_count > max {
        context.status(429);
        return Err(ThrusterError {
            cause: None,
            context,
            message: format!("Rate limit exceeded, please wait {} seconds", per_s),
        });
    }

    store.set(&key, new_count, per_s).await.unwrap();

    return next(context).await;
}
