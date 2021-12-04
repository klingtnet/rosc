extern crate rosc;

use rosc::{OscArray, OscType};

#[test]
fn test_osc_array_from_iter() {
    use std::iter::FromIterator;
    let iter = (0..3).map(OscType::Int);
    let osc_arr = OscArray::from_iter(iter);
    assert_eq!(
        osc_arr,
        OscArray {
            content: vec![OscType::Int(0), OscType::Int(1), OscType::Int(2)]
        }
    );
}
