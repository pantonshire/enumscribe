//! Module for the [`CappedString`](CappedString) type, which is a string type which always stores
//! its data inline.

use core::{str, convert::TryFrom, ops::Deref, borrow::Borrow, fmt};

/// TODO: documentation
pub struct CappedString<const N: usize> {
    /// The string data. It is an invariant that this must always be valid UTF-8.
    buf: [u8; N],
}

impl<const N: usize> CappedString<N> {
    /// TODO: documentation
    #[inline]
    #[must_use]
    pub fn new(s: &str) -> Option<Self> {
        unsafe { Self::from_utf8_unchecked(s.as_bytes()) }
    }

    /// TODO: documentation
    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(bs: &[u8]) -> Option<Self> {
        let buf =  <[u8; N]>::try_from(bs).ok()?;
        Some(Self { buf })
    }

    /// TODO: documentation
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.buf) }
    }

    /// TODO: documentation
    #[inline]
    #[must_use]
    pub fn to_uppercase<const M: usize>(&self) -> Option<CappedString<M>> {
        todo!()
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
        CappedString::new(v)
            .ok_or_else(|| E::invalid_length(v.len(), &self))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        str::from_utf8(v)
            .map_err(|_| E::invalid_value(serde::de::Unexpected::Bytes(v), &self))
            .and_then(|v| CappedString::new(v)
                .ok_or_else(|| E::invalid_length(v.len(), &self)))
    }
}
