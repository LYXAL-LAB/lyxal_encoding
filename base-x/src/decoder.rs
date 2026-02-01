#[cfg(feature = "alloc")]
use crate::Vec;

#[cfg(feature = "alloc")]
use crate::bigint::BigUint;
use crate::bigint::BigUintView;
use crate::DecodeError;

#[allow(clippy::extra_unused_lifetimes)]
pub(crate) trait Decoder<'a, 'b>
where
    <Self::Iter as Iterator>::Item: core::cmp::PartialEq + Copy,
{
    type Iter: core::iter::Iterator;

    fn iter(_: &'a str) -> Self::Iter;
    fn carry(&self, _: <Self::Iter as core::iter::Iterator>::Item) -> Option<u32>;
    fn alphabet<'c>(&self) -> &'c [<Self::Iter as core::iter::Iterator>::Item]
    where
        'b: 'c;

    #[cfg(feature = "alloc")]
    fn decode(&self, input: &'a str) -> Result<Vec<u8>, DecodeError> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        let alpha = self.alphabet();
        let base = alpha.len() as u32;

        let mut big = BigUint::with_capacity(4);

        for c in Self::iter(input) {
            if let Some(carry) = self.carry(c) {
                big.mul_add(base, carry);
            } else {
                return Err(DecodeError);
            }
        }

        let leader = alpha[0];
        let leaders = Self::iter(input).take_while(|byte| *byte == leader).count();

        let bytes = big.into_bytes_be();

        let mut res = Vec::with_capacity(bytes.len() + leaders);
        res.resize(leaders, 0);
        res.extend(bytes);

        Ok(res)
    }

    fn decode_to_buffer(&self, input: &'a str, output: &mut [u8]) -> Result<usize, DecodeError> {
        if input.is_empty() {
            return Ok(0);
        }
        let alpha = self.alphabet();
        let base = alpha.len() as u32;

        // On utilise un buffer fixe sur la pile pour le calcul intermédiaire.
        // 128 chunks = 512 octets de données binaires max.
        let mut chunks = [0u32; 128];
        let mut big = BigUintView::new(&mut chunks);

        for c in Self::iter(input) {
            if let Some(carry) = self.carry(c) {
                big.mul_add(base, carry).map_err(|_| DecodeError)?;
            } else {
                return Err(DecodeError);
            }
        }

        let written = big.copy_into_bytes_be(output).map_err(|_| DecodeError)?;

        let leader = alpha[0];
        let leaders = Self::iter(input).take_while(|byte| *byte == leader).count();

        if leaders > 0 {
            if output.len() < written + leaders {
                return Err(DecodeError);
            }
            // On décale les données vers la droite pour insérer les zéros de tête.
            output.copy_within(0..written, leaders);
            output[..leaders].fill(0);
        }

        Ok(written + leaders)
    }
}

pub(crate) struct U8Decoder<'b> {
    alphabet: &'b [u8],
    lookup: [u8; 256],
}

impl<'a> U8Decoder<'a> {
    #[inline]
    pub(crate) fn new(alphabet: &'a [u8]) -> Self {
        const INVALID_INDEX: u8 = 0xFF;
        let mut lookup = [INVALID_INDEX; 256];

        for (i, byte) in alphabet.iter().enumerate() {
            lookup[*byte as usize] = i as u8;
        }
        U8Decoder { alphabet, lookup }
    }
}

impl<'a, 'b> Decoder<'a, 'b> for U8Decoder<'b> {
    type Iter = core::str::Bytes<'a>;
    #[inline]
    fn iter(s: &'a str) -> Self::Iter {
        s.bytes()
    }
    #[inline]
    fn carry(&self, c: u8) -> Option<u32> {
        match self.lookup[c as usize] {
            0xFF => None,
            index => Some(index.into()),
        }
    }
    #[inline]
    fn alphabet<'c>(&self) -> &'c [u8]
    where
        'b: 'c,
    {
        self.alphabet
    }
}

pub(crate) struct CharDecoder<'b>(pub &'b [char]);

impl<'a, 'b> Decoder<'a, 'b> for CharDecoder<'b> {
    type Iter = core::str::Chars<'a>;

    #[inline]
    fn iter(s: &'a str) -> Self::Iter {
        s.chars()
    }
    #[inline]
    fn carry(&self, c: char) -> Option<u32> {
        self.0
            .iter()
            .enumerate()
            .find(|&(_, ch)| *ch == c)
            .map(|(i, _)| i as u32)
    }
    #[inline]
    fn alphabet<'c>(&self) -> &'c [char]
    where
        'b: 'c,
    {
        self.0
    }
}
