use sha2::{Sha256, Digest};
use crate::{spore::{Spore, Sporeprint}, errors::MyceliumError, attributes::Attribute};

/// When reffering to a collection of Mycelium, we call it Hyphae. As opposed to saying a mycelium of a mycelium of a mycelium, we ask if it's within the Hyphae, and if so, how deep?
pub type Hyphae = Vec<Mycelium>;

pub struct Mycelium {
    // Attributes of the Mycelium such as the origin and last update.
    pub(crate) attributes: Vec<Attribute>,
    // Child roots of the Mycelium which are other Myceliums. 
    pub(crate) hyphae: Hyphae,
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
            hyphae: vec![], 
        })
    }
}

impl Mycelium {
    /// Returns all child Mycelium.
    pub fn get_hyphae(&self) -> &Hyphae {
        &self.hyphae
    }

    /// Returns a string representing the hash of the Mycelium.
    pub fn get_hash(&self) -> String {
        let mut hasher = Sha256::new();

        for attr in self.get_attrs() {
            hasher.update(&attr.get_hash());
        }
        for root in self.get_hyphae() {
            hasher.update(root.get_hash());
        }

        hex::encode(hasher.finalize())
    }

    /// Scans for the first occurance of a sporeprint in the Mycelium.
    /// 
    /// In the future this could be made much more complex but the simple implementation now is to return if the sporeprint is found in the current Mycelium or any of its child roots.
    /// If it's found it returns the depth at which it's found with `1` being the current Mycelium and `2` being any mycelium within the first "layer" of child roots.
    /// 
    /// If the sporeprint is not found, it returns `None`.
    /// 
    /// If the depth is specified, it will only search that many layers deep.
    pub fn scan(&self, sporeprint: &Sporeprint, depth: Option<u8>) -> Result<Option<u8>, MyceliumError> {
        match depth {
            Some(d) if d == 0 => return Ok(None), // If we've reached the max depth, return None.
            _ => (),
        }

        for root in self.get_hyphae() {
            match root.get_origin() {
                (Some(x_sporeprint), _) if x_sporeprint == sporeprint => return Ok(Some(depth.unwrap_or(0))),
                _ => (), // For now we will ignore spores without an origin for ease.
            }
            // If we didn't find the sporeprint in the current root, search its roots.
            if let Some(d) = depth {
                match root.scan(sporeprint, Some(d - 1))? {
                    Some(_) => return Ok(Some(d)),
                    None => (),
                }
            } else {
                match root.scan(sporeprint, None)? {
                    Some(_) => return Ok(Some(0)),
                    None => (),
                }
            }
        }

        Ok(None) // If we didn't find the sporeprint anywhere, return None.
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

impl Mycelium {
    pub fn add(&mut self, mycelium: Mycelium) -> Result<(), MyceliumError> {
        let new_sporeprint = match mycelium.get_origin() {
            (Some(sporeprint), Some(_)) => sporeprint,
            _ => "",
        };

        self.attributes.retain(|x_mycelium| {
            match x_mycelium {
                Attribute::OriginSpore(sporeprint) => sporeprint != new_sporeprint,
                _ => true,
            }
        });

        self.hyphae.push(mycelium);

        self.set_update()?;

        Ok(())
    }
}