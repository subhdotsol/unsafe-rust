pub struct MyCell<T> {
    value: T,
}
impl<T> MyCell<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    pub fn replace(&mut self, value: T) -> T {
        std::mem::replace(&mut self.value, value)
    }

    pub fn take(&mut self) -> T {
        std::mem::take(&mut self.value)
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Default for MyCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cell = MyCell::default();
        assert_eq!(cell.get(), &Default::default());
    }

    #[test]
    fn test_new() {
        let cell = MyCell::new(42);
        assert_eq!(cell.get(), &42);
    }

    #[test]
    fn test_set() {
        let mut cell = MyCell::new(42);
        cell.set(43);
        assert_eq!(cell.get(), &43);
    }

    #[test]
    fn test_replace() {
        let mut cell = MyCell::new(42);
        let old = cell.replace(43);
        assert_eq!(old, 42);
        assert_eq!(cell.get(), &43);
    }

    #[test]
    fn test_take() {
        let mut cell = MyCell::new(42);
        let old = cell.take();
        assert_eq!(old, 42);
        assert_eq!(cell.get(), &Default::default());
    }

    #[test]
    fn test_into_inner() {
        let cell = MyCell::new(42);
        let value = cell.into_inner();
        assert_eq!(value, 42);
    }
}
