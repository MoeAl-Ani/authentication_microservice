use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Deserialize, Serialize, Debug)]
pub struct Link {
    pub rel: String,
    pub href: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SrpStep1Request {
    pub identity: String,
    pub public_a_str: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SrpStep1Response {
    pub public_b_str: String,
    pub salt_str: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SrpStep2Request {
    pub identity: String,
    pub m1_str: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SrpStep2Response {
    pub m2_str: String,
}