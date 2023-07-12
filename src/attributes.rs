use sha2::{Sha256, Digest};

use crate::{spore::Sporeprint, mycelium::Mycelium, errors::MyceliumError};

#[derive(Debug, PartialEq, Hash)]
pub enum Attribute {
    OriginSpore(Sporeprint),     // Used to identify the Spore which created the Mycelium.
    OriginMoment(u128),          // Represented as nanoseconds since UNIX_EPOCH.
    OriginSignature(Vec<u8>),    // Used to verify the origin of the Mycelium.
    LastUpdate(u128),            // Represented as nanoseconds since UNIX_EPOCH. Useful for picking the most recent Mycelium when pruning.
}
impl Attribute {
    /// Compares the variants of the attributes without checking their values.
    pub fn shallow_eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    pub fn get_hash(&self) -> String {
        let mut hasher = Sha256::new();

        match self {
            Attribute::OriginSpore(sporeprint) => hasher.update(sporeprint),
            Attribute::OriginMoment(moment) => hasher.update(moment.to_string()),
            Attribute::LastUpdate(moment) => hasher.update(moment.to_string()),
            Attribute::OriginSignature(_) => (), // Ignore the signature, since it would change the hash as soon as we sign.
        }

        hex::encode(hasher.finalize())
    }
}

impl Mycelium {
    /// A list of all attributes of the Mycelium. 
    /// This list should not contain duplicates.
    pub fn get_attrs(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    /// Returns the attribute if it exists.
    pub fn get_attr(&self, attribute: Attribute) -> Option<&Attribute> {
        for attr in &self.attributes {
            if attr.shallow_eq(&attribute) {
                return Some(&attr);
            }
        }
        None
    }

    /// Returns the origin of the Mycelium.
    /// 
    /// The origin is a tuple of the Sporeprint and the moment the Mycelium was created.
    pub fn get_origin(&self) -> (Option<&Sporeprint>, Option<&u128>) {
        let mut origin = (None, None);

        for attribute in &self.attributes {
            match attribute {
                Attribute::OriginSpore(sporeprint) => origin.0 = Some(sporeprint),
                Attribute::OriginMoment(moment) => origin.1 = Some(moment),
                _ => continue,
            }
        }

        origin
    }

    pub fn has_origin(&self) -> bool {
        let mut has_spore = false;
        let mut has_moment = false;

        for attr in &self.attributes {
            match attr {
                Attribute::OriginSpore(_) => has_spore = true,
                Attribute::OriginMoment(_) => has_moment = true,
                _ => (),
            }

            // Early exit if both attributes are found
            if has_spore && has_moment {
                return true;
            }
        }

        has_spore && has_moment
    }
}

impl Mycelium {
    /// Adds an attribute to the Mycelium. This will replace existing attributes with the same variant.
    pub fn set_attr(&mut self, attribute: Attribute) -> Result<(), MyceliumError> {
        self.attributes.retain(|x_attr| !x_attr.shallow_eq(&attribute));
        self.attributes.push(attribute);

        Ok(())
    }

    /// Updates the `LastUpdate` attribute to the current time.
    pub fn set_update(&mut self) -> Result<(), MyceliumError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
            .as_nanos();

        self.set_attr(Attribute::LastUpdate(now))
    }
}