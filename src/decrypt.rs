use std::mem::size_of;

use anyhow::Result;
use miniz_oxide::inflate;
use nom::number::complete::le_u32;

// constant headers
const XML_HEADER: &[u8] = "<?xml".as_bytes();
const DECRYPTED_HEADER: &[u8] = "CHEAT".as_bytes();

fn first_pass(input: &mut [u8]) {
    for i in 2..input.len() {
        input[i] ^= input[i - 2];
    }
}

fn second_pass(input: &mut [u8]) {
    for i in (0..=input.len() - 2).rev() {
        input[i] ^= input[i + 1];
    }
}

fn third_pass(input: &mut [u8]) {
    let mut key = 0xCE;
    for byte in input.iter_mut() {
        *byte ^= key;
        key = key.wrapping_add(1);
    }
}

pub fn decrypt_trainer(input: &[u8]) -> Result<Vec<u8>> {
    let mut input = input.to_vec();
    // return if already decrypted
    if input.starts_with(XML_HEADER) {
        return Ok(input);
    }

    first_pass(&mut input);
    second_pass(&mut input);
    third_pass(&mut input);

    // check if decrypted
    if !input.starts_with(DECRYPTED_HEADER) {
        return Err(anyhow::anyhow!("Failed to decrypt CETRAINER file"));
    }

    // remove header
    let data = &input[DECRYPTED_HEADER.len()..];

    let decompressed = inflate::decompress_to_vec(data)?;

    // read decompressed size
    let size = le_u32(decompressed.as_slice())
        .map_err(|_: nom::Err<nom::error::Error<_>>| {
            anyhow::anyhow!("Failed to read decompressed size")
        })?
        .1 as usize;

    // remove size
    let decompressed = decompressed[size_of::<u32>()..].to_vec();

    if size != decompressed.len() {
        return Err(anyhow::anyhow!("Decompressed size mismatch"));
    }

    // check if decompressed
    if !decompressed.starts_with(XML_HEADER) {
        return Err(anyhow::anyhow!("Failed to decompress CETRAINER file"));
    }

    Ok(decompressed)
}
