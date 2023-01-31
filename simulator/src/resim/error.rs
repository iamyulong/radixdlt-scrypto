use std::io;
use std::path::PathBuf;

use radix_engine::errors::{RejectionError, RuntimeError};
use radix_engine::system::package::{ExportError, ExtractAbiError};
use radix_engine::transaction::AbortReason;
use radix_engine::types::AddressError;
use radix_engine::wasm::PrepareError;
use radix_engine_interface::blueprints::resource::ParseNonFungibleGlobalIdError;
use radix_engine_interface::network::ParseNetworkError;
use sbor::*;
use transaction::errors::*;

use crate::ledger::*;
use crate::utils::*;

/// Represents a resim error.
#[derive(Debug)]
pub enum Error {
    NoDefaultAccount,

    HomeDirUnknown,

    ConfigDecodingError(sbor::DecodeError),

    IOError(io::Error),

    IOErrorAtPath(io::Error, PathBuf),

    DataError(DecodeError),

    JSONError(serde_json::Error),

    BuildError(BuildError),

    PackageAddressNotFound,

    ExtractAbiError(ExtractAbiError),

    InvalidPackage(PrepareError),

    TransactionConstructionError(BuildCallWithAbiError),

    TransactionValidationError(TransactionValidationError),

    TransactionFailed(RuntimeError),

    TransactionRejected(RejectionError),

    TransactionAborted(AbortReason),

    AbiExportError(ExportError),

    LedgerDumpError(DisplayError),

    CompileError(transaction::manifest::CompileError),

    DecompileError(transaction::manifest::DecompileError),

    InvalidId(String),

    InvalidPrivateKey,

    AddressError(AddressError),

    NonFungibleGlobalIdError(ParseNonFungibleGlobalIdError),

    FailedToBuildArgs(BuildArgsError),

    ParseNetworkError(ParseNetworkError),

    OwnerBadgeNotSpecified,
}
