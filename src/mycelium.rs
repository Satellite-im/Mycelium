use sha2::{Sha256, Digest};
use crate::{spore::Spore, errors::MyceliumError, attributes::Attribute};

pub type Hyphae = Vec<Mycelium>;

pub struct Mycelium {
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) roots: Hyphae,
}

impl Mycelium {
    pub fn new<T>(spore: T) -> Result<Self, MyceliumError>
    where T: Spore {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
            .as_nanos();

        Ok(Self {
            attributes: vec![
                Attribute::OriginSpore(spore.sporeprint()),
                Attribute::OriginMoment(now), // Represented as nanoseconds since UNIX_EPOCH.
            ],
            roots: vec![],
        })
    }
}

impl Mycelium {
    pub fn get_roots(&self) -> &Hyphae {
        &self.roots
    }

    fn get_hash(&self) -> String {
        let mut hasher = Sha256::new();

        for attr in self.get_attrs() {
            hasher.update(&attr.get_hash());
        }
        for root in self.get_roots() {
            hasher.update(root.get_hash());
        }

        hex::encode(hasher.finalize())
    }
}

impl Mycelium {
    pub fn sign<T>(&mut self, spore: &T) -> Result<(), MyceliumError> 
    where T: Spore {
        let hash = self.get_hash();

        match spore.sign(&hash.as_bytes()) {
            Ok(signature) => {
                self.attributes.push(Attribute::OriginSignature(signature));
                Ok(())
            },
            Err(e) => Err(MyceliumError::SporeError(e)),
        }
    }

    pub fn verify<T>(&self, spore: &T) -> Result<(), MyceliumError>
    where T: Spore {
        let hash = self.get_hash();

        match self.get_attr(Attribute::OriginSignature(vec![])) {
            Some(Attribute::OriginSignature(signature)) => {
                match spore.verify(&hash.as_bytes(), &signature) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(MyceliumError::SporeError(e)),
                }
            },
            _ => Err(MyceliumError::SignatureError("Missing `OriginSignature` attribute.".to_string())),
        }
    }
}