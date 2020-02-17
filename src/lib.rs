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

use std::{
	collections::{BTreeSet, VecDeque},
	ops::Range,
	sync::Arc,
};
use futures::{Stream, StreamExt};
use log::error;
use parity_secretstore_primitives::{
	Address, KeyServerId, Public, ServerKeyId,
	error::Error,
	executor::Executor,
	key_server::KeyServer,
	requester::Requester,
	service::{ServiceTask, ServiceTasksListenerRegistrar},
};
use substrate_secret_store_runtime::{
	Event as SecretStoreEvent,
};
use crate::{
	transaction_pool::SubstrateTransactionPool,
};

// hide blockchain-service dependency
pub use parity_secretstore_blockchain_service::Configuration;

pub type BlockchainServiceTask = parity_secretstore_blockchain_service::BlockchainServiceTask;

//mod document_key_shadow_retrieval;
//mod document_key_store;
//mod server_key_generation;
//mod server_key_retrieval;
//mod services;
mod transaction_pool;

/// Substrate block id.
pub enum BlockId<Hash> {
	/// Use block referenced by the hash.
	Hash(Hash),
	/// Use best known block.
	Best,
}

/// Block event that is maybe an event coming from SecretStore runtime module.
pub trait MaybeSecretStoreEvent {
	/// Try convert to secret store event.
	fn as_secret_store_event(self) -> Option<SecretStoreEvent>;
}

/// Substrate Secret Store module calls.
pub enum SecretStoreCall {
	/// Called when server kye is generated.
	ServerKeyGenerated(ServerKeyId, Public),
}

/// Substrate blockchain.
pub trait Blockchain: 'static + Send + Sync {
	/// Block hash type.
	type BlockHash: Clone + Send + Sync;
	/// Blockchain event type.
	type Event: MaybeSecretStoreEvent;
	/// Block events iterator type.
	type BlockEvents: Iterator<Item = Self::Event>;

	/// Get block events.
	fn block_events(&self, block_hash: Self::BlockHash) -> Self::BlockEvents;
	/// Get current key servers set. This should return current key servers set at the best
	/// known (finalized) block. That's because we use this to determine key server which
	/// will should start corresponding session AND the session starts at the time when
	/// current set should have been read from the best block.
	fn current_key_servers_set(&self) -> BTreeSet<KeyServerId>;

	/// Get pending server key generation tasks range at given block.
	fn server_key_generation_tasks(
		&self,
		block_hash: Self::BlockHash,
		range: Range<usize>,
	) -> Result<Vec<BlockchainServiceTask>, String>;
	/// Is server key generation request response required?
	fn is_server_key_generation_response_required(
		&self,
		key_id: ServerKeyId,
		key_server_id: KeyServerId,
	) -> Result<bool, String>;

	/// Get pending server key retrieval tasks range at given block.
	fn server_key_retrieval_tasks(
		&self,
		block_hash: Self::BlockHash,
		range: Range<usize>,
	) -> Result<Vec<BlockchainServiceTask>, String>;

	/// Get pending document key store tasks range at given block.
	fn document_key_store_tasks(
		&self,
		block_hash: Self::BlockHash,
		range: Range<usize>,
	) -> Result<Vec<BlockchainServiceTask>, String>;

	/// Get pending document key store tasks range at given block.
	fn document_key_shadow_retrieval_tasks(
		&self,
		block_hash: Self::BlockHash,
		range: Range<usize>,
	) -> Result<Vec<BlockchainServiceTask>, String>;
}

/// Transaction pool API.
pub trait TransactionPool: Send + Sync + 'static {
	/// Transaction hash.
	type TransactionHash: std::fmt::Display;

	/// Submit transaction to the pool.
	fn submit_transaction(&self, call: SecretStoreCall) -> Result<Self::TransactionHash, String>;
}

/// Ethereum block passed to the blockchain service.
struct SubstrateBlock<B: Blockchain> {
	/// Origin block.
	pub block_hash: B::BlockHash,
	/// Shared blockchain reference.
	pub blockchain: Arc<B>,
	/// This server key address.
	pub key_server_address: Address,
}

/// Start listening requests from given contract.
pub async fn start_service<B, E, TP, KS, LR>(
	key_server: Arc<KS>,
	listener_registrar: Arc<LR>,
	blockchain: Arc<B>,
	executor: Arc<E>,
	transaction_pool: Arc<TP>,
	config: Configuration,
	new_blocks_stream: impl Stream<Item = B::BlockHash>,
) -> Result<(), Error> where
	B: Blockchain,
	E: Executor,
	TP: TransactionPool,
	LR: ServiceTasksListenerRegistrar,
	KS: KeyServer,
{
//	let config = Arc::new(config);
	let key_server_address = config.self_id;
	let transaction_pool = Arc::new(SubstrateTransactionPool::new(
		blockchain.clone(),
		transaction_pool,
		key_server_address.clone(),
	));
	parity_secretstore_blockchain_service::start_service(
		key_server,
		listener_registrar,
		executor,
		transaction_pool,
		config,
		new_blocks_stream
			.map(|block_hash| SubstrateBlock {
				block_hash,
				blockchain: blockchain.clone(),
				key_server_address: key_server_address.clone(),
			})
	).await
}

impl<B: Blockchain> parity_secretstore_blockchain_service::Block for SubstrateBlock<B> {
	type NewBlocksIterator = Box<dyn Iterator<Item = BlockchainServiceTask>>;
	type PendingBlocksIterator = Box<dyn Iterator<Item = BlockchainServiceTask>>;

	fn new_tasks(&mut self) -> Self::NewBlocksIterator {
		Box::new(
			self.blockchain
				.block_events(self.block_hash.clone())
				.into_iter()
				.filter_map(MaybeSecretStoreEvent::as_secret_store_event)
				.filter_map(event_into_task),
		)
	}

	fn pending_tasks(&mut self) -> Self::PendingBlocksIterator {
		let (blockchain, block_hash) = (self.blockchain.clone(), self.block_hash.clone());
		let server_key_generation_tasks = move |range|
			blockchain.server_key_generation_tasks(block_hash.clone(), range);
		let (blockchain, block_hash) = (self.blockchain.clone(), self.block_hash.clone());
		let server_key_retrieval_tasks = move |range|
			blockchain.server_key_retrieval_tasks(block_hash.clone(), range);
		let (blockchain, block_hash) = (self.blockchain.clone(), self.block_hash.clone());
		let document_key_store_tasks = move |range|
			blockchain.document_key_store_tasks(block_hash.clone(), range);
		let (blockchain, block_hash) = (self.blockchain.clone(), self.block_hash.clone());
		let document_key_shadow_retrieval_tasks = move |range|
			blockchain.document_key_shadow_retrieval_tasks(block_hash.clone(), range);

		Box::new(
			PendingTasksIterator {
				pending: VecDeque::new(),
				range: 0..std::usize::MAX,
				get_pending_tasks: server_key_generation_tasks,
			}.chain(PendingTasksIterator {
				pending: VecDeque::new(),
				range: 0..std::usize::MAX,
				get_pending_tasks: server_key_retrieval_tasks,
			}).chain(PendingTasksIterator {
				pending: VecDeque::new(),
				range: 0..std::usize::MAX,
				get_pending_tasks: document_key_store_tasks,
			}).chain(PendingTasksIterator {
				pending: VecDeque::new(),
				range: 0..std::usize::MAX,
				get_pending_tasks: document_key_shadow_retrieval_tasks,
			})
		)
	}

	fn current_key_servers_set(&mut self) -> BTreeSet<KeyServerId> {
		self.blockchain.current_key_servers_set()
	}
}

struct PendingTasksIterator<F> {
	pending: VecDeque<BlockchainServiceTask>,
	range: Range<usize>,
	get_pending_tasks: F,
}

impl<F> Iterator for PendingTasksIterator<F>
	where
		F: Fn(Range<usize>) -> Result<Vec<BlockchainServiceTask>, String>,
{
	type Item = BlockchainServiceTask;

	fn next(&mut self) -> Option<Self::Item> {
		const PENDING_RANGE_LENGTH: usize = 16;

		loop {
			if let Some(pending_task) = self.pending.pop_front() {
				return Some(pending_task);
			}

			if self.range.start == self.range.end {
				return None;
			}

			let next_range_start = self.range.start + PENDING_RANGE_LENGTH;
			let pending_range = self.range.start..next_range_start;
			match (self.get_pending_tasks)(pending_range) {
				Ok(tasks) => self.pending.extend(tasks),
				Err(error) => error!(
					target: "secretstore",
					"Failed to read pending tasks: {}",
					error,
				),
			}

			if self.pending.len() == PENDING_RANGE_LENGTH {
				self.range = next_range_start..self.range.end;
			} else {
				self.range = self.range.end..self.range.end;
			}
		}
	}
}

/// Convert Secret Store event to blockchain service task.
fn event_into_task(event: SecretStoreEvent) -> Option<BlockchainServiceTask> {
	// right now we only support one SS module per runtime
	// if we ever will need multiple SS modules support, then we'll probably
	// need some Fn(Module) -> Address map function
	let origin = Default::default();

	match event {
		SecretStoreEvent::ServerKeyGenerationRequested(key_id, requester_address, threshold)
			=> Some(BlockchainServiceTask::Regular(
				origin,
				ServiceTask::GenerateServerKey(
					key_id,
					Requester::Address(requester_address),
					threshold as _,
				),
			)),
		_ => unimplemented!(),
	}
}
