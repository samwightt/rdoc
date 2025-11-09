/// VLQ (Variable-Length Quantity) hex decoder for rustdoc search index
///
/// Decodes hex-encoded strings where each character contributes 4 bits.
/// Characters with code < 96 are continuation bytes, >= 96 are terminal bytes.
pub struct VlqHexDecoder<'a> {
    string: &'a str,
    offset: usize,
}

impl<'a> VlqHexDecoder<'a> {
    pub fn new(string: &'a str) -> Self {
        Self { string, offset: 0 }
    }

    pub fn next(&mut self) -> Option<i32> {
        if self.offset >= self.string.len() {
            return None;
        }

        let c = self.string.as_bytes()[self.offset] as u32;

        // Decode a single VLQ value
        let mut n = 0u32;
        let mut current = c;

        // Read hex digits while char code < 96 (continuation bytes)
        while current < 96 && self.offset < self.string.len() {
            n = (n << 4) | (current & 15);
            self.offset += 1;
            if self.offset < self.string.len() {
                current = self.string.as_bytes()[self.offset] as u32;
            } else {
                break;
            }
        }

        // Last byte (char code >= 96)
        if current >= 96 && self.offset < self.string.len() {
            n = (n << 4) | (current & 15);
            self.offset += 1;
        }

        // LSB is sign bit, rest is value
        let sign = n & 1;
        let value = (n >> 1) as i32;

        Some(if sign == 1 { -value } else { value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_zero() {
        // Based on the docs: backtick (char code 96) should decode to 0
        // For now, let's test a simple case where we decode a single zero value
        let mut decoder = VlqHexDecoder::new("a");
        assert_eq!(decoder.next(), Some(0));
    }

    #[test]
    fn test_decode_positive_number() {
        // 'b' has char code 98
        // n = 98 & 15 = 2 (binary 0010)
        // sign = 2 & 1 = 0 (positive)
        // value = 2 >> 1 = 1
        let mut decoder = VlqHexDecoder::new("b");
        assert_eq!(decoder.next(), Some(1));
    }

    #[test]
    fn test_decode_multiple_values() {
        // Test decoding a sequence: 0, 1, 0
        let mut decoder = VlqHexDecoder::new("aba");
        assert_eq!(decoder.next(), Some(0));
        assert_eq!(decoder.next(), Some(1));
        assert_eq!(decoder.next(), Some(0));
        assert_eq!(decoder.next(), None);
    }
}
