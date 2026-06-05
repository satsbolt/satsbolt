// Offramp Swap Plugin Interface

pub fn initialize_offramp() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offramp_init() {
        assert!(initialize_offramp());
    }
}
