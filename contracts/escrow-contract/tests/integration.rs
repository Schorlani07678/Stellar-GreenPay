// Cross-contract pause integration test for EscrowContract + GreenPayContract
// Deploys both compiled contracts in one Env, links them, and verifies pause behavior

#![cfg(feature = "testutils")]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
    Address, Env, String as SorobanString, Vec,
};

use escrow_contract::{EscrowContract, EscrowContractClient, Milestone};
use greenpay_contract::{GreenPayContract, GreenPayContractClient};

fn test_env() -> Env {
    Env::default()
}

#[test]
fn test_cross_contract_pause_blocks_create_job_and_release_milestone() {
    let env = test_env();
    env.mock_all_auths();

    // Deploy both contracts
    let escrow_id = env.register(EscrowContract, ());
    let escrow_client = EscrowContractClient::new(&env, &escrow_id);

    let greenpay_id = env.register(GreenPayContract, ());
    let greenpay_client = GreenPayContractClient::new(&env, &greenpay_id);

    // Initialize both contracts
    let admin = Address::generate(&env);
    escrow_client.initialize(&admin);
    greenpay_client.initialize(&admin);

    // Link them
    escrow_client.set_greenpay_contract(&admin, &greenpay_id);

    // Set up a project in GreenPay
    let project_id = SorobanString::from_str(&env, "proj-test");
    let project_wallet = Address::generate(&env);
    greenpay_client.register_project(
        &admin,
        &project_id,
        &SorobanString::from_str(&env, "Test Project"),
        &project_wallet,
        &100u32, // co2_per_xlm
        &1i128,  // min_donation
    );

    // Set up USDC token
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    // Register token in GreenPay with oracle
    let oracle_id = env.register(greenpay_contract::MockOracle, ());
    greenpay_client.set_usdc_token(&admin, &token, &oracle_id);

    // Set up escrow job participants
    let client_addr = Address::generate(&env);
    let freelancer = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&client_addr, &1000i128);
    let job_id = SorobanString::from_str(&env, "job-test");

    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone {
        name: soroban_sdk::String::from_str(&env, "M1"),
        percentage: 100,
        released: false,
    });

    // Create job before pause - should work
    escrow_client.create_job(
        &client_addr,
        &freelancer,
        &job_id,
        &token,
        &500i128,
        &milestones,
    );

    // Pause GreenPay
    greenpay_client.pause(&admin);

    // After pause, create_job should fail
    let job_id2 = SorobanString::from_str(&env, "job-paused");
    let result = escrow_client.try_create_job(
        &client_addr,
        &freelancer,
        &job_id2,
        &token,
        &500i128,
        &milestones,
    );
    assert!(
        result.is_err(),
        "create_job should fail when GreenPay is paused"
    );

    // Try release_milestone on the existing job - should also fail due to pause check
    let result = escrow_client.try_release_milestone(&client_addr, &job_id, &0u32);
    assert!(
        result.is_err(),
        "release_milestone should fail when GreenPay is paused"
    );

    // Unpause GreenPay
    greenpay_client.unpause(&admin);

    // After unpause, operations should work again
    escrow_client.release_milestone(&client_addr, &job_id, &0u32);
}
