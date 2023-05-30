use crate::{stores::map::MapStore, utils::url::to_path_chunks, Store};

#[derive(Clone)]
pub struct Options {
    pub max: usize,
    pub per_s: usize,
}

impl Options {
    pub fn new(max: usize, per_s: usize) -> Self {
        return Self { max, per_s };
    }
}

#[derive(Clone)]
pub struct RateLimiter<S: Store + Clone> {
    pub options: Options,
    pub routes: Vec<(String, Options)>,
    pub store: S,
}

impl<S: Store + Clone> RateLimiter<S> {
    pub fn new(options: Options, store: S) -> Self {
        return RateLimiter {
            options,
            routes: vec![],
            store,
        };
    }

    pub fn override_routes(mut self, routes: Vec<(String, Options)>) -> Self {
        self.routes = routes;
        return self;
    }

    pub fn matches_route(&self, target: &str) -> Option<(String, Options)> {
        'outer: for (path, options) in &self.routes {
            let target_parts: Vec<_> = to_path_chunks(target).unwrap();
            let self_parts: Vec<_> = to_path_chunks(path).unwrap();

            if target_parts.len() != self_parts.len() {
                continue;
            }

            for (i, self_part) in self_parts.iter().enumerate() {
                if self_part.starts_with(':') {
                    continue;
                }

                if self_part != target_parts.get(i).unwrap() {
                    continue 'outer;
                }
            }

            return Some((path.clone(), options.clone()));
        }

        return None;
    }
}

impl Default for RateLimiter<MapStore> {
    fn default() -> Self {
        return Self::new(
            Options {
                max: 200,
                per_s: 60,
            },
            MapStore::new(),
        );
    }
}
