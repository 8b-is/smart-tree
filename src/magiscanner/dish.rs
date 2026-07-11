/// The data container passed between operations in a recipe.
/// Mirrors CyberChef's Dish but simplified: canonical form is raw bytes.
#[derive(Debug, Clone)]
pub struct Dish {
    data: Vec<u8>,
}

impl Dish {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn from_str(s: &str) -> Self {
        Self {
            data: s.as_bytes().to_vec(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl From<Vec<u8>> for Dish {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<&str> for Dish {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for Dish {
    fn from(s: String) -> Self {
        Self::new(s.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dish_roundtrip() {
        let dish = Dish::from_str("hello");
        assert_eq!(dish.as_str().unwrap(), "hello");
        assert_eq!(dish.len(), 5);
        assert!(!dish.is_empty());
    }

    #[test]
    fn test_dish_bytes() {
        let data = vec![0x00, 0xFF, 0x42];
        let dish = Dish::new(data.clone());
        assert_eq!(dish.as_bytes(), &data);
        assert_eq!(dish.into_bytes(), data);
    }
}
