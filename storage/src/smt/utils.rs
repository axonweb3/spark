#[macro_export]
macro_rules! create_table_cfs {
    ($cf: expr) => {{
        let cf1 = format!("{}_{}", $cf.to_string(), CFSuffixType::Branch);
        let cf2 = format!("{}_{}", $cf.to_string(), CFSuffixType::Leaf);

        vec![cf1, cf2].as_ref()
    }};
}

#[macro_export]
macro_rules! get_cf_prefix {
    ($ident1: ident, $expr1: expr, $ident2: ident, $expr2: expr) => {{
        let mut prefix = SmtPrefixType::$ident1($expr1).as_prefix();
        let added_prefix = SmtPrefixType::$ident2($expr2).as_prefix();
        prefix.extend_from_slice(&added_prefix);
        prefix
    }};

    ($ident: ident, $expr: expr) => {{
        let mut prefix = SmtPrefixType::Top.as_prefix();
        let added_prefix = SmtPrefixType::$ident($expr).as_prefix();
        prefix.extend_from_slice(&added_prefix);
        prefix
    }};
}

#[macro_export]
macro_rules! keys_to_h256 {
    ($keys: expr, $ident: ident) => {{
        $keys
            .into_iter()
            .map(|k| SmtKeyEncode::$ident(k).to_h256())
            .collect::<Vec<H256>>()
    }};
}

#[macro_export]
macro_rules! get_smt {
    ($db: expr, $cf: expr, $prefix: expr, $inner: expr) => {{
        let cf1 = format!("{}_{}", $cf.to_string(), CFSuffixType::Branch);
        let cf2 = format!("{}_{}", $cf.to_string(), CFSuffixType::Leaf);

        let cf1_handle = $db.cf_handle(&cf1).unwrap();
        let cf2_handle = $db.cf_handle(&cf2).unwrap();

        let smt = ColumnFamilyStoreMultiSMT::new_with_store(
            ColumnFamilyStoreMultiTree::<_, ()>::new($prefix, $inner, cf1_handle, cf2_handle),
        )
        .unwrap();

        smt
    }};

    ($db: expr, $cf: expr, $inner: expr) => {{
        let cf1 = format!("{}_{}", $cf.to_string(), CFSuffixType::Branch);
        let cf2 = format!("{}_{}", $cf.to_string(), CFSuffixType::Leaf);

        let cf1_handle = $db.cf_handle(&cf1).unwrap();
        let cf2_handle = $db.cf_handle(&cf2).unwrap();

        let smt = ColumnFamilyStoreSMT::new_with_store(ColumnFamilyStore::<_, ()>::new(
            $inner, cf1_handle, cf2_handle,
        ))
        .unwrap();

        smt
    }};
}

#[macro_export]
macro_rules! get_sub_leaves {
    ($ty: ty, $prefix: expr, $db: expr, $table: expr) => {{
        let prefix_len = $prefix.len();
        let key_len = prefix_len + 32;
        let mode = IteratorMode::From($prefix, Direction::Forward);
        let read_opt = ReadOptions::default();
        let cf = $db
            .cf_handle(&format!("{}_{}", $table, CFSuffixType::Leaf))
            .unwrap();
        let cf_iter = $db.get_iter_cf(cf, &read_opt, mode).unwrap();
        cf_iter
            .into_iter()
            .take_while(|(k, _)| key_len == k.len() && k[..prefix_len] == $prefix[..])
            .map(|(k, v)| {
                let leaf_key: [u8; 32] = k[prefix_len..].try_into().expect("checked 32 bytes");
                let leaf_value: [u8; 32] = v[..].try_into().expect("checked 32 bytes");
                (
                    Address::from_slice(&leaf_key[..20]),
                    <$ty>::from(LeafValue(leaf_value)),
                )
            })
            .collect::<HashMap<Address, $ty>>()
    }};
}
