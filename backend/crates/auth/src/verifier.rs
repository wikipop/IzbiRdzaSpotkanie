use std::collections::HashMap;

use jsonwebtoken::{Algorithm, DecodingKey, Validation};

use crate::{AuthError, Claims, KeyError, parse_public_keys};

/// Verifies tokens locally with the public keys from `JWT_PUBLIC_KEYS`.
///
/// Accepting several keys at once lets the gateway key be rotated without
/// restarting all 168 nano services at the same moment: add the new public
/// key first, then switch the gateway, and drop the old key once the old
/// tokens have expired.
pub struct TokenVerifier {
    keys: HashMap<String, DecodingKey>,
    validation: Validation,
}

impl TokenVerifier {
    pub fn new(keys: &[(String, String)], issuer: &str) -> Result<Self, KeyError> {
        if keys.is_empty() {
            return Err(KeyError::Empty);
        }
        let mut decoding_keys = HashMap::new();
        for (kid, pem) in keys {
            let key = DecodingKey::from_ed_pem(pem.as_bytes())
                .map_err(|_| KeyError::InvalidPem(kid.clone()))?;
            if decoding_keys.insert(kid.clone(), key).is_some() {
                return Err(KeyError::DuplicateKid(kid.clone()));
            }
        }

        // The algorithm is pinned and never taken from the token header, which
        // rules out `alg: none` and HS/RS-vs-EdDSA confusion attacks.
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[issuer]);
        validation.set_required_spec_claims(&["exp", "iss", "sub"]);
        validation.validate_exp = true;

        Ok(TokenVerifier {
            keys: decoding_keys,
            validation,
        })
    }

    /// Builds a verifier from the value of `JWT_PUBLIC_KEYS`.
    pub fn from_env_spec(spec: &str, issuer: &str) -> Result<Self, KeyError> {
        Self::new(&parse_public_keys(spec)?, issuer)
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::InvalidToken("missing `kid` in the header".to_owned()))?;
        let key = self.keys.get(&kid).ok_or(AuthError::UnknownKey)?;
        let data = jsonwebtoken::decode::<Claims>(token, key, &self.validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        Ok(data.claims)
    }
}

impl std::fmt::Debug for TokenVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenVerifier")
            .field("kids", &self.keys.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}
