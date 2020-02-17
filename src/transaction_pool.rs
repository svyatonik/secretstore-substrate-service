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
use parity_secretstore_primitives::{
	Address, ServerKeyId,
	key_server::{
		ServerKeyGenerationArtifacts, ServerKeyRetrievalArtifacts,
		DocumentKeyCommonRetrievalArtifacts, DocumentKeyShadowRetrievalArtifacts,
	},
	requester::Requester,
};
use crate::{
	Blockchain, TransactionPool,
};

/// Substrate transction pool.
pub struct SubstrateTransactionPool<B, P> {
	/// Shared blockchain reference.
	_blockchain: Arc<B>,
	/// Shared reference to actual transaction pool.
	_transaction_pool: Arc<P>,
}

impl<B, P> SubstrateTransactionPool<B, P>
	where
		B: Blockchain,
		P: TransactionPool,
{
	/// Create new transaction pool.
	pub fn new(
		_blockchain: Arc<B>,
		_transaction_pool: Arc<P>,
	) -> Self {
		SubstrateTransactionPool {
			_blockchain,
			_transaction_pool,
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
		_contract_address: Address,
		_key_id: ServerKeyId,
		_artifacts: ServerKeyGenerationArtifacts,
	) {
		unimplemented!()
	}

	fn publish_server_key_generation_error(&self, _contract_address: Address, _key_id: ServerKeyId) {
		unimplemented!()
	}

	fn publish_retrieved_server_key(
		&self,
		_contract_address: Address,
		_key_id: ServerKeyId,
		_artifacts: ServerKeyRetrievalArtifacts,
	) {
		unimplemented!()
	}

	fn publish_server_key_retrieval_error(&self, _contract_address: Address, _key_id: ServerKeyId) {
		unimplemented!()
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
