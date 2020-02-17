// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Secret Store.

// Parity Secret Store is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Secret Store is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Secret Store.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use log::{error, trace};
use parity_secretstore_primitives::{
	Address, ServerKeyId,
	key_server::{
		ServerKeyGenerationArtifacts, ServerKeyRetrievalArtifacts,
		DocumentKeyCommonRetrievalArtifacts, DocumentKeyShadowRetrievalArtifacts,
	},
	requester::Requester,
};
use crate::{
	Blockchain, SecretStoreCall, TransactionPool,
};

/// Substrate transction pool.
pub struct SubstrateTransactionPool<B, P> {
	/// Shared blockchain reference.
	blockchain: Arc<B>,
	/// Shared reference to actual transaction pool.
	transaction_pool: Arc<P>,
	/// This key server address.
	key_server_address: Address,
}

impl<B, P> SubstrateTransactionPool<B, P>
	where
		B: Blockchain,
		P: TransactionPool,
{
	/// Create new transaction pool.
	pub fn new(
		blockchain: Arc<B>,
		transaction_pool: Arc<P>,
		key_server_address: Address,
	) -> Self {
		SubstrateTransactionPool {
			blockchain,
			transaction_pool,
			key_server_address,
		}
	}

	/// Send response transaction if required.
	fn submit_response_transaction(
		&self,
		format_request: impl Fn() -> String,
		is_response_required: impl FnOnce() -> Result<bool, String>,
		prepare_response: impl FnOnce() -> Result<SecretStoreCall, String>,
	) {
		match is_response_required() {
			Ok(true) => (),
			Ok(false) => return,
			Err(error) => error!(
				target: "secretstore",
				"Failed to check if response {} is required: {}",
				format_request(),
				error,
			),
		}

		let submit_result = prepare_response()
			.and_then(|transaction| self
				.transaction_pool
				.submit_transaction(transaction)
			);

		match submit_result {
			Ok(transaction_hash) => trace!(
				target: "secretstore",
				"Submitted response {}: {}",
				format_request(),
				transaction_hash,
			),
			Err(error) => error!(
				target: "secretstore",
				"Failed to submit response {}: {}",
				format_request(),
				error,
			),
		}
	}
}

impl<B, P> parity_secretstore_blockchain_service::TransactionPool
	for
		SubstrateTransactionPool<B, P>
	where
		B: Blockchain,
		P: TransactionPool,
{
	fn publish_generated_server_key(
		&self,
		_origin: Address,
		key_id: ServerKeyId,
		artifacts: ServerKeyGenerationArtifacts,
	) {
		self.submit_response_transaction(
			|| format!("ServerKeyGenerationSuccess({})", key_id),
			|| self.blockchain.is_server_key_generation_response_required(key_id, self.key_server_address),
			|| Ok(SecretStoreCall::ServerKeyGenerated(key_id, artifacts.key)),
		)
	}

	fn publish_server_key_generation_error(&self, _origin: Address, key_id: ServerKeyId) {
		self.submit_response_transaction(
			|| format!("ServerKeyGenerationFailure({})", key_id),
			|| self.blockchain.is_server_key_generation_response_required(key_id, self.key_server_address),
			|| Ok(SecretStoreCall::ServerKeyGenerationError(key_id)),
		)
	}

	fn publish_retrieved_server_key(
		&self,
		_origin: Address,
		key_id: ServerKeyId,
		artifacts: ServerKeyRetrievalArtifacts,
	) {
		self.submit_response_transaction(
			|| format!("ServerKeyRetrievalSuccess({})", key_id),
			|| self.blockchain.is_server_key_retrieval_response_required(key_id, self.key_server_address),
			|| Ok(SecretStoreCall::ServerKeyRetrieved(key_id, artifacts.key)),
		)
	}

	fn publish_server_key_retrieval_error(&self, _origin: Address, key_id: ServerKeyId) {
		self.submit_response_transaction(
			|| format!("ServerKeyRetrievalFailure({})", key_id),
			|| self.blockchain.is_server_key_retrieval_response_required(key_id, self.key_server_address),
			|| Ok(SecretStoreCall::ServerKeyRetrievalError(key_id)),
		)
	}

	fn publish_stored_document_key(&self, _contract_address: Address, _key_id: ServerKeyId) {
		unimplemented!()
	}

	fn publish_document_key_store_error(&self, _contract_address: Address, _key_id: ServerKeyId) {
		unimplemented!()
	}

	fn publish_retrieved_document_key_common(
		&self,
		_contract_address: Address,
		_key_id: ServerKeyId,
		_requester: Requester,
		_artifacts: DocumentKeyCommonRetrievalArtifacts,
	) {
		unimplemented!()
	}

	fn publish_document_key_common_retrieval_error(
		&self,
		_contract_address: Address,
		_key_id: ServerKeyId,
		_requester: Requester,
	) {
		unimplemented!()
	}

	fn publish_retrieved_document_key_personal(
		&self,
		_contract_address: Address,
		_key_id: ServerKeyId,
		_requester: Requester,
		_artifacts: DocumentKeyShadowRetrievalArtifacts,
	) {
		unimplemented!()
	}

	fn publish_document_key_personal_retrieval_error(
		&self,
		_contract_address: Address,
		_key_id: ServerKeyId,
		_requester: Requester,
	) {
		unimplemented!()
	}
}
