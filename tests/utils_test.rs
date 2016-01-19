extern crate rosc;

#[test]
fn test_pad() {
    assert_eq!(4, rosc::utils::pad(4));
    assert_eq!(8, rosc::utils::pad(5));
    assert_eq!(8, rosc::utils::pad(6));
    assert_eq!(8, rosc::utils::pad(7));
}
