#[cfg(test)]
mod tests {
    use crate::Args;
    
    #[test]
    fn test_tracing_args_parsing() {
        // Test that the tracing endpoint and auth can be parsed from command line arguments
        let args = Args::parse_from([
            "cabinet",
            "--tracing-endpoint", "http://localhost:4317",
            "--tracing-auth", "test-token"
        ]);
        
        assert_eq!(args.tracing_endpoint, Some("http://localhost:4317".to_string()));
        assert_eq!(args.tracing_auth, Some("test-token".to_string()));
    }
    
    // Note: We can't easily test the actual OpenTelemetry integration in unit tests
    // as it requires a running tracing backend. This would be better tested in
    // integration tests or manual verification.
}