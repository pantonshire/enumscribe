//! Module for the [`CappedString`](CappedString) type, which is a string type which always stores
//! its data inline.

use core::{str, ops::Deref, borrow::Borrow, fmt};

/// TODO: documentation
pub struct CappedString<const N: usize> {
    /// The string data. It is an invariant that the first `len` bytes must be valid UTF-8.
    buf: [u8; N],
    // The length of the string data in the buffer. It is an invariant that `len <= N`.
    len: usize,
}

impl<const N: usize> CappedString<N> {
    /// TODO: documentation
    #[inline]
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        unsafe { Self::from_utf8_unchecked(s.as_bytes()) }
    }

    /// TODO: documentation
    /// 
    /// # Safety
    /// - `bs` must be valid UTF-8.
    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(bs: &[u8]) -> Option<Self> {
        let mut buf = [0u8; N];
        buf.get_mut(..bs.len())?.copy_from_slice(bs);
        
        // SAFETY:
        // - `bs.len() <= N` has already been checked by the `get_mut` call, which will return
        //   `None` and cause us to return early if the condition does not hold.
        // 
        unsafe { Some(Self::from_raw_parts(buf, bs.len())) }
    }

    /// TODO: documentation
    /// 
    /// # Safety
    /// - `len <= N` must hold.
    /// - The first `len` bytes of `buf` must be valid UTF-8.
    #[inline]
    #[must_use]
    pub unsafe fn from_raw_parts(buf: [u8; N], len: usize) -> Self {
        Self { buf, len }
    }


    /// TODO: documentation
    #[inline]
    #[must_use]
    pub fn into_raw_parts(self) -> ([u8; N], usize) {
        (self.buf, self.len)
    }

    /// TODO: documentation
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        // SAFETY:
        // - It is an invariant of `CappedString<N>` that `len <= N`.
        // - It is an invariant of `CappedString<N>` that the first `len` bytes of `buf` are valid
        //   UTF-8.
        unsafe {
            let buf_occupied_prefix = self.buf.get_unchecked(..self.len);
            str::from_utf8_unchecked(buf_occupied_prefix)
        }
    }

    /// TODO: documentation
    #[inline]
    #[must_use]
    pub fn to_uppercase<const M: usize>(&self) -> Option<CappedString<M>> {
        let mut buf = [0u8; M];
        let mut cursor = 0usize;

        for c_orig in self.as_str().chars() {
            for c_upper in c_orig.to_uppercase() {
                let encode_buf = cursor
                    .checked_add(c_upper.len_utf8())
                    .and_then(|encode_buf_end| buf.get_mut(cursor..encode_buf_end))?;

                // FIXME: avoid the panic asm that gets generated for this encode (can never panic,
                // as we always have at least `c_upper.len_utf8()` buffer space).
                let encoded = c_upper.encode_utf8(encode_buf);
                cursor = cursor.checked_add(encoded.len())?;
            }
        }

        let filled_buf = buf.get(..cursor)?;

        // SAFETY:
        // `filled_buf` has been filled with a sequence of bytes obtained from `char::encode_utf8`,
        // so it is valid UTF-8.
        unsafe { CappedString::from_utf8_unchecked(filled_buf) }
    }
}

impl<const N: usize> Deref for CappedString<N> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for CappedString<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl<const N: usize> Borrow<str> for CappedString<N> {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> serde::Deserialize<'de> for CappedString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        deserializer.deserialize_str(CappedStringVisitor::<N>)
    }
}

struct CappedStringVisitor<const N: usize>;

impl<'de, const N: usize> serde::de::Visitor<'de> for CappedStringVisitor<N> {
    type Value = CappedString<N>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a string up to {} bytes long", N)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        CappedString::from_str(v)
            .ok_or_else(|| E::invalid_length(v.len(), &self))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        str::from_utf8(v)
            .map_err(|_| E::invalid_value(serde::de::Unexpected::Bytes(v), &self))
            .and_then(|v| CappedString::from_str(v)
                .ok_or_else(|| E::invalid_length(v.len(), &self)))
    }
}

#[cfg(test)]
mod tests {
    use super::CappedString;

    #[test]
    fn test_capped_string_uppercase() {
        {
            let s1 = CappedString::<5>::from_str("hello").unwrap();
            let s2 = s1.to_uppercase::<5>().unwrap();
            assert_eq!(s2.as_str(), "HELLO");
        }
        {
            let s1 = CappedString::<20>::from_str("hello").unwrap();
            let s2 = s1.to_uppercase::<20>().unwrap();
            assert_eq!(s2.as_str(), "HELLO");
        }
        {
            let s1 = CappedString::<5>::from_str("hElLo").unwrap();
            let s2 = s1.to_uppercase::<5>().unwrap();
            assert_eq!(s2.as_str(), "HELLO");
        }
        {
            let s1 = CappedString::<5>::from_str("hello").unwrap();
            assert!(s1.to_uppercase::<4>().is_none());
        }
        {
            let s1 = CappedString::<5>::from_str("groß").unwrap();
            let s2 = s1.to_uppercase::<5>().unwrap();
            assert_eq!(s2.as_str(), "GROSS");
        }
        {
            let s1 = CappedString::<1>::from_str("").unwrap();
            let s2 = s1.to_uppercase::<1>().unwrap();
            assert_eq!(s2.as_str(), "");
        }
        {
            let s1 = CappedString::<0>::from_str("").unwrap();
            let s2 = s1.to_uppercase::<0>().unwrap();
            assert_eq!(s2.as_str(), "");
        }
    }
}
