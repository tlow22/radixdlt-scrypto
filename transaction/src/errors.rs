use sbor::describe::Type;
use sbor::rust::string::String;
use sbor::*;
use scrypto::component::{ComponentAddress, PackageAddress};
use scrypto::engine::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderValidationError {
    UnknownVersion(u8),
    InvalidEpochRange,
    EpochRangeTooLarge,
    OutOfEpochRange,
    InvalidNetwork,
    InvalidCostUnitLimit,
    InvalidTipBps,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {
    TooManySignatures,
    InvalidIntentSignature,
    InvalidNotarySignature,
    DuplicateSigner,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum IdAllocationError {
    OutOfID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdValidationError {
    IdAllocationError(IdAllocationError),
    BucketNotFound(BucketId),
    ProofNotFound(ProofId),
    BucketLocked(BucketId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallDataValidationError {
    DecodeError(DecodeError),
    IdValidationError(IdValidationError),
    VaultNotAllowed(VaultId),
    KeyValueStoreNotAllowed(KeyValueStoreId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionValidationError {
    TransactionTooLarge,
    DeserializationError(DecodeError),
    IntentHashRejected,
    HeaderValidationError(HeaderValidationError),
    SignatureValidationError(SignatureValidationError),
    IdValidationError(IdValidationError),
    CallDataValidationError(CallDataValidationError),
}

/// Represents an error when parsing arguments.
#[derive(Debug, Clone)]
pub enum BuildArgsError {
    /// The argument is not provided.
    MissingArgument(usize, Type),

    /// The argument is of unsupported type.
    UnsupportedType(usize, Type),

    UnsupportedRootType(Type),

    /// Failure when parsing an argument.
    FailedToParse(usize, Type, String),

    /// Failed to interpret this string as a resource specifier
    InvalidResourceSpecifier(String),
}

/// Represents an error when building a transaction.
#[derive(Debug, Clone)]
pub enum BuildCallWithAbiError {
    /// The given blueprint function does not exist.
    FunctionNotFound(String),

    /// The given component method does not exist.
    MethodNotFound(String),

    /// The provided arguments do not match ABI.
    FailedToBuildArgs(BuildArgsError),

    /// Failed to export the ABI of a function.
    FailedToExportFunctionAbi(PackageAddress, String, String),

    /// Failed to export the ABI of a method.
    FailedToExportMethodAbi(ComponentAddress, String),

    /// Account is required but not provided.
    AccountNotProvided,
}
