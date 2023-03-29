pub trait Truncate {
    fn truncate(&self, max: usize) -> &Self;
}

impl Truncate for str {
    fn truncate(&self, max: usize) -> &Self {
        self.char_indices()
            .nth(max)
            .map_or(self, |(index, _)| &self[..index])
    }
}
