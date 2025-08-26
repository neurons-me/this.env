use this_env::env::{Env, EnvType, TrustLevel};

#[test]
fn test_env_operations() {
    let base_path = std::path::Path::new(".this/env");
    let mut env = Env::new(
        "example.com".to_string(),
        "env123".to_string(),
        EnvType::RemoteWeb,
        TrustLevel::High,
    );

    env.set_metadata("location", "datacenter");
    env.add_endorsement("alice", "sig123");
    assert!(env.save(base_path).is_ok());
    assert_eq!(env.get_metadata("location"), Some(&"datacenter".to_string()));
    assert!(env.is_endorsed_by("alice"));
}