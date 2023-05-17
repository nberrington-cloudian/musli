use core::ffi::CStr;
use core::fmt;
#[cfg(feature = "std")]
use core::hash::{BuildHasher, Hash};
use core::marker;

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BinaryHeap, VecDeque};
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

use crate::de::{Decode, Decoder, PairDecoder, PairsDecoder, SequenceDecoder, ValueVisitor};
use crate::en::{Encode, Encoder, PairEncoder, PairsEncoder, SequenceEncoder};
use crate::error::Error;
use crate::internal::size_hint;
use crate::mode::Mode;

impl<M> Encode<M> for String
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        Encode::<M>::encode(self.as_str(), encoder)
    }
}

impl<'de, M> Decode<'de, M> for String
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor<E>(marker::PhantomData<E>);

        impl<'de, E> ValueVisitor<'de> for Visitor<E>
        where
            E: Error,
        {
            type Target = str;
            type Ok = String;
            type Error = E;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "string")
            }

            #[inline]
            fn visit_owned(self, value: String) -> Result<Self::Ok, Self::Error> {
                Ok(value)
            }

            #[inline]
            fn visit_borrowed(self, string: &'de str) -> Result<Self::Ok, Self::Error> {
                self.visit_ref(string)
            }

            #[inline]
            fn visit_ref(self, string: &str) -> Result<Self::Ok, Self::Error> {
                Ok(string.to_owned())
            }
        }

        decoder.decode_string(Visitor(marker::PhantomData))
    }
}

impl<M> Encode<M> for Box<str>
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        Encode::<M>::encode(self.as_ref(), encoder)
    }
}

impl<'de, M> Decode<'de, M> for Box<str>
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(<String as Decode<M>>::decode(decoder)?.into())
    }
}

macro_rules! cow {
    ($ty:ty, $source:ty, $decode:ident, |$owned:ident| $owned_expr:expr, |$borrowed:ident| $borrowed_expr:expr, |$reference:ident| $reference_expr:expr) => {
        impl<M> Encode<M> for Cow<'_, $ty>
        where
            M: Mode,
        {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                Encode::<M>::encode(self.as_ref(), encoder)
            }
        }

        impl<'de, M> Decode<'de, M> for Cow<'de, $ty>
        where
            M: Mode,
        {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                struct Visitor<E>(marker::PhantomData<E>);

                impl<'de, E> ValueVisitor<'de> for Visitor<E>
                where
                    E: Error,
                {
                    type Target = $source;
                    type Ok = Cow<'de, $ty>;
                    type Error = E;

                    #[inline]
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "string")
                    }

                    #[inline]
                    fn visit_owned(
                        self,
                        $owned: <$source as ToOwned>::Owned,
                    ) -> Result<Self::Ok, Self::Error> {
                        Ok($owned_expr)
                    }

                    #[inline]
                    fn visit_borrowed(
                        self,
                        $borrowed: &'de $source,
                    ) -> Result<Self::Ok, Self::Error> {
                        Ok($borrowed_expr)
                    }

                    #[inline]
                    fn visit_ref(self, $reference: &$source) -> Result<Self::Ok, Self::Error> {
                        Ok($reference_expr)
                    }
                }

                decoder.$decode(Visitor(marker::PhantomData))
            }
        }
    };
}

cow! {
    str, str, decode_string,
    |owned| Cow::Owned(owned),
    |borrowed| Cow::Borrowed(borrowed),
    |reference| Cow::Owned(reference.to_owned())
}

cow! {
    CStr, [u8], decode_bytes,
    |owned| Cow::Owned(CString::from_vec_with_nul(owned).map_err(E::custom)?),
    |borrowed| Cow::Borrowed(CStr::from_bytes_with_nul(borrowed).map_err(E::custom)?),
    |reference| Cow::Owned(CStr::from_bytes_with_nul(reference).map_err(E::custom)?.to_owned())
}

macro_rules! sequence {
    (
        $ty:ident <T $(: $trait0:ident $(+ $trait:ident)*)? $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $insert:ident,
        $access:ident,
        $factory:expr
    ) => {
        impl<M, T $(, $extra)*> Encode<M> for $ty<T $(, $extra)*>
        where
            M: Mode,
            T: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                let mut seq = encoder.encode_sequence(self.len())?;

                for value in self {
                    value.encode(seq.next()?)?;
                }

                seq.end()
            }
        }

        impl<'de, M, T $(, $extra)*> Decode<'de, M> for $ty<T $(, $extra)*>
        where
            M: Mode,
            T: Decode<'de, M> $(+ $trait0 $(+ $trait)*)*,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                let mut $access = decoder.decode_sequence()?;
                let mut out = $factory;

                while let Some(value) = $access.next()? {
                    out.$insert(T::decode(value)?);
                }

                $access.end()?;
                Ok(out)
            }
        }
    }
}

sequence!(
    Vec<T>,
    push,
    seq,
    Vec::with_capacity(size_hint::cautious(seq.size_hint()))
);
sequence!(
    VecDeque<T>,
    push_back,
    seq,
    VecDeque::with_capacity(size_hint::cautious(seq.size_hint()))
);
#[cfg(feature = "std")]
sequence!(
    HashSet<T: Eq + Hash, S: BuildHasher + Default>,
    insert,
    seq,
    HashSet::with_capacity_and_hasher(size_hint::cautious(seq.size_hint()), S::default())
);
sequence!(
    BinaryHeap<T: Ord>,
    push,
    seq,
    BinaryHeap::with_capacity(size_hint::cautious(seq.size_hint()))
);

macro_rules! map {
    (
        $ty:ident<K $(: $key_bound0:ident $(+ $key_bound:ident)*)?, V $(, $extra:ident: $extra_bound0:ident $(+ $extra_bound:ident)*)*>,
        $access:ident,
        $with_capacity:expr
    ) => {
        impl<'de, M, K, V $(, $extra)*> Encode<M> for $ty<K, V $(, $extra)*>
        where
            M: Mode,
            K: Encode<M>,
            V: Encode<M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
            where
                E: Encoder,
            {
                let mut map = encoder.encode_map(self.len())?;

                for (k, v) in self {
                    let mut entry = map.next()?;
                    k.encode(entry.first()?)?;
                    v.encode(entry.second()?)?;
                    entry.end()?;
                }

                map.end()
            }
        }

        impl<'de, K, V, M $(, $extra)*> Decode<'de, M> for $ty<K, V $(, $extra)*>
        where
            M: Mode,
            K: Decode<'de, M> $(+ $key_bound0 $(+ $key_bound)*)*,
            V: Decode<'de, M>,
            $($extra: $extra_bound0 $(+ $extra_bound)*),*
        {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: Decoder<'de>,
            {
                let mut $access = decoder.decode_map()?;
                let mut out = $with_capacity;

                while let Some(mut entry) = $access.next()? {
                    let key = entry.first().and_then(K::decode)?;
                    let value = entry.second().and_then(V::decode)?;
                    out.insert(key, value);
                }

                $access.end()?;
                Ok(out)
            }
        }
    }
}

map!(BTreeMap<K: Ord, V>, map, BTreeMap::new());

#[cfg(feature = "std")]
map!(
    HashMap<K: Eq + Hash, V, S: BuildHasher + Default>,
    map,
    HashMap::with_capacity_and_hasher(size_hint::cautious(map.size_hint()), S::default())
);

impl<M> Encode<M> for CString
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_bytes(self.to_bytes_with_nul())
    }
}

impl<'de, M> Decode<'de, M> for CString
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor<E>(marker::PhantomData<E>);

        impl<'de, E> ValueVisitor<'de> for Visitor<E>
        where
            E: Error,
        {
            type Target = [u8];
            type Ok = CString;
            type Error = E;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a cstring")
            }

            #[inline]
            fn visit_owned(self, value: Vec<u8>) -> Result<Self::Ok, Self::Error> {
                CString::from_vec_with_nul(value).map_err(E::custom)
            }

            #[inline]
            fn visit_borrowed(self, bytes: &'de [u8]) -> Result<Self::Ok, Self::Error> {
                self.visit_ref(bytes)
            }

            #[inline]
            fn visit_ref(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
                Ok(CStr::from_bytes_with_nul(bytes)
                    .map_err(E::custom)?
                    .to_owned())
            }
        }

        decoder.decode_bytes(Visitor(marker::PhantomData))
    }
}
