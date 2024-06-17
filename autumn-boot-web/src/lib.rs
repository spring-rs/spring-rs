use autumn_boot::App;
use axum::Router;

pub trait WebConfig<T> {
    fn web(&self, router: T) -> Self;
}

impl WebConfig<Router> for App {
    fn web(&self, router: Router) -> Self {
        todo!()
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
