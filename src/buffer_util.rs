use memchr::memchr;


/// Finds subslice in a slice of a buffer. It's included here as a means
/// of fastest known implementation of the thing.
///
// If you know any faster way to do this send a pull request
pub fn find_substr<B:AsRef<[u8]>, N:AsRef<[u8]>>(haystack: B, needle: N)
    -> Option<usize>
{
    let haystack = haystack.as_ref();
    let needle = needle.as_ref();
    debug_assert!(needle.len() > 0);
    if needle.len() > haystack.len() {
        return None;
    }
    let mut offset = 0;
    let end = haystack.len() - needle.len()+1;
    loop {
        match memchr(needle[0], &haystack[offset..end]) {
            Some(x) if &haystack[offset+x..offset+x+needle.len()] == needle
            => {
                return Some(offset+x);
            }
            Some(x) => {
                offset += x + 1;
                continue;
            }
            None => {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::find_substr;

    #[test]
    fn middle() {
        assert_eq!(find_substr("hello\r\n\r\nworld", "\r\n\r\n"), Some(5));
    }

    #[test]
    fn end() {
        assert_eq!(find_substr("hello world\r\n\r\n", "\r\n\r\n"), Some(11));
    }
    #[test]
    fn partial() {
        assert_eq!(find_substr("hello\r\nworld\r\n\r\n", "\r\n\r\n"), Some(12));
    }
}
