pub trait Depacketize {
    fn depacketize(packet: &[u8]) -> Option<Self>
    where
        Self: Sized;

    #[cfg(feature = "base64")]
    fn depacketize_from_base64(b64: &str) -> Option<Self>
    where
        Self: Sized,
    {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        Self::depacketize(&decoded)
    }
}
