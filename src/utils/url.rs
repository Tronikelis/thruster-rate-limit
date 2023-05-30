/// converts a url into chunks delimited by "/"
/// so /a/:foo becomes (a, :foo)
pub fn to_path_chunks(url: &str) -> Option<Vec<&str>> {
    let path = match url.split('?').next() {
        Some(path) => path,
        None => return None,
    };

    return Some(path.split('/').filter(|x| **x != *"").collect());
}
