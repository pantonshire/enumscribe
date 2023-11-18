//! Module for the [`CappedString`](CappedString) type, which is a string type which always stores
//! its data inline.

use core::{str, ops::Deref, borrow::Borrow, fmt};

/// A string type which is either borrowed or stores up to `N` bytes of string data inline.
pub enum CowCappedString<'a, const N: usize> {
    /// A reference to string data stored elsewhere.
    Borrowed(&'a str),
    /// The string data is stored inline.
    Owned(CappedString<N>),
}

impl<'a, const N: usize> CowCappedString<'a, N> {
    /// Returns the string data contained by this `CowCappedString`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            CowCappedString::Borrowed(s) => s,
            CowCappedString::Owned(s) => s,
        }
    }

    /// Returns a new `CappedString` with capacity `M` containing the string converted to
    /// uppercase. Returns `None` if the uppercase-converted string is longer than `M` bytes.
    #[inline]
    #[must_use]
    pub fn to_uppercase<const M: usize>(&self) -> Option<CappedString<M>> {
        CappedString::<M>::uppercase_from_str(self)
    }
}

impl<'a, const N: usize> Deref for CowCappedString<'a, N> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<'a, const N: usize> AsRef<str> for CowCappedString<'a, N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl<'a, const N: usize> Borrow<str> for CowCappedString<'a, N> {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> serde::Deserialize<'de> for CowCappedString<'de, N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        deserializer.deserialize_str(CowCappedStringVisitor::<N>)
    }
}

#[cfg(feature = "serde")]
struct CowCappedStringVisitor<const N: usize>;

#[cfg(feature = "serde")]
impl<'de, const N: usize> serde::de::Visitor<'de> for CowCappedStringVisitor<N> {
    type Value = CowCappedString<'de, N>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a borrowed string or a string up to {} bytes long", N)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        CappedStringVisitor::<N>.visit_str(v)
            .map(CowCappedString::Owned)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        CappedStringVisitor::<N>.visit_bytes(v)
            .map(CowCappedString::Owned)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CowCappedString::Borrowed(v))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        str::from_utf8(v)
            .map_err(|_| E::invalid_value(serde::de::Unexpected::Bytes(v), &self))
            .and_then(|v| self.visit_borrowed_str(v))
    }
}

/// A string type which stores up to `N` bytes of string data inline.
pub struct CappedString<const N: usize> {
    /// The string data. It is an invariant that the first `len` bytes must be valid UTF-8.
    buf: [u8; N],
    // The length of the string data in the buffer. It is an invariant that `len <= N`.
    len: usize,
}

impl<const N: usize> CappedString<N> {
    /// Returns a new `CappedString` containing a copy of the given string data. Returns `None` if
    /// the string data is larger than `N` bytes.
    #[inline]
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        unsafe { Self::from_utf8_unchecked(s.as_bytes()) }
    }

    /// Returns a new `CappedString` containing an uppercase conversion of the given string data.
    /// Returns `None` if the converted string is larger than `N` bytes.
    #[inline]
    #[must_use]
    pub fn uppercase_from_str(s: &str) -> Option<Self> {
        let mut buf = [0u8; N];
        let mut cursor = 0usize;

        for c_orig in s.chars() {
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
        unsafe { Self::from_utf8_unchecked(filled_buf) }
    }

    /// Returns a new `CappedString` containing a copy of the given UTF-8 encoded string data.
    /// Returns `None` if more than `N` bytes of data are given.
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

    /// Returns a new `CappedString` from a given buffer and length.
    /// 
    /// # Safety
    /// - `len <= N` must hold.
    /// - The first `len` bytes of `buf` must be valid UTF-8.
    #[inline]
    #[must_use]
    pub unsafe fn from_raw_parts(buf: [u8; N], len: usize) -> Self {
        Self { buf, len }
    }


    /// Consumes the `CappedString` and returns its buffer and length.
    #[inline]
    #[must_use]
    pub fn into_raw_parts(self) -> ([u8; N], usize) {
        (self.buf, self.len)
    }

    /// Returns the string data contained by this `CappedString`.
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

    /// Returns a new `CappedString` with capacity `M` containing the string converted to
    /// uppercase. Returns `None` if the uppercase-converted string is longer than `M` bytes.
    #[inline]
    #[must_use]
    pub fn to_uppercase<const M: usize>(&self) -> Option<CappedString<M>> {
        CappedString::<M>::uppercase_from_str(self)
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

impl<const N: usize> PartialEq for CappedString<N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> Eq for CappedString<N> {}

impl<const N: usize> PartialEq<str> for CappedString<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
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

#[cfg(feature = "serde")]
struct CappedStringVisitor<const N: usize>;

#[cfg(feature = "serde")]
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
            .and_then(|v| self.visit_str(v))
    }
}

#[cfg(test)]
mod tests {
    use super::{CappedString, CowCappedString};

    #[cfg(feature = "serde")]
    #[test]
    fn test_cow_capped_string_deserialize() {
        struct DeBorrowedOnly<const N: usize>(String);

        impl<'de, const N: usize> serde::Deserialize<'de> for DeBorrowedOnly<N> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>
            {
                match CowCappedString::<'de, N>::deserialize(deserializer)? {
                    CowCappedString::Borrowed(s) => Ok(Self(s.to_owned())),
                    CowCappedString::Owned(_) => {
                        Err(serde::de::Error::custom("expected borrowed CowCappedString"))
                    },
                }
            }
        }

        struct DeOwnedOnly<const N: usize>(String);

        impl<'de, const N: usize> serde::Deserialize<'de> for DeOwnedOnly<N> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>
            {
                match CowCappedString::<'de, N>::deserialize(deserializer)? {
                    CowCappedString::Borrowed(_) => {
                        Err(serde::de::Error::custom("expected owned CowCappedString"))
                    },
                    CowCappedString::Owned(s) => Ok(Self(s.to_owned())),
                }
            }
        }

        {
            let DeBorrowedOnly(s) = serde_json::from_str::<DeBorrowedOnly<5>>(
                r#""hello""#
            ).unwrap();
            assert_eq!(s, "hello");
        }
        {
            let DeBorrowedOnly(s) = serde_json::from_str::<DeBorrowedOnly<0>>(
                r#""hello""#
            ).unwrap();
            assert_eq!(s, "hello");
        }
        {
            let s = serde_json::from_str::<DeOwnedOnly<5>>(
                r#""hello""#
            );
            assert!(s.is_err());
        }
        {
            let DeOwnedOnly(s) = serde_json::from_str::<DeOwnedOnly<3>>(
                r#""\u87f9""#
            ).unwrap();
            assert_eq!(s, "蟹");
        }
        {
            let s = serde_json::from_str::<DeBorrowedOnly<3>>(
                r#""\u87f9""#
            );
            assert!(s.is_err());
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_capped_string_deserialize() {
        {
            let s = serde_json::from_str::<CappedString<5>>(
                r#""hello""#
            ).unwrap();
            assert_eq!(s.as_str(), "hello");
        }
        {
            let s = serde_json::from_str::<CappedString<4>>(
                r#""hello""#
            );
            assert!(s.is_err());
        }
        {
            let s = serde_json::from_str::<CappedString<10>>(
                r#""hello""#
            ).unwrap();
            assert_eq!(s.as_str(), "hello");
        }
        {
            let s = serde_json::from_str::<CappedString<12>>(
                r#""hello\tworld\n""#
            ).unwrap();
            assert_eq!(s.as_str(), "hello\tworld\n");
        }
        {
            let s = serde_json::from_str::<CappedString<3>>(
                r#""\u87f9""#
            ).unwrap();
            assert_eq!(s.as_str(), "蟹");
        }
        {
            let s = serde_json::from_str::<CappedString<2>>(
                r#""\u87f9""#
            );
            assert!(s.is_err());
        }
    }

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
            let s = CappedString::<5>::from_str("hello").unwrap();
            assert!(s.to_uppercase::<4>().is_none());
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
