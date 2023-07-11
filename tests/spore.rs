#[cfg(test)]
mod tests {
    use mycelium::spore::{self, Spore};

    #[test]
    fn test_creation() {
        let did_spore = spore::did_spore::DidSpore::new();
        let sporeprint = did_spore.sporeprint();

        assert_ne!(sporeprint, "");
    }

    #[test]
    fn test_sign() {
        let did_spore = spore::did_spore::DidSpore::new();
        
        let data = "Hello, world!";
        let signature = did_spore.sign(data.as_bytes()).unwrap();

        assert_ne!(signature, vec![]);
    }

    #[test]
    fn test_verify() {
        let did_spore = spore::did_spore::DidSpore::new();
        
        let data = "Hello, world!";
        let signature = did_spore.sign(data.as_bytes()).unwrap();

        assert_ne!(signature, vec![]);

        did_spore.verify(data.as_bytes(), &signature).unwrap();
    }

    #[test]
    fn test_to_from_sporeprint() {
        let did_spore = spore::did_spore::DidSpore::new();
        let did_sporeprint = did_spore.sporeprint();

        println!("did_sporeprint: {}", did_sporeprint);

        let data = "Hello, world!";
        let signature = did_spore.sign(data.as_bytes()).unwrap();

        // Shadow the previous did_spore and instead resolve from the sporeprint. (This should also strip the private keys)
        let did_spore = spore::did_spore::DidSpore::resolve(&did_sporeprint).unwrap();

        did_spore.verify(data.as_bytes(), &signature).unwrap(); // Verify the signature of data signed by the spore, using the resolved sporeprint.
    }
}