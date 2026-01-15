mod common;

use common::TraqTestEnvironment;
use infra::traq_client::TraqClientImpl;
use domain::traq_client::TraqClient;
use uuid::Uuid;

#[tokio::test]
#[ignore]  // Ignore by default, run with --ignored
async fn test_environment_starts() {
    let env = TraqTestEnvironment::start()
        .await
        .expect("Failed to start traQ environment");
    
    assert!(!env.base_url().is_empty());
    assert!(!env.admin_token().is_empty());
    
    println!("traQ URL: {}", env.base_url());
    println!("Admin token: {}", env.admin_token());
}

#[tokio::test]
#[ignore]
async fn test_get_user_with_real_traq() {
    let env = TraqTestEnvironment::start()
        .await
        .expect("Failed to start traQ environment");
    
    let client = TraqClientImpl::new(env.base_url().to_string());
    
    // Test with random UUID (should get 404)
    let result: Result<_, _> = client.get_user(env.admin_token(), &Uuid::now_v7()).await;
    
    // Should error (user doesn't exist)
    assert!(result.is_err());
}
