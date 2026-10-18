use std::time::{Duration, SystemTime, UNIX_EPOCH};

use domain::{EventId, SlotBitmap, UserId};
use jsonwebtoken::{Algorithm, EncodingKey, Header};

use crate::{Claims, KeyError};

/// Issues tokens. Lives only in the gateway — the only holder of the private key.
pub struct TokenIssuer {
    key: EncodingKey,
    kid: String,
    issuer: String,
    ttl: Duration,
}

impl TokenIssuer {
    pub fn new(
        kid: impl Into<String>,
        private_pem: &str,
        issuer: impl Into<String>,
        ttl: Duration,
    ) -> Result<Self, KeyError> {
        let kid = kid.into();
        let key = EncodingKey::from_ed_pem(private_pem.as_bytes())
            .map_err(|_| KeyError::InvalidPem(kid.clone()))?;
        Ok(TokenIssuer {
            key,
            kid,
            issuer: issuer.into(),
            ttl,
        })
    }

    /// Issues a token. `nick` is the nick stored in the gateway database
    /// (the original spelling from the first join).
    pub fn issue(
        &self,
        user_id: &UserId,
        nick: &str,
        event_id: &EventId,
        slots: &SlotBitmap,
    ) -> String {
        let iat = unix_now();
        let claims = Claims {
            iss: self.issuer.clone(),
            sub: user_id.to_string(),
            nick: nick.to_owned(),
            eid: event_id.to_string(),
            slots: slots.to_base64url(),
            iat,
            exp: iat + self.ttl.as_secs(),
        };
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(self.kid.clone());
        jsonwebtoken::encode(&header, &claims, &self.key)
            .expect("signing with a successfully loaded Ed25519 key cannot fail")
    }
}

impl std::fmt::Debug for TokenIssuer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenIssuer")
            .field("kid", &self.kid)
            .field("issuer", &self.issuer)
            .field("ttl", &self.ttl)
            .finish_non_exhaustive()
    }
}

pub(crate) fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is after 1970")
        .as_secs()
}
