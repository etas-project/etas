//! Lossless storage of native SDK units against shared, authenticated bytes.
//! This is an encoding layer: dependency selection and checked IR stay unchanged.
use super::{StdBundleFile, invalid, write_file};
use crate::DriverError;
use std::{
    collections::{BTreeMap, HashMap},
    io::{Cursor, Read},
    path::Path,
    sync::Arc,
};
const WINDOW: usize = 64;
const FACTOR: u64 = 257;
const MAGIC: &[u8] = b"ETASDEL1";

#[derive(Clone, Debug)]
struct Base {
    file: StdBundleFile,
    bytes: Arc<[u8]>,
    blocks: HashMap<u64, usize>,
}
#[derive(Clone, Debug, Default)]
pub(super) struct BundleStore {
    bases: BTreeMap<String, Base>,
}
fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0u64, |hash, byte| {
        hash.wrapping_mul(FACTOR).wrapping_add(*byte as u64)
    })
}
fn number(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&(value as u64).to_le_bytes());
}
fn literal(output: &mut Vec<u8>, bytes: &[u8]) {
    if !bytes.is_empty() {
        output.push(1);
        number(output, bytes.len());
        output.extend_from_slice(bytes);
    }
}
impl BundleStore {
    pub(super) fn write(
        &mut self,
        root: &Path,
        path: std::path::PathBuf,
        kind: &str,
        bytes: &[u8],
    ) -> Result<StdBundleFile, DriverError> {
        if let Some(base) = self.bases.get(kind) {
            if base.bytes.as_ref() == bytes {
                return Ok(base.file.clone());
            }
            let patch = encode(base, bytes);
            if patch.len() < bytes.len() {
                if decode(&base.bytes, &patch)? != bytes {
                    return Err(invalid("SDK encoding changed the checked unit bytes"));
                }
                write_file(&root.join(&path), &patch)?;
                return Ok(StdBundleFile {
                    path,
                    hash: etas_package_metadata::blake3_hash(&patch),
                    base: Some(Box::new(base.file.clone())),
                });
            }
        }
        write_file(&root.join(&path), bytes)?;
        let file = StdBundleFile {
            path,
            hash: etas_package_metadata::blake3_hash(bytes),
            base: None,
        };
        if !self.bases.contains_key(kind) {
            let mut blocks = HashMap::new();
            for (index, block) in bytes.chunks_exact(WINDOW).enumerate() {
                blocks.entry(hash(block)).or_insert(index * WINDOW);
            }
            self.bases.insert(
                kind.to_owned(),
                Base {
                    file: file.clone(),
                    bytes: Arc::from(bytes),
                    blocks,
                },
            );
        }
        Ok(file)
    }
}
fn encode(base: &Base, bytes: &[u8]) -> Vec<u8> {
    let mut output = MAGIC.to_vec();
    number(&mut output, bytes.len());
    let power = (1..WINDOW).fold(1u64, |value, _| value.wrapping_mul(FACTOR));
    let mut position = 0;
    let mut start = 0;
    let mut rolling = if bytes.len() >= WINDOW {
        hash(&bytes[..WINDOW])
    } else {
        0
    };
    while position + WINDOW <= bytes.len() {
        if let Some(&offset) = base.blocks.get(&rolling)
            && base.bytes[offset..offset + WINDOW] == bytes[position..position + WINDOW]
        {
            let mut count = WINDOW;
            while offset + count < base.bytes.len()
                && position + count < bytes.len()
                && base.bytes[offset + count] == bytes[position + count]
            {
                count += 1;
            }
            literal(&mut output, &bytes[start..position]);
            output.push(0);
            number(&mut output, offset);
            number(&mut output, count);
            position += count;
            start = position;
            if position + WINDOW <= bytes.len() {
                rolling = hash(&bytes[position..position + WINDOW]);
            }
        } else {
            rolling = rolling
                .wrapping_sub((bytes[position] as u64).wrapping_mul(power))
                .wrapping_mul(FACTOR);
            position += 1;
            if position + WINDOW <= bytes.len() {
                rolling = rolling.wrapping_add(bytes[position + WINDOW - 1] as u64);
            }
        }
    }
    literal(&mut output, &bytes[start..]);
    output
}
fn read_number(input: &mut Cursor<&[u8]>) -> Result<usize, DriverError> {
    let mut bytes = [0; 8];
    input.read_exact(&mut bytes).map_err(invalid)?;
    usize::try_from(u64::from_le_bytes(bytes)).map_err(invalid)
}
pub(super) fn decode(base: &[u8], patch: &[u8]) -> Result<Vec<u8>, DriverError> {
    let payload = patch
        .strip_prefix(MAGIC)
        .ok_or_else(|| invalid("incompatible SDK delta"))?;
    let mut input = Cursor::new(payload);
    let size = read_number(&mut input)?;
    let mut output = Vec::with_capacity(size);
    while input.position() < payload.len() as u64 {
        let mut tag = [0];
        input.read_exact(&mut tag).map_err(invalid)?;
        match tag[0] {
            0 => {
                let offset = read_number(&mut input)?;
                let length = read_number(&mut input)?;
                let bytes = base
                    .get(
                        offset
                            ..offset
                                .checked_add(length)
                                .ok_or_else(|| invalid("SDK delta range overflow"))?,
                    )
                    .ok_or_else(|| invalid("SDK delta references missing base bytes"))?;
                output.extend_from_slice(bytes);
            }
            1 => {
                let length = read_number(&mut input)?;
                let start = output.len();
                output.resize(start + length, 0);
                input.read_exact(&mut output[start..]).map_err(invalid)?;
            }
            _ => return Err(invalid("invalid SDK delta operation")),
        }
    }
    if output.len() != size {
        return Err(invalid(
            "SDK delta length differs from its checked encoding",
        ));
    }
    Ok(output)
}
