#[cfg(test)]
mod tests {
    use crate::types::WyndError;

    #[test]
    fn test_wynd_error_new() {
        let error_message = "Something went wrong".to_string();
        let error = WyndError::new(error_message.clone());

        assert_eq!(error.inner, error_message);
        assert_eq!(&*error, "Something went wrong"); // Test Deref
    }

    #[test]
    fn test_wynd_error_new_empty() {
        let error = WyndError::new(String::new());

        assert_eq!(error.inner, "");
        assert_eq!(&*error, ""); // Test Deref with empty string
    }

    #[test]
    fn test_wynd_error_new_special_characters() {
        let error_message = "Error: 日本語 with émojis 🚀 and newlines\n\ttabs".to_string();
        let error = WyndError::new(error_message.clone());

        assert_eq!(error.inner, error_message);
        assert_eq!(&*error, "Error: 日本語 with émojis 🚀 and newlines\n\ttabs");
    }

    #[test]
    fn test_wynd_error_deref() {
        let error_message = "Connection failed".to_string();
        let error = WyndError::new(error_message);

        // Test direct deref
        let deref_result: &str = &*error;
        assert_eq!(deref_result, "Connection failed");

        // Test that we can use string methods through deref
        assert_eq!(error.len(), 17);
        assert!(error.contains("failed"));
        assert!(error.starts_with("Connection"));
        assert!(error.ends_with("failed"));
    }

    #[test]
    fn test_wynd_error_deref_coercion() {
        let error = WyndError::new("Test error".to_string());

        // Test that we can pass WyndError where &str is expected
        fn takes_str(s: &str) -> usize {
            s.len()
        }

        assert_eq!(takes_str(&error), 10);

        // Test with string slice methods
        let uppercase = error.to_uppercase();
        assert_eq!(uppercase, "TEST ERROR");

        let lowercase = error.to_lowercase();
        assert_eq!(lowercase, "test error");
    }

    #[test]
    fn test_wynd_error_clone_and_equality() {
        let error1 = WyndError::new("Same message".to_string());
        let error2 = WyndError::new("Same message".to_string());
        let error3 = WyndError::new("Different message".to_string());

        // Test that errors with same message have same content
        assert_eq!(&*error1, &*error2);
        assert_ne!(&*error1, &*error3);

        // Test Display equality
        assert_eq!(format!("{}", error1), format!("{}", error2));
        assert_ne!(format!("{}", error1), format!("{}", error3));
    }

    #[test]
    fn test_wynd_error_with_long_message() {
        let long_message = "A".repeat(10000);
        let error = WyndError::new(long_message.clone());

        assert_eq!(error.inner.len(), 10000);
        assert_eq!(&*error, long_message);
        assert_eq!(format!("{}", error), long_message);
    }

    #[test]
    fn test_wynd_error_memory_usage() {
        let error = WyndError::new("test".to_string());

        // WyndError should only contain a String, so size should be similar to String
        let error_size = std::mem::size_of_val(&error);
        let string_size = std::mem::size_of::<String>();

        assert_eq!(error_size, string_size);
    }

    #[test]
    fn test_wynd_error_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let error = Arc::new(WyndError::new("Thread safe error".to_string()));
        let error_clone = Arc::clone(&error);

        let handle = thread::spawn(move || format!("{}", error_clone));

        let result = handle.join().unwrap();
        assert_eq!(result, "Thread safe error");
        assert_eq!(format!("{}", error), "Thread safe error");
    }

    #[test]
    fn test_wynd_error_pattern_matching() {
        let error = WyndError::new("Pattern test".to_string());

        // Test pattern matching on the dereferenced string
        match &*error {
            "Pattern test" => assert!(true),
            _ => assert!(false, "Pattern matching failed"),
        }

        // Test with starts_with pattern
        match &*error {
            s if s.starts_with("Pattern") => assert!(true),
            _ => assert!(false, "Pattern prefix matching failed"),
        }
    }
}
