pub fn hex_encode<T: AsRef<[u8]>>(src: T) -> String {
    faster_hex::hex_string(src.as_ref())
}

pub fn hex_decode(src: &str) -> Result<Vec<u8>, String> {
    if src.is_empty() {
        return Ok(Vec::new());
    }

    let src = if src.starts_with("0x") {
        src.split_at(2).1
    } else {
        src
    };

    let src = src.as_bytes();
    let mut ret = vec![0u8; src.len() / 2];
    faster_hex::hex_decode(src, &mut ret).map_err(|e| e.to_string())?;

    Ok(ret)
}

pub(crate) fn deserialize_uint<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value_str: &str = serde::Deserialize::deserialize(d)?;
    if let Some(raw_str) = value_str.strip_prefix("0x") {
        u64::from_str_radix(raw_str, 16).map_err(serde::de::Error::custom)
    } else {
        value_str.parse().map_err(serde::de::Error::custom)
    }
}

mod tests {

    #[test]
    fn test_serialize_deserialize() {
        use crate::types::axon_rpc_client::{mock_header, Header};

        let deserialized_header: Header = mock_header();

        println!("Deserialized Header: {:?}", deserialized_header);
    }

    #[test]
    fn test_hex_decode() {
        use crate::types::axon_rpc_client::Hex;
        let hex = String::from("0x");
        let res = Hex::from_string(hex.clone()).unwrap();
        assert!(res.is_empty());

        let res = Hex::decode(hex).unwrap();
        assert!(res.is_empty());

        let hex = String::from("123456");
        let _ = Hex::from_string(hex.clone()).unwrap();
        let _ = Hex::decode(hex).unwrap();

        let hex = String::from("0x123f");
        let _ = Hex::from_string(hex.clone()).unwrap();
        let _ = Hex::decode(hex).unwrap();
    }

    #[test]
    fn test_metadata_json_serialize() {
        use crate::types::axon_rpc_client::{mock_metadata, Metadata};
        let metadata: Metadata = mock_metadata();
        let deserialized_metadata = serde_json::to_value(metadata).unwrap();

        println!("{:?}", deserialized_metadata.to_string());

        assert!(deserialized_metadata.get("version").unwrap().is_object());
        assert!(deserialized_metadata.get("epoch").unwrap().is_u64());
        assert!(deserialized_metadata
            .get("propose_counter")
            .unwrap()
            .is_array());
        assert!(deserialized_metadata
            .get("verifier_list")
            .unwrap()
            .get(0)
            .unwrap()
            .get("bls_pub_key")
            .unwrap()
            .is_string());
        assert_eq!(
            deserialized_metadata
                .get("propose_counter")
                .and_then(|propose_counter| propose_counter.get(0))
                .and_then(|item| item.get("count"))
                .and_then(|count| count.as_u64())
                .unwrap(),
            48136
        );
    }

    #[test]
    fn test_metadata_version() {
        use crate::types::axon_rpc_client::MetadataVersion;
        let version_0 = MetadataVersion {
            start: 1,
            end:   100,
        };
        let version_1 = MetadataVersion {
            start: 101,
            end:   200,
        };

        (1..=100).for_each(|n| assert!(version_0.contains(n)));
        (101..=200).for_each(|n| assert!(version_1.contains(n)));
    }

    #[test]
    fn test_hex_codec() {
        use crate::utils::codec::{hex_decode, hex_encode};
        use rand::random;
        let data = (0..128).map(|_| random()).collect::<Vec<u8>>();
        let data = data.to_vec();

        assert_eq!(hex_encode(&data), hex::encode(data.clone()));
        assert_eq!(
            hex_decode(&hex_encode(&data)).unwrap(),
            hex::decode(hex::encode(data)).unwrap()
        );
        assert!(hex_decode(String::new().as_str()).unwrap().is_empty());
    }
}
