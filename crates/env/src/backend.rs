// Copyright (C) Use Ink (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    call::{
        Call,
        CallParams,
        ConstructorReturnType,
        CreateParams,
        DelegateCall,
        FromAddr,
        LimitParamsV2,
    },
    event::Event,
    hash::{
        CryptoHash,
        HashOutput,
    },
    DecodeDispatch,
    DispatchError,
    Result,
};
use ink_primitives::{
    types::Environment,
    H160,
    H256,
    U256,
};
use ink_storage_traits::Storable;
pub use pallet_revive_uapi::ReturnFlags;

/// Environmental contract functionality that does not require `Environment`.
pub trait EnvBackend {
    /// Writes the value to the contract storage under the given storage key.
    ///
    /// Returns the size of the pre-existing value at the specified key if any.
    fn set_contract_storage<K, V>(&mut self, key: &K, value: &V) -> Option<u32>
    where
        K: scale::Encode,
        V: Storable;

    /// Returns the value stored under the given storage key in the contract's storage if
    /// any.
    ///
    /// # Errors
    ///
    /// - If the decoding of the typed value failed
    fn get_contract_storage<K, R>(&mut self, key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable;

    /// Removes the `value` at `key`, returning the previous `value` at `key` from storage
    /// if any.
    ///
    /// # Errors
    ///
    /// - If the decoding of the typed value failed
    fn take_contract_storage<K, R>(&mut self, key: &K) -> Result<Option<R>>
    where
        K: scale::Encode,
        R: Storable;

    /// Returns the size of a value stored under the given storage key is returned if any.
    fn contains_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode;

    /// Clears the contract's storage key entry under the given storage key.
    ///
    /// Returns the size of the previously stored value at the specified key if any.
    fn clear_contract_storage<K>(&mut self, key: &K) -> Option<u32>
    where
        K: scale::Encode;

    /// Returns the execution input to the executed contract and decodes it as `T`.
    ///
    /// # Note
    ///
    /// - The input is the 4-bytes selector followed by the arguments of the called
    ///   function in their SCALE encoded representation.
    /// - No prior interaction with the environment must take place before calling this
    ///   procedure.
    ///
    /// # Usage
    ///
    /// Normally contracts define their own `enum` dispatch types respective
    /// to their exported constructors and messages that implement `scale::Decode`
    /// according to the constructors or messages selectors and their arguments.
    /// These `enum` dispatch types are then given to this procedure as the `T`.
    ///
    /// When using ink! users do not have to construct those enum dispatch types
    /// themselves as they are normally generated by the ink! code generation
    /// automatically.
    ///
    /// # Errors
    ///
    /// If the given `T` cannot be properly decoded from the expected input.
    fn decode_input<T>(&mut self) -> core::result::Result<T, DispatchError>
    where
        T: DecodeDispatch;

    /// Returns the value back to the caller of the executed contract.
    ///
    /// # Note
    ///
    /// Calling this method will end contract execution immediately.
    /// It will return the given return value back to its caller.
    ///
    /// The `flags` parameter can be used to revert the state changes of the
    /// entire execution if necessary.
    #[cfg(not(feature = "std"))]
    fn return_value<R>(&mut self, flags: ReturnFlags, return_value: &R) -> !
    where
        R: scale::Encode;

    /// Returns the value back to the caller of the executed contract.
    ///
    /// # Note
    ///
    /// When the `std` feature is used, the contract is allowed to
    /// return normally. This feature should only be used for integration tests.
    ///
    /// The `flags` parameter can be used to revert the state changes of the
    /// entire execution if necessary.
    #[cfg(feature = "std")]
    fn return_value<R>(&mut self, flags: ReturnFlags, return_value: &R)
    where
        R: scale::Encode;

    /// todo: comment
    fn return_value_rlp<R>(&mut self, flags: ReturnFlags, return_value: &R) -> !
    where
        R: alloy_rlp::Encodable;

    /// Conducts the crypto hash of the given input and stores the result in `output`.
    fn hash_bytes<H>(&mut self, input: &[u8], output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash;

    /// Conducts the crypto hash of the given encoded input and stores the result in
    /// `output`.
    fn hash_encoded<H, T>(&mut self, input: &T, output: &mut <H as HashOutput>::Type)
    where
        H: CryptoHash,
        T: scale::Encode;

    /// Recovers the compressed ECDSA public key for given `signature` and `message_hash`,
    /// and stores the result in `output`.
    fn ecdsa_recover(
        &mut self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
        output: &mut [u8; 33],
    ) -> Result<()>;

    /// Retrieves an Ethereum address from the ECDSA compressed `pubkey`
    /// and stores the result in `output`.
    fn ecdsa_to_eth_address(
        &mut self,
        pubkey: &[u8; 33],
        output: &mut [u8; 20],
    ) -> Result<()>;

    /// Verifies a sr25519 signature.
    ///
    /// # Errors
    ///
    /// - If the signature verification failed.
    ///
    /// **WARNING**: this function is from the [unstable interface](https://github.com/paritytech/substrate/tree/master/frame/contracts#unstable-interfaces),
    /// which is unsafe and normally is not available on production chains.
    fn sr25519_verify(
        &mut self,
        signature: &[u8; 64],
        message: &[u8],
        pub_key: &[u8; 32],
    ) -> Result<()>;

    /// Low-level interface to call a chain extension method.
    ///
    /// Returns the output of the chain extension of the specified type.
    ///
    /// # Errors
    ///
    /// - If the chain extension with the given ID does not exist.
    /// - If the inputs had an unexpected encoding.
    /// - If the output could not be properly decoded.
    /// - If some extension specific condition has not been met.
    ///
    /// # Developer Note
    ///
    /// A valid implementation applies the `status_to_result` closure on
    /// the status code returned by the actual call to the chain extension
    /// method.
    /// Only if the closure finds that the given status code indicates a
    /// successful call to the chain extension method is the resulting
    /// output buffer passed to the `decode_to_result` closure, in order to
    /// drive the decoding and error management process from the outside.
    fn call_chain_extension<I, T, E, ErrorCode, F, D>(
        &mut self,
        id: u32,
        input: &I,
        status_to_result: F,
        decode_to_result: D,
    ) -> ::core::result::Result<T, E>
    where
        I: scale::Encode,
        T: scale::Decode,
        E: From<ErrorCode>,
        F: FnOnce(u32) -> ::core::result::Result<(), ErrorCode>,
        D: FnOnce(&[u8]) -> ::core::result::Result<T, E>;

    /// Sets a new code hash for the current contract.
    ///
    /// This effectively replaces the code which is executed for this contract address.
    ///
    /// # Errors
    ///
    /// - If the supplied `code_hash` cannot be found on-chain.
    fn set_code_hash(&mut self, code_hash: &H256) -> Result<()>;
}

/// Environmental contract functionality.
pub trait TypedEnvBackend: EnvBackend {
    /// Returns the address of the caller of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`caller`][`crate::caller`]
    fn caller(&mut self) -> H160;

    /// Returns the transferred value for the contract execution.
    ///
    /// # Note
    ///
    /// For more details visit: [`transferred_value`][`crate::transferred_value`]
    fn transferred_value(&mut self) -> U256;

    /// Returns the price for the specified amount of gas.
    ///
    /// # Note
    ///
    /// For more details visit: [`weight_to_fee`][`crate::weight_to_fee`]
    fn weight_to_fee<E: Environment>(&mut self, gas: u64) -> E::Balance;

    /// Returns the timestamp of the current block.
    ///
    /// # Note
    ///
    /// For more details visit: [`block_timestamp`][`crate::block_timestamp`]
    fn block_timestamp<E: Environment>(&mut self) -> E::Timestamp;

    /// Returns the address of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`account_id`][`crate::account_id`]
    fn account_id<E: Environment>(&mut self) -> E::AccountId;

    /// Returns the address of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`address`][`crate::address`]
    fn address(&mut self) -> H160;

    /// Returns the balance of the executed contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`balance`][`crate::balance`]
    fn balance(&mut self) -> U256;

    /// Returns the current block number.
    ///
    /// # Note
    ///
    /// For more details visit: [`block_number`][`crate::block_number`]
    fn block_number<E: Environment>(&mut self) -> E::BlockNumber;

    /// Returns the minimum balance that is required for creating an account
    /// (i.e. the chain's existential deposit).
    ///
    /// # Note
    ///
    /// For more details visit: [`minimum_balance`][`crate::minimum_balance`]
    fn minimum_balance<E: Environment>(&mut self) -> E::Balance;

    /// Emits an event with the given event data.
    ///
    /// # Note
    ///
    /// For more details visit: [`emit_event`][`crate::emit_event`]
    fn emit_event<E, Evt>(&mut self, event: Evt)
    where
        E: Environment,
        Evt: Event;

    /// Invokes a contract message and returns its result.
    ///
    /// # Note
    ///
    /// **This will call into the latest `call_v2` host function.**
    ///
    /// For more details visit: [`invoke_contract`][`crate::invoke_contract`]
    fn invoke_contract<E, Args, R>(
        &mut self,
        call_data: &CallParams<E, Call, Args, R>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode;

    /// Invokes a contract message via delegate call and returns its result.
    ///
    /// # Note
    ///
    /// For more details visit:
    /// [`invoke_contract_delegate`][`crate::invoke_contract_delegate`]
    fn invoke_contract_delegate<E, Args, R>(
        &mut self,
        call_data: &CallParams<E, DelegateCall, Args, R>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        E: Environment,
        Args: scale::Encode,
        R: scale::Decode;

    /// Instantiates another contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`instantiate_contract`][`crate::instantiate_contract`]
    fn instantiate_contract<E, ContractRef, Args, R>(
        &mut self,
        params: &CreateParams<E, ContractRef, LimitParamsV2, Args, R>,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <R as ConstructorReturnType<ContractRef>>::Output,
        >,
    >
    where
        E: Environment,
        ContractRef: FromAddr + crate::ContractReverseReference,
        <ContractRef as crate::ContractReverseReference>::Type:
            crate::reflect::ContractConstructorDecoder,
        Args: scale::Encode,
        R: ConstructorReturnType<ContractRef>;

    /// Terminates a smart contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`terminate_contract`][`crate::terminate_contract`]
    fn terminate_contract(&mut self, beneficiary: H160) -> !;

    /// Transfers value from the contract to the destination account ID.
    ///
    /// # Note
    ///
    /// For more details visit: [`transfer`][`crate::transfer`]
    fn transfer<E>(&mut self, destination: H160, value: U256) -> Result<()>
    where
        E: Environment;

    /// Checks whether a specified contract lives at `addr`.
    ///
    /// # Note
    ///
    /// For more details visit: [`is_contract`][`crate::is_contract`]
    #[allow(clippy::wrong_self_convention)]
    fn is_contract(&mut self, account: &H160) -> bool;

    /// Checks whether the caller of the current contract is the origin of the whole call
    /// stack.
    ///
    /// # Note
    ///
    /// For more details visit: [`caller_is_origin`][`crate::caller_is_origin`]
    fn caller_is_origin<E>(&mut self) -> bool
    where
        E: Environment;

    /// Checks whether the caller of the current contract is root.
    ///
    /// # Note
    ///
    /// For more details visit: [`caller_is_root`][`crate::caller_is_root`]
    fn caller_is_root<E>(&mut self) -> bool
    where
        E: Environment;

    /// Retrieves the code hash of the contract at the given `account` id.
    ///
    /// # Note
    ///
    /// For more details visit: [`code_hash`][`crate::code_hash`]
    fn code_hash(&mut self, account: &H160) -> Result<H256>;

    /// Retrieves the code hash of the currently executing contract.
    ///
    /// # Note
    ///
    /// For more details visit: [`own_code_hash`][`crate::own_code_hash`]
    fn own_code_hash(&mut self) -> Result<H256>;

    fn call_runtime<E, Call>(&mut self, call: &Call) -> Result<()>
    where
        E: Environment,
        Call: scale::Encode;

    /// Execute an XCM message locally, using the contract's address as the origin.
    ///
    /// # Note
    ///
    /// For more details visit: [`xcm`][`crate::xcm_execute`].
    fn xcm_execute<E, Call>(&mut self, msg: &xcm::VersionedXcm<Call>) -> Result<()>
    where
        E: Environment,
        Call: scale::Encode;

    /// Send an XCM message, using the contract's address as the origin.
    ///
    /// # Note
    ///
    /// For more details visit: [`xcm`][`crate::xcm_send`].
    fn xcm_send<E, Call>(
        &mut self,
        dest: &xcm::VersionedLocation,
        msg: &xcm::VersionedXcm<Call>,
    ) -> Result<xcm::v4::XcmHash>
    where
        E: Environment,
        Call: scale::Encode;
}
