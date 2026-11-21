//! Opt-in native backend smoke test. Never uses the CLI's real service/user pair.
#[test]
#[ignore = "requires an unlocked native keyring session; run explicitly before release"]
fn native_keyring_round_trip() {
    struct Cleanup(keyring::Entry);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = self.0.delete_password();
        }
    }

    let unique = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let entry = Cleanup(keyring::Entry::new("togglcli-release-smoke", &unique).unwrap());
    assert!(matches!(
        entry.0.get_password(),
        Err(keyring::Error::NoEntry)
    ));
    entry.0.set_password("not-a-real-api-token").unwrap();
    assert_eq!(entry.0.get_password().unwrap(), "not-a-real-api-token");
    entry.0.delete_password().unwrap();
    assert!(matches!(
        entry.0.get_password(),
        Err(keyring::Error::NoEntry)
    ));
}
