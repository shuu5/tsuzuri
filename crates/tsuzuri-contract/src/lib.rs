//! 契約の型: server と面と hook が共有する型（便 b で置く）。この便では空の骨格。

#[cfg(test)]
mod tests {
    #[test]
    fn skeleton_contract_crate_is_in_workspace() {
        assert_eq!(env!("CARGO_PKG_NAME"), "tsuzuri-contract");
    }
}
