#![cfg(feature="oid-macro")]
//! Test the API provided to compare OIDs

use der_parser::oid;
use der_parser::oid::Oid;

const OID_RSA_ENCRYPTION: &[u8] = &oid!(raw 1.2.840.113549.1.1.1);
const OID_EC_PUBLIC_KEY: &[u8] = &oid!(raw 1.2.840.10045.2.1);
fn compare_oid(oid: &Oid) -> bool {
    match oid.bytes() {
        OID_RSA_ENCRYPTION => true,
        OID_EC_PUBLIC_KEY => true,
        _ => false,
    }
}

#[test]
fn test_compare_oid() {
    let oid = Oid::from(&[1, 2, 840, 113_549, 1, 1, 1]).unwrap();
    assert_eq!(oid, oid!(1.2.840.113549.1.1.1));
    let oid = Oid::from(&[1, 2, 840, 113_549, 1, 1, 1]).unwrap();
    assert!(compare_oid(&oid));
}
