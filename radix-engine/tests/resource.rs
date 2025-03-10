use radix_engine::engine::{ApplicationError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::model::ResourceManagerError;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_set_mintable_with_self_resource_address() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, _) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/resource");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "ResourceTest",
            "set_mintable_with_self_resource_address",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_resource_manager() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/resource");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "ResourceTest", "create_fungible", args!())
        .call_function(package_address, "ResourceTest", "query", args!())
        .call_function(package_address, "ResourceTest", "burn", args!())
        .call_function(
            package_address,
            "ResourceTest",
            "update_resource_metadata",
            args!(),
        )
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn mint_with_bad_granularity_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/resource");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible_and_mint",
            args!(0u8, dec!("0.1")),
        )
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(
            ResourceManagerError::InvalidAmount(amount, granularity),
        )) = e
        {
            amount.eq(&Decimal::from("0.1")) && *granularity == 0
        } else {
            false
        }
    });
}

#[test]
fn mint_too_much_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/resource");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible_and_mint",
            args!(0u8, dec!("1000000000000000001")),
        )
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(
                ResourceManagerError::MaxMintAmountExceeded
            ))
        )
    })
}
