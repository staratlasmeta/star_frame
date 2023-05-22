/// Trait for implementing Flat maps for option types
pub trait OptionFlatMap {
    /// Data to be operated on
    type Item;
    /// Same as [`Option::map`] followed by [`Option::flatten`]
    fn flat_map<U>(self, function: impl FnOnce(Self::Item) -> Option<U>) -> Option<U>;
}
#[allow(clippy::map_flatten)]
impl<T> OptionFlatMap for Option<T> {
    type Item = T;
    fn flat_map<U>(self, function: impl FnOnce(Self::Item) -> Option<U>) -> Option<U> {
        self.map(function).flatten()
    }
}
