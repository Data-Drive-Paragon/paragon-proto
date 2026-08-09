use paragon_proto::*;

#[test]
fn test_integration_version_and_validation() {
    assert_eq!(proto_version(), 1);
    assert!(validate_destination(b"core.system"));
    assert!(!validate_destination(b"invalid destination"));
}
