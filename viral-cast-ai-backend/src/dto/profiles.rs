use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// macro_rules! serde_skip_if_none {
//     ($struct_name:ident { $($field:ident : $type:ty),* $(,)? }) => {
//         #[derive(Serialize, Deserialize, Debug)]
//         pub struct $struct_name {
//             $(
//                 #[serde(skip_serializing_if = "Option::is_none")]
//                 pub $field: $type,
//             )*
//         }
//     };
// }

// #[derive(Serialize, Deserialize, Debug)]
// serde_skip_if_none!(CreateProfileSchema {
//     pub name: Option<String>,
//     pub photo_profile: Option<String>,
//     pub background_profile: Option<String>,
//     pub gender: Option<String>,
//     pub telp: Option<String>,
//     pub birth_date: Option<String>,

//     pub roles_number: Option<i32>,
//     pub store_name: Option<String>,
//     pub store_telp: Option<String>,
// });

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProfileSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_place: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_uuid: Option<Uuid>,
    // Region codes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub district_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub village_code: Option<String>,
    // Neighborhood codes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProfileSchema {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub photo_profile: Option<String>,
    pub background_profile: Option<String>,
    pub gender: Option<String>,
    pub telp: Option<String>,
    pub birth_date: Option<String>,
    pub birth_place: Option<String>,
    pub roles_number: Option<i32>,
    pub store_uuid: Option<Uuid>,
    // Region codes
    pub province_code: Option<String>,
    pub regency_code: Option<String>,
    pub district_code: Option<String>,
    pub village_code: Option<String>,
    // Neighborhood codes
    pub rt: Option<String>,
    pub rw: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct GetProfilesSchema {
    pub uuid: Uuid,
    pub user_uuid: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub photo_profile: Option<String>,
    pub background_profile: Option<String>,
    pub gender: Option<String>,
    pub telp: Option<String>,
    pub birth_date: Option<String>,
    pub birth_place: Option<String>,
    pub roles_number: Option<i32>,
    pub roles_name: Option<String>,
    pub store_uuid: Option<Uuid>,
    // Region codes
    pub province_code: Option<String>,
    pub regency_code: Option<String>,
    pub district_code: Option<String>,
    pub village_code: Option<String>,
    // Neighborhood codes
    pub rt: Option<String>,
    pub rw: Option<String>,
    pub postal_code: Option<String>,
    // pub created_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedProfile {
    pub uuid: Uuid,
    pub user_uuid: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub photo_profile: Option<String>,
    pub background_profile: Option<String>,
    pub gender: Option<String>,
    pub telp: Option<String>,
    pub birth_date: Option<String>,
    pub birth_place: Option<String>,
    pub roles: Roles,
    pub store_uuid: Option<Uuid>,
    // Region codes
    pub province_code: Option<String>,
    pub regency_code: Option<String>,
    pub district_code: Option<String>,
    pub village_code: Option<String>,
    // Neighborhood codes
    pub rt: Option<String>,
    pub rw: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Roles {
    pub number: Option<i32>,
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, json, to_string};
    use uuid::Uuid;

    #[test]
    fn profiles_processed_profile_serde() {
        let roles = Roles {
            number: Some(2),
            name: Some("Manager".to_string()),
        };
        let profile = ProcessedProfile {
            uuid: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap(),
            user_uuid: Some(Uuid::new_v4()),
            first_name: Some("Budi".to_string()),
            last_name: Some("Santoso".to_string()),
            photo_profile: Some("photo.png".to_string()),
            background_profile: Some("bg.png".to_string()),
            gender: Some("male".to_string()),
            telp: Some("081234567890".to_string()),
            birth_date: Some("1990-01-01".to_string()),
            birth_place: Some("Jakarta".to_string()),
            roles,
            store_uuid: None,
            province_code: Some("31".to_string()),
            regency_code: Some("3174".to_string()),
            district_code: Some("317401".to_string()),
            village_code: Some("31740101".to_string()),
            rt: Some("01".to_string()),
            rw: Some("02".to_string()),
            postal_code: Some("12345".to_string()),
        };

        let s = to_string(&profile).expect("serialize");
        let v: serde_json::Value = from_str(&s).expect("deserialize to value");
        assert_eq!(v["uuid"], json!("123e4567-e89b-12d3-a456-426614174000"));
        assert_eq!(v["first_name"], json!("Budi"));
        assert_eq!(v["last_name"], json!("Santoso"));
        assert_eq!(v["roles"]["number"], json!(2));
        assert_eq!(v["roles"]["name"], json!("Manager"));
        assert_eq!(v["province_code"], json!("31"));
        assert_eq!(v["postal_code"], json!("12345"));
    }

    #[test]
    fn profiles_roles_clone_and_serde() {
        let r1 = Roles {
            number: Some(1),
            name: Some("Administrator".to_string()),
        };
        let r2 = r1.clone();
        assert_eq!(r1.number, r2.number);
        assert_eq!(r1.name, r2.name);

        let s = to_string(&r1).unwrap();
        let v: serde_json::Value = from_str(&s).unwrap();
        assert_eq!(v, json!({"number":1, "name":"Administrator"}));
    }
}
