use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::ledger::*;
use radix_engine::transaction::TransactionExecutor;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine::wasm::WasmInstrumenter;
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::signing::EcdsaSecp256k1PrivateKey;

fn bench_transfer(c: &mut Criterion) {
    // Set up environment.
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut wasm_instrumenter = WasmInstrumenter::new();
    let mut executor = TransactionExecutor::new(
        &mut substate_store,
        &mut wasm_engine,
        &mut wasm_instrumenter,
    );

    // Create a key pair
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
        .call_method(SYS_FAUCET_COMPONENT, "free_xrd", args!())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.new_account_with_resource(
                &rule!(require(NonFungibleAddress::from_public_key(&public_key))),
                bucket_id,
            )
        })
        .build();
    let account1 = executor
        .execute_and_commit(
            &TestTransaction::new(manifest.clone(), 1, vec![public_key.into()]),
            &FeeReserveConfig::standard(),
            &ExecutionConfig::default(),
        )
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];
    let account2 = executor
        .execute_and_commit(
            &TestTransaction::new(manifest, 2, vec![public_key.into()]),
            &FeeReserveConfig::standard(),
            &ExecutionConfig::default(),
        )
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Fill first account
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
        .call_method(SYS_FAUCET_COMPONENT, "free_xrd", args!())
        .call_method(
            account1,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    for nonce in 0..1000 {
        executor
            .execute_and_commit(
                &TestTransaction::new(manifest.clone(), nonce, vec![public_key.into()]),
                &FeeReserveConfig::standard(),
                &ExecutionConfig::default(),
            )
            .expect_commit();
    }

    // Create a transfer manifest
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(100.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account_by_amount(dec!("0.000001"), RADIX_TOKEN, account1)
        .call_method(
            account2,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();

    // Loop
    let mut nonce = 3;
    c.bench_function("Transfer", |b| {
        b.iter(|| {
            let receipt = executor.execute_and_commit(
                &TestTransaction::new(manifest.clone(), nonce, vec![public_key.into()]),
                &FeeReserveConfig::standard(),
                &ExecutionConfig::default(),
            );
            receipt.expect_commit_success();
            nonce += 1;
        })
    });
}

criterion_group!(radix_engine, bench_transfer);
criterion_main!(radix_engine);
