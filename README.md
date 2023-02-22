# rust-patch
Patch structs with other structs

`rust-patch` allows you to avoid boilerplate code when implementing partial updates of Rust structs.  
Simply define a patch struct containing a subset of your fields, derive the `Patch` trait,
and specify the original struct using the `#[patch]` attribute.  
Fields of a patch struct may either be of the same type `T` as in the original struct or `Option<T>`.  
In the latter case, the field to be patched will be left unchanged if the corresponding field in the patch is `None`

This crate is `no_std` compatible.
```rust
use rust_patch::Patch;
use serde::Deserialize;

#[derive(PartialEq, Debug)]
struct User {
    id: String,
    display_name: String,
    email: String,
}

#[derive(Deserialize, Patch)]
#[patch = "User"]
struct UserPatch {
    display_name: Option<String>,
    email: Option<String>,
}

let user = User {
    id: "6bf25b70-bffa-49e0-905b-2d2e608e3abd".to_string(),
    display_name: "Max Mustermann".to_string(),
    email: "max.mustermann@example.org".to_string(),
};

let raw_patch = r#"{
    "id": "some invalid id",
    "email": "max.mustermann@example.com"
}"#;

let patch: UserPatch = serde_json::from_str(raw_patch).unwrap();
let patched_user = patch.apply(user);

// Since `id` is not part of our `UserPatch` struct it stays unchanged
assert_eq! {
    patched_user,
    User {
        id: "6bf25b70-bffa-49e0-905b-2d2e608e3abd".to_string(),
        display_name: "Max Mustermann".to_string(),
        email: "max.mustermann@example.com".to_string()
    }
};
```