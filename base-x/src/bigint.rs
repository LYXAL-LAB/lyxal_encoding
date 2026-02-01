#[cfg(feature = "alloc")]
use crate::Vec;


/// BigUintView est une vue sur un buffer de chunks u32.
/// Elle permet de faire de l'arithmétique sans allocation.
pub struct BigUintView<'a> {
    pub chunks: &'a mut [u32],
    /// L'indice du premier chunk non nul dans le buffer.
    pub start: usize,
}

impl<'a> BigUintView<'a> {
    /// Initialise une vue à partir d'un buffer existant.
    pub fn new(chunks: &'a mut [u32]) -> Self {
        for x in chunks.iter_mut() {
            *x = 0;
        }
        let len = chunks.len();
        BigUintView {
            chunks,
            start: len,
        }
    }

    /// Divide self by `divider`, return the remainder of the operation.
    #[inline]
    pub fn div_mod(&mut self, divider: u32) -> u32 {
        let mut carry = 0u64;

        for i in self.start..self.chunks.len() {
            let chunk = &mut self.chunks[i];
            carry = (carry << 32) | u64::from(*chunk);
            *chunk = (carry / u64::from(divider)) as u32;
            carry %= u64::from(divider);
        }

        while self.start < self.chunks.len() && self.chunks[self.start] == 0 {
            self.start += 1;
        }

        carry as u32
    }

    /// Perform a multiplication followed by addition.
    #[inline]
    pub fn mul_add(&mut self, multiplicator: u32, addition: u32) -> Result<(), ()> {
        let mut carry = u64::from(addition);

        for i in (self.start..self.chunks.len()).rev() {
            carry += u64::from(self.chunks[i]) * u64::from(multiplicator);
            self.chunks[i] = carry as u32;
            carry >>= 32;
        }

        if carry > 0 {
            if self.start > 0 {
                self.start -= 1;
                self.chunks[self.start] = carry as u32;
            } else {
                return Err(());
            }
        }
        Ok(())
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.start >= self.chunks.len()
    }

    /// Remplit le buffer à partir d'octets Big-Endian.
    pub fn load_be_bytes(&mut self, bytes: &[u8]) -> bool {
        let byte_len = bytes.len();
        if byte_len == 0 {
            self.start = self.chunks.len();
            return true;
        }

        let needed_chunks = byte_len.div_ceil(4);
        if needed_chunks > self.chunks.len() {
            return false;
        }

        self.start = self.chunks.len() - needed_chunks;
        for x in &mut self.chunks[0..self.start] {
            *x = 0;
        }
        
        let mut byte_idx = byte_len;
        for i in (self.start..self.chunks.len()).rev() {
            let mut chunk = 0u32;
            let mut shift = 0;
            while shift < 32 && byte_idx > 0 {
                byte_idx -= 1;
                chunk |= (bytes[byte_idx] as u32) << shift;
                shift += 8;
            }
            self.chunks[i] = chunk;
        }

        while self.start < self.chunks.len() && self.chunks[self.start] == 0 {
            self.start += 1;
        }

        true
    }

    /// Copie les octets dans un buffer de sortie.
    pub fn copy_into_bytes_be(&self, out: &mut [u8]) -> Result<usize, ()> {
        if self.is_zero() {
            return Ok(0);
        }

        let first_chunk = self.chunks[self.start];
        let skip_in_first = (first_chunk.leading_zeros() / 8) as usize;
        let total_bytes = (self.chunks.len() - self.start) * 4 - skip_in_first;

        if out.len() < total_bytes {
            return Err(());
        }

        let mut out_idx = 0;
        for i in self.start..self.chunks.len() {
            let chunk = self.chunks[i];
            let chunk_bytes = chunk.to_be_bytes();
            let start_byte = if i == self.start { skip_in_first } else { 0 };
            
            for &b in chunk_bytes.iter().skip(start_byte) {
                out[out_idx] = b;
                out_idx += 1;
            }
        }

        Ok(out_idx)
    }
}

// Compatibilité descendante pour le mode alloc
#[cfg(feature = "alloc")]
pub struct BigUint {
    pub chunks: Vec<u32>,
}

#[cfg(feature = "alloc")]
impl BigUint {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut chunks = Vec::with_capacity(capacity);
        chunks.push(0);
        BigUint { chunks }
    }

    pub fn div_mod(&mut self, divider: u32) -> u32 {
        let start = self.chunks.iter().position(|&x| x != 0).unwrap_or(self.chunks.len());
        let mut view = BigUintView {
            chunks: &mut self.chunks,
            start,
        };
        view.div_mod(divider)
    }

    pub fn mul_add(&mut self, multiplicator: u32, addition: u32) {
        let start = self.chunks.iter().position(|&x| x != 0).unwrap_or(self.chunks.len());
        if start == 0 && !self.is_zero() {
            self.chunks.insert(0, 0);
            let mut view = BigUintView {
                chunks: &mut self.chunks,
                start: 1,
            };
            view.mul_add(multiplicator, addition).expect("BigUint allocation failed unexpectedly");
        } else {
            if self.chunks.is_empty() {
                self.chunks.push(0);
            }
            let used_start = self.chunks.iter().position(|&x| x != 0).unwrap_or(self.chunks.len().saturating_sub(1));
            let mut view = BigUintView {
                chunks: &mut self.chunks,
                start: used_start,
            };
            view.mul_add(multiplicator, addition).expect("BigUint allocation failed unexpectedly");
        }
    }

    pub fn is_zero(&self) -> bool {
        self.chunks.iter().all(|&x| x == 0)
    }

    pub fn from_bytes_be(bytes: &[u8]) -> Self {
        let mut chunks = vec![0u32; bytes.len().div_ceil(4)];
        let mut view = BigUintView {
            chunks: &mut chunks,
            start: 0,
        };
        view.load_be_bytes(bytes);
        BigUint { chunks }
    }

    pub fn into_bytes_be(self) -> Vec<u8> {
        let mut chunks = self.chunks;
        let chunks_len = chunks.len();
        let start = chunks.iter().position(|&x| x != 0).unwrap_or(chunks_len);
        let view = BigUintView {
            chunks: &mut chunks,
            start,
        };
        let mut out = vec![0u8; chunks_len * 4];
        if let Ok(len) = view.copy_into_bytes_be(&mut out) {
            out.truncate(len);
            out
        } else {
            Vec::new()
        }
    }
}
