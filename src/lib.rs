use if_chain::if_chain;
use std::sync::Arc;
use thruster::{
    context::typed_hyper_context::TypedHyperContext, errors::ThrusterError, middleware_fn, Context,
    ContextState, MiddlewareNext, MiddlewareResult,
};
use thruster_jab::{fetch, JabDI};

pub mod stores;
use stores::Store;

#[derive(Clone)]
pub struct RateLimiter<S: Store + Clone + Sync> {
    pub max: usize,
    pub per_s: usize,
    pub store: S,
}

pub trait Configuration<C: Send> {
    fn should_limit(&self, _context: &TypedHyperContext<C>) -> bool {
        return true;
    }
    fn get_key(&self, context: &TypedHyperContext<C>) -> String {
        if_chain! {
            if let Some(request) = context.hyper_request.as_ref();
            if let Some(ip) = request.ip;
            then {
                return ip.to_string();
            }
        }

        return "".to_string();
    }
}

#[middleware_fn]
pub async fn rate_limit_middleware<
    C: Send + ContextState<RateLimiter<S>> + ContextState<Arc<JabDI>>,
    S: 'static + Store + Send + Sync + Clone,
>(
    mut context: TypedHyperContext<C>,
    next: MiddlewareNext<TypedHyperContext<C>>,
) -> MiddlewareResult<TypedHyperContext<C>> {
    let di: &Arc<JabDI> = context.extra.get();
    let configuration = fetch!(di, dyn Configuration<C> + Sync);

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
