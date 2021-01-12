use crate::oauth::ExternalAccount;

pub struct UserEntity {
    pub id: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub phone_number: String,
    pub language_id: i32
}

impl UserEntity {
    pub fn from_external_account(extenral_account: &ExternalAccount) -> Self {
        UserEntity {
            first_name: extenral_account.first_name.clone(),
            last_name: extenral_account.last_name.clone(),
            email: extenral_account.email.clone(),
            phone_number: "0403231145".to_string(),
            id: None,
            language_id: 1
        }
    }
}