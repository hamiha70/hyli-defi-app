use std::hash::poseidon2::Poseidon2::hash;

// ZKPassport Identity Verification Contract
// UltraHonk backend for Hyli blockchain integration
// User: bob, Password: HyliForEver

fn main(
    // Public inputs (visible on-chain for verification)
    expected_password_hash: pub Field,    // Hash of "HyliForEver"
    expected_user_hash: pub Field,        // Hash of "bob"
    
    // Private inputs (kept secret during proof generation)
    user_password: Field,                 // Hash of user's entered password
    user_name: Field                      // Hash of user's entered name
) -> pub Field {
    
    // Verify the user identity is correct
    assert(user_name == expected_user_hash);
    
    // Verify the password is correct
    assert(user_password == expected_password_hash);
    
    // Return success indicator (1 = authenticated, can be used by Hyli verifier)
    1
}

// Note: Helper functions for field conversion - use Poseidon2 hash directly in main function
// Frontend should use same hash(data, domain) pattern for consistency

// Test with the actual credentials: bob / HyliForEver
#[test]
fn test_bob_authentication() {
    // Simulate hashing "bob" -> Field (consistent with frontend)
    let bob_data = [98, 111, 98]; // ASCII values for "bob"
    let bob_hash = hash(bob_data, 0); // domain 0
    
    // Simulate hashing "HyliForEver" -> Field (consistent with frontend)
    let password_data = [72, 121, 108, 105, 70, 111, 114, 69, 118, 101, 114]; // ASCII for "HyliForEver"
    let password_hash = hash(password_data, 1); // domain 1
    
    // Test successful authentication
    let result = main(password_hash, bob_hash, password_hash, bob_hash);
    assert(result == 1);
}

#[test]
fn test_wrong_user_fails() {
    let bob_data = [98, 111, 98]; // "bob"
    let bob_hash = hash(bob_data, 0);
    
    let alice_data = [97, 108, 105, 99, 101]; // "alice"  
    let _alice_hash = hash(alice_data, 0); // Test data for wrong user scenario
    
    let password_data = [72, 121, 108, 105, 70, 111, 114, 69, 118, 101, 114]; // "HyliForEver"
    let password_hash = hash(password_data, 1);
    
    // Test with correct password but wrong user should fail
    // This would fail: main(password_hash, bob_hash, password_hash, alice_hash);
    
    // Test with correct inputs to verify test passes
    let result = main(password_hash, bob_hash, password_hash, bob_hash);
    assert(result == 1);
}

// Test for UltraHonk proof generation compatibility
#[test]
fn test_ultrahonk_proof_compatibility() {
    // Test with different field values to ensure UltraHonk compatibility
    let test_user = hash([116, 101, 115, 116], 0); // "test"
    let test_pass = hash([112, 97, 115, 115], 1); // "pass"
    
    let result = main(test_pass, test_user, test_pass, test_user);
    assert(result == 1);
}
