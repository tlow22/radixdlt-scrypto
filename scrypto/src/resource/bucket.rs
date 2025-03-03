use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::buffer::scrypto_encode;
use crate::core::{
    BucketFnIdentifier, FnIdentifier, NativeFnIdentifier, Receiver, ResourceManagerFnIdentifier,
};
use crate::engine::types::RENodeId;
use crate::engine::{api::*, call_engine, types::BucketId};
use crate::math::*;
use crate::misc::*;
use crate::native_functions;
use crate::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ConsumingBucketBurnInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketTakeInput {
    pub amount: Decimal,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketPutInput {
    pub bucket: scrypto::resource::Bucket,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketTakeNonFungiblesInput {
    pub ids: BTreeSet<NonFungibleId>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketGetNonFungibleIdsInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketGetAmountInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketGetResourceAddressInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct BucketCreateProofInput {}

/// Represents a transient resource container.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bucket(pub BucketId);

impl Bucket {
    /// Creates a new bucket to hold resources of the given definition.
    pub fn new(resource_address: ResourceAddress) -> Self {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::ResourceManager(resource_address)),
            FnIdentifier::Native(NativeFnIdentifier::ResourceManager(
                ResourceManagerFnIdentifier::CreateBucket,
            )),
            scrypto_encode(&ResourceManagerCreateBucketInput {}),
        );
        call_engine(input)
    }

    native_functions! {
        Receiver::Consumed(RENodeId::Bucket(self.0)), NativeFnIdentifier::Bucket => {
           pub fn burn(self) -> () {
                BucketFnIdentifier::Burn,
                ConsumingBucketBurnInput {}
            }
        }
    }

    fn take_internal(&mut self, amount: Decimal) -> Self {
        let input = RadixEngineInput::InvokeMethod(
            Receiver::Ref(RENodeId::Bucket(self.0)),
            FnIdentifier::Native(NativeFnIdentifier::Bucket(BucketFnIdentifier::Take)),
            scrypto_encode(&BucketTakeInput { amount }),
        );
        call_engine(input)
    }

    native_functions! {
        Receiver::Ref(RENodeId::Bucket(self.0)), NativeFnIdentifier::Bucket => {
            pub fn take_non_fungibles(&mut self, non_fungible_ids: &BTreeSet<NonFungibleId>) -> Self {
                BucketFnIdentifier::TakeNonFungibles,
                BucketTakeNonFungiblesInput {
                    ids: non_fungible_ids.clone()
                }
            }
            pub fn put(&mut self, other: Self) -> () {
                BucketFnIdentifier::Put,
                BucketPutInput {
                    bucket: other
                }
            }
            pub fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
                BucketFnIdentifier::GetNonFungibleIds,
                BucketGetNonFungibleIdsInput {
                }
            }
            pub fn amount(&self) -> Decimal {
                BucketFnIdentifier::GetAmount,
                BucketGetAmountInput {
                }
            }
            pub fn resource_address(&self) -> ResourceAddress {
                BucketFnIdentifier::GetResourceAddress,
                BucketGetResourceAddressInput {
                }
            }
            pub fn create_proof(&self) -> scrypto::resource::Proof {
                BucketFnIdentifier::CreateProof,
                BucketCreateProofInput {
                }
            }
        }
    }

    /// Takes some amount of resources from this bucket.
    pub fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        self.take_internal(amount.into())
    }

    /// Takes a specific non-fungible from this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    pub fn take_non_fungible(&mut self, non_fungible_id: &NonFungibleId) -> Bucket {
        self.take_non_fungibles(&BTreeSet::from([non_fungible_id.clone()]))
    }

    /// Uses resources in this bucket as authorization for an operation.
    pub fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        ComponentAuthZone::push(self.create_proof());
        let output = f();
        ComponentAuthZone::pop().drop();
        output
    }

    /// Checks if this bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    pub fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns a singleton non-fungible id
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    pub fn non_fungible_id(&self) -> NonFungibleId {
        let non_fungible_ids = self.non_fungible_ids();
        if non_fungible_ids.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        self.non_fungible_ids().into_iter().next().unwrap()
    }

    /// Returns a singleton non-fungible.
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    pub fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT bucket");
        }
        non_fungibles.into_iter().next().unwrap()
    }
}

//========
// error
//========

/// Represents an error when decoding bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseBucketError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBucketError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseBucketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Bucket {
    type Error = ParseBucketError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            4 => Ok(Self(u32::from_le_bytes(copy_u8_array(slice)))),
            _ => Err(ParseBucketError::InvalidLength(slice.len())),
        }
    }
}

impl Bucket {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

scrypto_type!(Bucket, ScryptoType::Bucket, Vec::new());
