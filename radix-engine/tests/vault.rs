use radix_engine::engine::{KernelError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto::engine::types::RENodeId;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn non_existent_vault_in_component_creation_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "NonExistentVault",
            "create_component_with_non_existent_vault",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::RENodeCreateNodeNotFound(RENodeId::Vault(_)))
        )
    });
}

#[test]
fn non_existent_vault_in_committed_component_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "NonExistentVault", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(component_address, "create_non_existent_vault", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::RENodeNotFound(RENodeId::Vault(_)))
        )
    });
}

#[test]
fn non_existent_vault_in_key_value_store_creation_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "NonExistentVault",
            "create_key_value_store_with_non_existent_vault",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::RENodeNotFound(RENodeId::Vault(_)))
        )
    });
}

#[test]
fn non_existent_vault_in_committed_key_value_store_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "NonExistentVault", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(
            component_address,
            "create_non_existent_vault_in_key_value_store",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::RENodeNotFound(RENodeId::Vault(_)))
        )
    });
}

#[test]
fn create_mutable_vault_into_map() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "VaultTest", "new_vault_into_map", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn invalid_double_ownership_of_vault() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "invalid_double_ownership_of_vault",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::RENodeCreateNodeNotFound(RENodeId::Vault(_)))
        )
    });
}

#[test]
fn create_mutable_vault_into_map_and_referencing_before_storing() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_map_then_get",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_overwrite_vault_in_map() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "VaultTest", "new_vault_into_map", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(component_address, "overwrite_vault_in_map", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::StoredNodeRemoved(RENodeId::Vault(_)))
        )
    });
}

#[test]
fn create_mutable_vault_into_vector() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_vector",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_remove_vaults() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_vector",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(component_address, "clear_vector", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::StoredNodeRemoved(RENodeId::Vault(_)))
        )
    });
}

#[test]
fn can_push_vault_into_vector() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_into_vector",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component_address = receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(component_address, "push_vault_into_vector", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_take() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(package_address, "VaultTest", "new_vault_with_take", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_take_non_fungible() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_take_non_fungible",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_get_nonfungible_ids() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_non_fungible_ids",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_get_nonfungible_id() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_non_fungible_id",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_get_amount() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_amount",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_mutable_vault_with_get_resource_manager() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/vault");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "VaultTest",
            "new_vault_with_get_resource_manager",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
