use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "trigger", content = "data")]
pub enum Event {
    #[serde(rename(deserialize = "onboarding.registrationCompleted"))]
    Registration(Registration),
    #[serde(rename(deserialize = "onboarding.onboardingCompleted"))]
    Onboarding(Onboarding),
    #[serde(rename(deserialize = "onboarding.loginCompleted"))]
    Login(Login),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub success: bool,
    pub data: Option<RegistrationData>,
    pub error_message: Option<String>,
    pub onboarding_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationData {
    #[serde(default = "registration", rename(serialize = "type"))]
    pub t: String,
    pub user_id: String,
    pub session_id: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Onboarding {
    pub success: bool,
    pub data: Option<OnboardingData>,
    pub error_message: Option<String>,
    pub onboarding_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingData {
    #[serde(default = "onboarding", rename(serialize = "type"))]
    pub t: String,
    pub user_id: String,
    pub session_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    pub success: bool,
    pub data: Option<LoginData>,
    pub session_id: Option<String>,
    pub error_message: Option<String>,
    pub onboarding_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginData {
    #[serde(default = "login", rename(serialize = "type"))]
    pub t: String,
    pub target: String,
    pub tokens: Option<serde_json::Value>,
}

fn registration() -> String {
    "registration".to_string()
}
fn onboarding() -> String {
    "onboarding".to_string()
}
fn login() -> String {
    "login".to_string()
}
