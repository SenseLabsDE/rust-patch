use std::fmt::Debug;

use rust_patch::Patch;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Item {
    data: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct IdItem {
    id: u32,
    data: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct UnnamedItem(u32);

#[derive(Copy, Clone, Debug, PartialEq)]
struct UnitItem;

#[derive(Copy, Clone, Debug, PartialEq)]
struct OptItem {
    data: Option<u32>,
}

#[derive(Patch)]
#[patch = "Item"]
#[patch = "IdItem"]
#[patch = "item::ModItem"]
struct ItemPatch {
    data: Option<u32>,
}

#[derive(Patch)]
#[patch = "IdItem"]
struct DataPatch {
    data: u32,
}

#[derive(Patch)]
#[patch = "UnnamedItem"]
struct UnnamedPatch(Option<u32>);

#[derive(Patch)]
#[patch = "Item"]
#[patch = "IdItem"]
#[patch = "UnitItem"]
struct UnitPatch;

#[derive(Patch)]
#[patch = "OptItem"]
struct DirectPatch {
    #[patch(direct)]
    data: Option<u32>,
}

#[derive(Patch)]
#[patch = "OptItem"]
struct OptPatch {
    #[patch(as_option)]
    data: Option<u32>,
}

mod item {
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct ModItem {
        pub data: u32,
    }
}

fn test_patch<T: PartialEq + Debug, P: Patch<T>>(data: T, patch: P, expected: T) {
    let res = patch.apply(data);
    assert_eq!(expected, res);
}

#[test]
fn basic() {
    test_patch(
        Item { data: 0 },
        ItemPatch { data: Some(1) },
        Item { data: 1 },
    );
}

#[test]
fn empty_patch() {
    test_patch(Item { data: 0 }, ItemPatch { data: None }, Item { data: 0 });
}

#[test]
fn extra_field() {
    test_patch(
        IdItem { id: 10, data: 0 },
        ItemPatch { data: Some(1) },
        IdItem { id: 10, data: 1 },
    );
}

#[test]
fn mandatory_field() {
    test_patch(
        IdItem { id: 10, data: 0 },
        DataPatch { data: 1 },
        IdItem { id: 10, data: 1 },
    );
}

#[test]
fn unnamed_struct() {
    test_patch(UnnamedItem(0), UnnamedPatch(Some(1)), UnnamedItem(1));
}

#[test]
fn unit_struct() {
    test_patch(UnitItem, UnitPatch, UnitItem);
}

#[test]
fn patch_mod() {
    test_patch(
        item::ModItem { data: 10 },
        ItemPatch { data: Some(5) },
        item::ModItem { data: 5 },
    )
}

#[test]
fn patch_direct() {
    test_patch(
        OptItem { data: Some(1) },
        DirectPatch { data: None },
        OptItem { data: None },
    );
    test_patch(
        OptItem { data: Some(1) },
        DirectPatch { data: Some(0) },
        OptItem { data: Some(0) },
    );
}

#[test]
fn patch_as_option() {
    test_patch(
        OptItem { data: Some(1) },
        OptPatch { data: None },
        OptItem { data: Some(1) },
    );
    test_patch(
        OptItem { data: Some(1) },
        OptPatch { data: Some(0) },
        OptItem { data: Some(0) },
    );
}
