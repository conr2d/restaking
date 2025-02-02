use jito_restaking_core::{
    avs::Avs, config::Config, operator::Operator, operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::vault::Vault;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::fixture::TestBuilder;

#[tokio::test]
async fn test_operator_add_vault_ok() {
    let mut fixture = TestBuilder::new().await;
    let mut restaking_program_client = fixture.restaking_program_client();

    // Initialize config
    let config_admin = Keypair::new();
    let config = Config::find_program_address(&jito_restaking_program::id()).0;
    fixture
        .transfer(&config_admin.pubkey(), 10.0)
        .await
        .unwrap();
    restaking_program_client
        .initialize_config(&config, &config_admin)
        .await
        .unwrap();

    // Initialize AVS
    let avs_admin = Keypair::new();
    let avs_base = Keypair::new();
    fixture.transfer(&avs_admin.pubkey(), 10.0).await.unwrap();
    let avs_pubkey = Avs::find_program_address(&jito_restaking_program::id(), &avs_base.pubkey()).0;
    restaking_program_client
        .initialize_avs(&config, &avs_pubkey, &avs_admin, &avs_base)
        .await
        .unwrap();

    // Initialize operator
    let base = Keypair::new();
    let operator_admin = Keypair::new();

    fixture
        .transfer(&operator_admin.pubkey(), 10.0)
        .await
        .unwrap();

    let operator_pubkey =
        Operator::find_program_address(&jito_restaking_program::id(), &base.pubkey()).0;
    restaking_program_client
        .initialize_operator(&config, &operator_pubkey, &operator_admin, &base)
        .await
        .unwrap();

    let vault_pubkey =
        Vault::find_program_address(&jito_restaking_program::id(), &Pubkey::new_unique()).0;

    // Operator adds vault
    let operator_vault_ticket = OperatorVaultTicket::find_program_address(
        &jito_restaking_program::id(),
        &operator_pubkey,
        &vault_pubkey,
    )
    .0;
    restaking_program_client
        .operator_add_vault(
            &config,
            &operator_pubkey,
            &vault_pubkey,
            &operator_vault_ticket,
            &operator_admin,
            &operator_admin,
        )
        .await
        .unwrap();

    // Verify operator state
    let operator = restaking_program_client
        .get_operator(&operator_pubkey)
        .await
        .unwrap();
    assert_eq!(operator.vault_count(), 1);

    // Verify operator vault ticket
    let ticket = restaking_program_client
        .get_operator_vault_ticket(&operator_pubkey, &vault_pubkey)
        .await
        .unwrap();
    assert_eq!(ticket.operator(), operator_pubkey);
    assert_eq!(ticket.vault(), vault_pubkey);
    assert_eq!(ticket.index(), 0);
    assert_eq!(ticket.state().slot_added(), 1);
}
