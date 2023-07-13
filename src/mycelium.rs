use either::Either::{self, Left};
use sha2::{Sha256, Digest};
use crate::{spore::{Spore, Sporeprint}, errors::MyceliumError, attributes::Attribute};

pub struct Mycelium {
    // Attributes of the Mycelium such as the origin and last update.
    pub(crate) attributes: Vec<Attribute>,
    // Child roots of the Mycelium which are other Myceliums. 
    pub(crate) mycelia: Vec<Either<Mycelium, Sporeprint>>,
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
            mycelia: vec![],
        })
    }
}

impl Mycelium {
    /// Returns all child Mycelium.
    pub fn get_mycelia(&self) -> &Vec<Either<Mycelium, Sporeprint>> {
        &self.mycelia
    }

    /// Returns a string representing the hash of the Mycelium.
    pub fn get_hash(&self) -> String {
        let mut hasher = Sha256::new();

        for attr in self.get_attrs() {
            hasher.update(&attr.get_hash());
        }
        for root in self.get_mycelia() {
            match root {
                Either::Left(mycelium) => hasher.update(&mycelium.get_hash()),
                Either::Right(sporeprint) => hasher.update(sporeprint),
            }
        }
        
        hex::encode(hasher.finalize())
    }

    /// Scans for the first occurance of a sporeprint in the Mycelium.
    /// 
    /// Return an integer representing the depth (at which it was found) if the sporeprint is indeed found within the Hyphae.
    /// If the sporeprint is not found, it returns `None`.
    /// 
    /// If the depth is specified, it will only search that many layers deep.
    /// 
    /// This is pretty simple right now but serves the purpose of the MVP.
    pub fn scan(&self, sporeprint: &Sporeprint, depth: Option<u8>) -> Result<Option<u8>, MyceliumError> {
        match depth {
            Some(d) if d == 0 => return Ok(None), // If we've reached the max depth, return None.
            _ => (),
        }
    
        for root in self.get_mycelia() {
            match root {
                Either::Left(mycelium) => {
                    match mycelium.get_origin() {
                        (Some(x_sporeprint), _) if x_sporeprint == sporeprint => return Ok(Some(depth.unwrap_or(0))),
                        _ => (), // For now we will ignore spores without an origin for ease.
                    }
                    // If we didn't find the sporeprint in the current root, search its roots.
                    if let Some(d) = depth {
                        match mycelium.scan(sporeprint, Some(d - 1))? {
                            Some(_) => return Ok(Some(d)),
                            None => (),
                        }
                    } else {
                        match mycelium.scan(sporeprint, None)? {
                            Some(_) => return Ok(Some(0)),
                            None => (),
                        }
                    }
                },
                Either::Right(sp) if sp == sporeprint => return Ok(Some(depth.unwrap_or(0))),
                _ => (),
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

        self.mycelia.push(Left(mycelium));

        self.set_update()?;

        Ok(())
    }

    /// This method prunes the Mycelium's child Mycelia in two steps:
    ///
    /// 1. It retains only those mycelia that have an origin. If a mycelium doesn't have an origin (neither OriginSpore nor OriginMoment), it's removed.
    /// 2. For each remaining mycelia, it keeps only the first occurrence of a mycelium from a given origin.
    ///    If there are multiple mycelia with the same origin at the same depth, it keeps the one with the most recent update,
    ///    and replaces all others with their corresponding Sporeprint.
    ///
    /// This way, it ensures that each origin is represented by a single, most recent mycelium and removes redundancy.
    pub fn prune(&mut self) -> Result<(), MyceliumError> {
        // Retain only those Myceliums that have an origin.
        self.mycelia.retain(|root| {
            match root {
                Either::Left(mycelium) => mycelium.has_origin(),
                Either::Right(_) => true,
            }
        });

        // Use a HashMap to keep track of seen sporeprints and their most recent update and index.
        let mut seen: std::collections::HashMap<String, (usize, u128)> = std::collections::HashMap::new();
        // Collect indices that need to be replaced into a vector.
        let mut replace_indices = Vec::new();

        // Iterate over the mycelia.
        for i in 0..self.mycelia.len() {
            match &self.mycelia[i] {
                Either::Left(mycelium) => {
                    match mycelium.get_origin() {
                        (Some(sporeprint), Some(moment)) => {
                            // If we've seen this sporeprint before and the current mycelium is more recent, 
                            // add the index of the older mycelium to the replace_indices vector.
                            if let Some((index, prev_moment)) = seen.get(sporeprint) {
                                if *moment > *prev_moment {
                                    replace_indices.push(*index);
                                    seen.insert(sporeprint.clone(), (i, *moment));
                                } else {
                                    // If the current mycelium is not more recent, add its index to the replace_indices vector.
                                    replace_indices.push(i);
                                }
                            } else {
                                // If it's the first time we've seen this sporeprint, add it to the seen HashMap.
                                seen.insert(sporeprint.clone(), (i, *moment));
                            }
                        },
                        _ => (),
                    }
                },
                Either::Right(_) => (),
            }
        }

        // Iterate over replace_indices and replace the corresponding mycelium with a sporeprint.
        for i in replace_indices {
            match &self.mycelia[i] {
                Either::Left(mycelium) => {
                    if let (Some(sporeprint), _) = mycelium.get_origin() {
                        self.mycelia[i] = Either::Right(sporeprint.clone());
                    }
                },
                Either::Right(_) => (),
            }
        }

        Ok(())
    }
}