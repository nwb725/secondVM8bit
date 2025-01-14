
pub trait BytesConverter {
    fn to_bytes(&self) -> Vec<u8>;
}

impl BytesConverter for String {
    fn to_bytes(&self) -> Vec<u8> {
        self.clone().into_bytes()
    }
} 

impl BytesConverter for u8 {
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl BytesConverter for &str {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl BytesConverter for Vec<u8> {
    fn to_bytes(&self) -> Vec<u8> {
        self.clone()
    }
}
