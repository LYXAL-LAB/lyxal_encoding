
#[cfg(feature = "alloc")]
use crate::bigint::BigUint;
#[cfg(feature = "alloc")]
pub(crate) fn encode<T>(alpha: &[T], input: &[u8]) -> Vec<T>
where
    T: Copy,
{
    if input.is_empty() {
        return Vec::new();
    }

    let base = alpha.len() as u32;

    // Convert the input byte array to a BigUint
    let mut big = BigUint::from_bytes_be(input);
    let mut out = Vec::with_capacity(input.len());

    // Find the highest power of `base` that fits in `u32`
    let big_pow = 32 / (32 - base.leading_zeros());
    let big_base = base.pow(big_pow);

    'fast: loop {
        let mut big_rem = big.div_mod(big_base);

        if big.is_zero() {
            loop {
                let (result, remainder) = (big_rem / base, big_rem % base);
                out.push(alpha[remainder as usize]);
                big_rem = result;

                if big_rem == 0 {
                    break 'fast;
                }
            }
        } else {
            for _ in 0..big_pow {
                let (result, remainder) = (big_rem / base, big_rem % base);
                out.push(alpha[remainder as usize]);
                big_rem = result;
            }
        }
    }

    let leaders = input
        .iter()
        .take(input.len() - 1)
        .take_while(|i| **i == 0)
        .map(|_| alpha[0]);

    out.extend(leaders);
    out
}

pub(crate) fn encode_to_buffer(
    alpha: &[u8],
    input: &[u8],
    output: &mut [u8],
) -> Result<usize, crate::EncodeError> {
    if input.is_empty() {
        return Ok(0);
    }

    let base = alpha.len() as u32;

    // Stack buffer for BigUint computation (512 bytes capacity)
    let mut chunks = [0u32; 128];
    let mut big = crate::bigint::BigUintView::new(&mut chunks);
    if !big.load_be_bytes(input) {
        return Err(crate::EncodeError::InputTooLarge);
    }

    let mut out_idx = 0;
    let big_pow = 32 / (32 - base.leading_zeros());
    let big_base = base.pow(big_pow);

    'fast: loop {
        let mut big_rem = big.div_mod(big_base);

        if big.is_zero() {
            loop {
                let (result, remainder) = (big_rem / base, big_rem % base);
                if out_idx >= output.len() {
                    return Err(crate::EncodeError::BufferTooSmall);
                }
                output[out_idx] = alpha[remainder as usize];
                out_idx += 1;
                big_rem = result;

                if big_rem == 0 {
                    break 'fast;
                }
            }
        } else {
            for _ in 0..big_pow {
                let (result, remainder) = (big_rem / base, big_rem % base);
                if out_idx >= output.len() {
                    return Err(crate::EncodeError::BufferTooSmall);
                }
                output[out_idx] = alpha[remainder as usize];
                out_idx += 1;
                big_rem = result;
            }
        }
    }

    // Add leaders (zeros at the beginning of input)
    let leader = alpha[0];
    for &byte in input.iter().take(input.len() - 1) {
        if byte == 0 {
            if out_idx >= output.len() {
                return Err(crate::EncodeError::BufferTooSmall);
            }
            output[out_idx] = leader;
            out_idx += 1;
        } else {
            break;
        }
    }

    // Reverse the output in place
    output[..out_idx].reverse();

    Ok(out_idx)
}
