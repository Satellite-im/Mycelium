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
}
