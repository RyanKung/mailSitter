#[cfg(feature = "ddep")]
mod test_ggep {
    use crate::ddep::get_otp_via_mail;

    #[test]
    fn test_duckduckgo_mail_reges() {
        let mail = "To continue, open this link in your browser:\r\n\r\nhttps://duckduckgo.com/email/login?otp=unstamped-matching-onboard-proofs&user=0xhack1984\r\n\r\nOr, enter this one-time passphrase in your open DuckDuckGo tab:\r\n\r\nunstamped matching onboard proofs\r\n\r\nIf you didn’t expect this email, someone may have accidentally entered your Duck\r\nAddress when attempting to enable Email Protection in their browser. If needed,\r\nyou can reach us at support@duck.com.\r\n";
        assert_eq!(
            get_otp_via_mail(mail),
            Some("unstamped matching onboard proofs".to_string())
        )
    }
}
