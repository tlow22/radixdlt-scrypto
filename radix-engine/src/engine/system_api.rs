use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::core::{Network, SNodeRef};
use scrypto::engine::types::*;
use scrypto::resource::AccessRule;
use scrypto::values::*;

use crate::engine::values::*;
use crate::engine::*;
use crate::fee::*;
use crate::wasm::*;

use super::call_frame::REValueRef;

pub trait SystemApi<'p, W, I>
where
    W: WasmEngine<I>,
    I: WasmInstance,
{
    fn cost_unit_counter(&mut self) -> &mut CostUnitCounter;

    fn fee_table(&self) -> &FeeTable;

    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn globalize_value(&mut self, value_id: &ValueId) -> Result<(), CostUnitCounterError>;

    fn borrow_value(&mut self, value_id: &ValueId) -> Result<REValueRef<'_>, CostUnitCounterError>;

    fn borrow_value_mut(
        &mut self,
        value_id: &ValueId,
    ) -> Result<RENativeValueRef, CostUnitCounterError>;

    fn return_value_mut(&mut self, val_ref: RENativeValueRef) -> Result<(), CostUnitCounterError>;

    fn drop_value(&mut self, value_id: &ValueId) -> Result<REValue, CostUnitCounterError>;

    fn create_value<V: Into<REValueByComplexity>>(&mut self, v: V)
        -> Result<ValueId, RuntimeError>;

    fn read_value_data(&mut self, address: SubstateAddress) -> Result<ScryptoValue, RuntimeError>;

    fn write_value_data(
        &mut self,
        address: SubstateAddress,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError>;

    fn remove_value_data(&mut self, address: SubstateAddress)
        -> Result<ScryptoValue, RuntimeError>;

    fn transaction_hash(&mut self) -> Result<Hash, CostUnitCounterError>;

    fn transaction_network(&mut self) -> Result<Network, CostUnitCounterError>;

    fn generate_uuid(&mut self) -> Result<u128, CostUnitCounterError>;

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), CostUnitCounterError>;

    fn check_access_rule(
        &mut self,
        access_rule: AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError>;

    fn pay_fee(&mut self, vault_id: VaultId, amount: Decimal) -> Result<(), RuntimeError>;
}
