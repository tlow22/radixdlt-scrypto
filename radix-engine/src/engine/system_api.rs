use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::core::SNodeRef;
use scrypto::engine::types::*;
use scrypto::prelude::AccessRules;
use scrypto::resource::AccessRule;
use scrypto::values::*;

use crate::engine::call_frame::{DataInstruction, REValueRef, SubstateAddress};
use crate::engine::*;
use crate::fee::*;
use crate::model::*;
use crate::wasm::*;

pub trait SystemApi<'borrowed, W, I>
where
    W: WasmEngine<I>,
    I: WasmInstance,
{
    fn wasm_engine(&mut self) -> &mut W;

    fn wasm_instrumenter(&mut self) -> &mut WasmInstrumenter;

    fn cost_unit_counter(&mut self) -> &mut CostUnitCounter;

    fn fee_table(&self) -> &FeeTable;

    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn get_non_fungible(
        &mut self,
        non_fungible_address: &NonFungibleAddress,
    ) -> Option<NonFungible>;

    fn set_non_fungible(
        &mut self,
        non_fungible_address: NonFungibleAddress,
        non_fungible: Option<NonFungible>,
    );

    fn borrow_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<&ResourceManager, RuntimeError>;

    fn borrow_global_mut_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManager, RuntimeError>;

    fn return_borrowed_global_resource_manager(
        &mut self,
        resource_address: ResourceAddress,
        resource_manager: ResourceManager,
    );

    fn borrow_native_value(&mut self, value_id: &ValueId) -> REValueRef<'borrowed>;
    fn return_native_value(&mut self, value_id: ValueId, val_ref: REValueRef<'borrowed>);

    fn borrow_global_mut_value(&mut self, address: Address) -> SubstateValue;
    fn return_global_mut_value(&mut self, address: Address, value: SubstateValue);

    fn create_bucket(&mut self, container: ResourceContainer) -> Result<BucketId, RuntimeError>;

    fn take_bucket(&mut self, bucket_id: BucketId) -> Result<Bucket, RuntimeError>;

    fn create_vault(&mut self, container: ResourceContainer) -> Result<VaultId, RuntimeError>;

    fn create_proof(&mut self, proof: Proof) -> Result<ProofId, RuntimeError>;

    fn take_proof(&mut self, proof_id: ProofId) -> Result<Proof, RuntimeError>;

    fn create_resource(&mut self, resource_manager: ResourceManager) -> ResourceAddress;

    fn create_package(&mut self, package: ValidatedPackage) -> PackageAddress;

    fn globalize(
        &mut self,
        component_address: ComponentAddress,
        access_rules_list: Vec<AccessRules>,
    ) -> Result<(), RuntimeError>;

    fn create_local_component(
        &mut self,
        component: Component,
    ) -> Result<ComponentAddress, RuntimeError>;

    fn create_kv_store(&mut self) -> KeyValueStoreId;

    fn data(
        &mut self,
        address: SubstateAddress,
        instruction: DataInstruction,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn get_epoch(&mut self) -> u64;

    fn get_transaction_hash(&mut self) -> Hash;

    fn generate_uuid(&mut self) -> u128;

    fn user_log(&mut self, level: Level, message: String);

    fn check_access_rule(
        &mut self,
        access_rule: AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError>;
}
