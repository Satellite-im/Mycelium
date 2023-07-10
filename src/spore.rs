pub type Sporeprint = [u8; 32];

/// A [`Spore`] is fun way of describing a cryptographic keypair which satisfies the needs of [`Mycelium`].
pub trait Spore {
    fn sporeprint(&self) -> Sporeprint; // Used to help easily identify the Spore.
    fn sign(&self, data: &[u8]) -> Vec<u8>; // Used to sign data.
    fn verify(&self, data: &[u8], signature: &[u8]) -> bool; // Verify the signature of data signed by any Spore.
}