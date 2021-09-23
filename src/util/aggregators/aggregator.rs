pub trait Aggregator<T> {
    fn new() -> Self;
    fn update(&mut self, message: T);
    fn messages(&self, n: usize) -> Vec<String>;
}
