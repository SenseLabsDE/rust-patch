use rust_patch::Patch;
use serde::Deserialize;

#[derive(PartialEq, Debug)]
struct User {
    id: String,
    name: String,
    email: String,
}

#[derive(Deserialize, Patch)]
#[patch = "User"]
struct UserPatch {
    name: Option<String>,
    email: Option<String>,
}

fn main() {
    let user = User {
        id: "6bf25b70-bffa-49e0-905b-2d2e608e3abd".to_string(),
        name: "Max Mustermann".to_string(),
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
            name: "Max Mustermann".to_string(),
            email: "max.mustermann@example.com".to_string()
        }
    };
}
