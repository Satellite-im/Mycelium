use crate::errors::SporeError;

pub type Sporeprint = String;

/// A [`Spore`] is fun way of describing a cryptographic keypair which satisfies the needs of [`Mycelium`].
pub trait Spore {
    fn new() -> Self where Self: Sized; // Used to generate a new Spore.
    fn sporeprint(&self) -> Sporeprint; // Used to help easily identify the Spore.
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SporeError>; // Used to sign data.
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), SporeError>; // Verify the signature of data signed by any Spore.
    fn resolve(sporeprint: &Sporeprint) -> Result<Self, SporeError> where Self: Sized; // Used to resolve a Sporeprint into a Spore.
}

pub mod did_spore { // Supports the standard keypair format used by Warp
    use did_key::*;
    use crate::errors::SporeError;

    use super::{Sporeprint, Spore};

    pub struct DidSpore {
        pub key: KeyPair
    }
    impl Spore for DidSpore {
        fn new() -> Self {
            let key = generate::<Ed25519KeyPair>(None);
            Self { key }
        }
        fn sporeprint(&self) -> Sporeprint {
            self.key.fingerprint()
        }
        fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SporeError> {
            Ok(self.key.sign(data))
        }
        fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), SporeError> {
            self.key.verify(data, signature).map_err(|e| SporeError::VerifyError(format!("{:?}", e)))
        }
        fn resolve(sporeprint: &Sporeprint) -> Result<Self, SporeError> {
            let key = did_key::resolve(&format!("did:key:{}", sporeprint)).map_err(|e| SporeError::ResolveError(format!("{:?}", e)))?;
            Ok(Self { key })
        }
    }
    impl TryFrom<Sporeprint> for DidSpore {
        type Error = SporeError;
        fn try_from(did: Sporeprint) -> Result<Self, Self::Error> {
            let spore = Self::resolve(&did);
            match spore {
                Ok(did_spore) => Ok(did_spore),
                Err(e) => Err(SporeError::ResolveError(format!("{:?}", e))),
            }
        }
    }
}
