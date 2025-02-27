use minicbor::{CborLen, Decode, Encode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

#[derive(Encode, Decode, CborLen, Serialize, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cbor(transparent)]
#[serde(transparent)]
pub struct Token(#[n(0)] pub String);

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("{TOKEN}")
    }
}

impl Token {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
}

pub mod auth0 {
    use super::*;
    use crate::orchestrator::email_address::EmailAddress;
    use std::fmt::{Display, Formatter};

    // Req/Res types

    #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
    pub struct DeviceCode<'a> {
        pub device_code: Cow<'a, str>,
        pub user_code: Cow<'a, str>,
        pub verification_uri: Cow<'a, str>,
        pub verification_uri_complete: Cow<'a, str>,
        pub expires_in: usize,
        pub interval: usize,
    }

    #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
    pub struct AuthorizationCode {
        pub code: String,
    }

    impl AuthorizationCode {
        pub fn new(s: impl Into<String>) -> Self {
            Self { code: s.into() }
        }
    }

    #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
    pub struct TokensError<'a> {
        pub error: Cow<'a, str>,
        pub error_description: Cow<'a, str>,
    }

    #[derive(serde::Deserialize, Debug, Clone)]
    #[cfg_attr(test, derive(PartialEq, Eq))]
    pub struct OidcToken {
        pub token_type: TokenType,
        pub access_token: Token,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
    pub struct UserInfo {
        pub sub: String,
        pub nickname: String,
        pub name: String,
        pub picture: String,
        pub updated_at: String,
        pub email: EmailAddress,
        pub email_verified: bool,
    }

    impl Display for UserInfo {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("UserInfo")
                .field("email", &self.sub)
                .field("nickname", &self.nickname)
                .field("picture", &self.picture)
                .field("updated_at", &self.updated_at)
                .field("email", &self.email)
                .field("email_verified", &self.email_verified)
                .finish()
        }
    }

    #[derive(Encode, Decode, CborLen, Debug)]
    #[cfg_attr(test, derive(Clone))]
    #[rustfmt::skip]
    #[cbor(map)]
    pub struct AuthenticateOidcToken {
        #[n(1)] pub token_type: TokenType,
        #[n(2)] pub access_token: Token,
    }

    impl AuthenticateOidcToken {
        pub fn new(token: OidcToken) -> Self {
            Self {
                token_type: token.token_type,
                access_token: token.access_token,
            }
        }
    }

    // Auxiliary types

    #[derive(serde::Deserialize, Encode, Decode, CborLen, Debug, Clone)]
    #[cfg_attr(test, derive(PartialEq, Eq))]
    #[rustfmt::skip]
    #[cbor(index_only)]
    pub enum TokenType {
        #[n(0)] Bearer,
    }
}

pub mod enrollment_token {
    use serde::Serialize;
    use std::fmt::{Display, Formatter};

    use ockam::identity::Attributes;

    use super::*;

    // Main req/res types

    #[derive(Encode, Debug)]
    #[cfg_attr(test, derive(Decode, Clone))]
    #[rustfmt::skip]
    #[cbor(map)]
    pub struct RequestEnrollmentToken {
        #[b(1)] pub attributes: Attributes,
    }

    impl RequestEnrollmentToken {
        pub fn new(attributes: Attributes) -> Self {
            Self { attributes }
        }
    }

    #[derive(Encode, Decode, CborLen, Serialize, Debug)]
    #[cfg_attr(test, derive(Clone))]
    #[rustfmt::skip]
    #[cbor(map)]
    pub struct EnrollmentToken {
        #[n(1)] pub token: Token,
    }

    impl Display for EnrollmentToken {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.token.0)
        }
    }

    impl EnrollmentToken {
        pub fn new(token: Token) -> Self {
            Self { token }
        }
    }

    #[derive(Encode, Debug)]
    #[cfg_attr(test, derive(Decode, Clone))]
    #[rustfmt::skip]
    #[cbor(map)]
    pub struct AuthenticateEnrollmentToken {
        #[n(1)] pub token: Token,
    }

    impl AuthenticateEnrollmentToken {
        pub fn new(token: EnrollmentToken) -> Self {
            Self { token: token.token }
        }
    }
}
