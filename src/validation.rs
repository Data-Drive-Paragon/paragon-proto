pub fn validate_destination(destination: &[u8]) -> bool {
    if destination.is_empty() {
        return false;
    }
    
    let mut has_dot = false;
    let mut prev_was_dot = false;
    
    for (i, &byte) in destination.iter().enumerate() {
        let is_upper = (b'A'..=b'Z').contains(&byte);
        let is_lower = (b'a'..=b'z').contains(&byte);
        let is_digit = (b'0'..=b'9').contains(&byte);
        let is_dot = byte == b'.';
        
        if !is_upper && !is_lower && !is_digit && !is_dot {
            return false;
        }
        
        if is_dot {
            has_dot = true;
            if prev_was_dot || i == 0 || i == destination.len() - 1 {
                return false;
            }
            prev_was_dot = true;
        } else {
            prev_was_dot = false;
        }
    }
    
    has_dot
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_destination() {
        assert!(validate_destination(b"service.node"));
        assert!(validate_destination(b"a.b.c"));
        assert!(!validate_destination(b""));
        assert!(!validate_destination(b".node"));
        assert!(!validate_destination(b"service."));
        assert!(!validate_destination(b"service..node"));
        assert!(!validate_destination(b"service node"));
        assert!(!validate_destination(b"service_node"));
    }
}
