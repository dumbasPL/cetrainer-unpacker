use miniz_oxide::inflate;
use nom::{
    bytes::complete::take, error::Error, multi::many_m_n, number::complete::le_u32, Err, IResult,
};

#[derive(Debug)]
pub struct FileEntry {
    pub filename: String,
    pub folder: String,
    pub data: Vec<u8>,
}

fn length_prefixed_string(input: &[u8]) -> IResult<&[u8], String> {
    let (remaining, len) = le_u32(input)?;
    let (remaining, string) = take(len)(remaining)?;
    let string = String::from_utf8(string.to_vec()).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(
            remaining,
            nom::error::ErrorKind::Verify,
        ))
    })?;
    Ok((remaining, string))
}

fn file_entry(input: &[u8]) -> IResult<&[u8], FileEntry> {
    let (remaining, filename) = length_prefixed_string(input)?;
    let (remaining, folder) = length_prefixed_string(remaining)?;
    let (remaining, len) = le_u32(remaining)?;
    let (remaining, data) = take(len)(remaining)?;
    Ok((
        remaining,
        FileEntry {
            filename,
            folder,
            data: data.to_vec(),
        },
    ))
}

fn convert_compressed_error<I, J>(e: Err<Error<I>>, og_input: J) -> Err<Error<J>> {
    match e {
        Err::Incomplete(needed) => Err::Incomplete(needed),
        Err::Error(e) => Err::Error(Error::new(og_input, e.code)),
        Err::Failure(e) => Err::Failure(Error::new(og_input, e.code)),
    }
}

pub fn parse_and_decompress(input: &[u8]) -> IResult<&[u8], Vec<FileEntry>> {
    let (compressed, count) = le_u32(input)?;
    let uncompressed = inflate::decompress_to_vec(compressed).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(
            compressed,
            nom::error::ErrorKind::Verify,
        ))
    })?;
    let uncompressed = uncompressed.as_slice();

    let (remaining, files) = many_m_n(count as usize, count as usize, file_entry)(uncompressed)
        .map_err(|e| convert_compressed_error(e, compressed))?;

    match remaining {
        [] => Ok((&[], files)),
        _ => Err(Err::Error(Error::new(
            compressed,
            nom::error::ErrorKind::Verify,
        ))),
    }
}
