use crate::{spore::Sporeprint, mycelium::Mycelium, errors::MyceliumError};

#[derive(Debug, PartialEq)]
pub enum Attribute {
    OriginSpore(Sporeprint),    // Used to identify the Spore which created the Mycelium.
    OriginMoment(u128),         // Represented as nanoseconds since UNIX_EPOCH.
    OriginSignature(Vec<u8>),   // Used to verify the origin of the Mycelium.
}
impl Attribute {
    /// Compares the variants of the attributes without checking their values.
    pub fn shallow_eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
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
}

impl Mycelium {
    pub fn set_attr(&mut self, attribute: Attribute) -> Result<(), MyceliumError> {
        self.attributes.retain(|x_attr| !x_attr.shallow_eq(&attribute));
        self.attributes.push(attribute);

        Ok(())
    }
}