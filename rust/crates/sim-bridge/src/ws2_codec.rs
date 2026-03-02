use sim_engine::EngineSnapshot;

const WS2_MAGIC: [u8; 4] = *b"WS2\0";
const WS2_VERSION: u16 = 1;
const WS2_HEADER_SIZE: usize = 16;

pub(crate) fn encode_ws2_blob(snapshot: &EngineSnapshot) -> Option<Vec<u8>> {
    let serialized = bincode::serialize(snapshot).ok()?;
    let compressed = zstd::stream::encode_all(serialized.as_slice(), 3).ok()?;
    let checksum = crc32fast::hash(&compressed);
    let payload_len = compressed.len() as u32;
    let mut out = Vec::with_capacity(WS2_HEADER_SIZE + compressed.len());
    out.extend_from_slice(&WS2_MAGIC);
    out.extend_from_slice(&WS2_VERSION.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    out.extend_from_slice(&checksum.to_le_bytes());
    out.extend_from_slice(&payload_len.to_le_bytes());
    out.extend_from_slice(compressed.as_slice());
    Some(out)
}

pub(crate) fn decode_ws2_blob(bytes: &[u8]) -> Option<EngineSnapshot> {
    if bytes.len() < WS2_HEADER_SIZE {
        return None;
    }
    if bytes[0..4] != WS2_MAGIC {
        return None;
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != WS2_VERSION {
        return None;
    }
    let checksum = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let payload_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
    if bytes.len() != WS2_HEADER_SIZE + payload_len {
        return None;
    }
    let payload = &bytes[WS2_HEADER_SIZE..];
    if crc32fast::hash(payload) != checksum {
        return None;
    }
    let decoded = zstd::stream::decode_all(payload).ok()?;
    bincode::deserialize::<EngineSnapshot>(&decoded).ok()
}
