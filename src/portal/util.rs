use rand::{Rng, distr::Alphanumeric};
use zbus::zvariant::Value;

pub struct SessionToken(String);

impl SessionToken {
    pub fn new() -> Self {
        let token: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let session_token = format!("whydotool_{token}");

        Self(session_token)
    }
}

impl<'a> From<SessionToken> for Value<'a> {
    fn from(token: SessionToken) -> Self {
        Value::from(token.0)
    }
}
