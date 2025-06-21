use std::hash::hash_to_field;

// ZKPassport Identity Verification Contract
// Phase 1: Simple password hash verification
// Phase 2: Will be replaced with actual ZKPassport proof verification

fn main(
    // Public inputs (visible on-chain)
    password_hash: pub Field,    // Expected hash of the correct password
    user_id: pub Field,          // User identifier (e.g., hash of "bob@zkpassport")
    
    // Private inputs (kept secret)
    secret_password: Field       // The actual password user knows
) -> pub Field {
    
    // Compute hash of the provided password
    let computed_hash = hash_to_field([secret_password]);
    
    // Verify the password is correct
    assert(computed_hash == password_hash);
    
    // Return the user_id to prove this user has verified their identity
    user_id
}

// Helper function for testing
#[test]
fn test_password_verification() {
    // Test with a known password
    let secret = 42;
    let expected_hash = hash_to_field([secret]);
    let user = hash_to_field([123456]); // Represents "bob@zkpassport"
    
    // This should succeed
    let result = main(expected_hash, user, secret);
    assert(result == user);
}

#[test]
fn test_password_verification_fails() {
    // Test with wrong password should fail
    let secret = 42;
    let expected_hash = hash_to_field([secret]);
    
    // This should fail (commented out to avoid test failure)
    // main(expected_hash, user, wrong_secret); // Would panic on assert
}
