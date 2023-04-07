use std::fmt::Debug;

use rust_patch::Patch;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Item {
    data: Option<u32>,
}

#[derive(Patch)]
#[patch = "Item"]
#[patch(as_option)]
struct ItemPatchOption {
    data: Option<u32>,
}

#[derive(Patch)]
#[patch = "Item"]
#[patch(direct)]
struct ItemPatchDirect {
    data: Option<u32>,
}

fn test_patch<T: PartialEq + Debug, P: Patch<T>>(data: T, patch: P, expected: T) {
    let res = patch.apply(data);
    assert_eq!(expected, res);
}

#[test]
fn as_option() {
    test_patch(
        Item { data: Some(0) },
        ItemPatchOption { data: Some(1) },
        Item { data: Some(1) },
    );
}
