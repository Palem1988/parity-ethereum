// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use engine::Engine;
use machine::{
	ExecutedBlock,
	Machine
};
use types::{
	header::Header,
	engines::{
		Seal,
		SealingState,
		params::CommonParams,
	},
	errors::EthcoreError as Error,
};


/// `InstantSeal` params.
#[derive(Default, Debug, PartialEq)]
pub struct InstantSealParams {
	/// Whether to use millisecond timestamp
	pub millisecond_timestamp: bool,
}

impl From<::ethjson::spec::InstantSealParams> for InstantSealParams {
	fn from(p: ::ethjson::spec::InstantSealParams) -> Self {
		InstantSealParams {
			millisecond_timestamp: p.millisecond_timestamp,
		}
	}
}

/// An engine which does not provide any consensus mechanism, just seals blocks internally.
/// Only seals blocks which have transactions.
pub struct InstantSeal {
	params: InstantSealParams,
	machine: Machine,
}

impl InstantSeal {
	/// Returns new instance of InstantSeal over the given state machine.
	pub fn new(params: InstantSealParams, machine: Machine) -> Self {
		InstantSeal {
			params,
			machine,
		}
	}
}

impl Engine for InstantSeal {
	fn name(&self) -> &str {
		"InstantSeal"
	}

	fn machine(&self) -> &Machine { &self.machine }

	fn sealing_state(&self) -> SealingState { SealingState::Ready }

	fn generate_seal(&self, block: &ExecutedBlock, _parent: &Header) -> Seal {
		if block.transactions.is_empty() {
			Seal::None
		} else {
			Seal::Regular(Vec::new())
		}
	}

	fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {
		Ok(())
	}

	fn open_block_header_timestamp(&self, parent_timestamp: u64) -> u64 {
		use std::{time, cmp};

		let dur = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap_or_default();
		let mut now = dur.as_secs();
		if self.params.millisecond_timestamp {
			now = now * 1000 + dur.subsec_millis() as u64;
		}
		cmp::max(now, parent_timestamp)
	}

	fn is_timestamp_valid(&self, header_timestamp: u64, parent_timestamp: u64) -> bool {
		header_timestamp >= parent_timestamp
	}

	fn params(&self) -> &CommonParams {
		self.machine.params()
	}

}


#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use ethereum_types::{H520, Address};
	use test_helpers::get_temp_state_db;
	use crate::spec;
	use types::{
		header::Header,
		engines::Seal,
	};
	use block::*;

	#[test]
	fn instant_can_seal() {
		let spec = spec::new_instant();
		let engine = &*spec.engine;
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let genesis_header = spec.genesis_header();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, Address::zero(), (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b = b.close_and_lock().unwrap();
		if let Seal::Regular(seal) = engine.generate_seal(&b, &genesis_header) {
			assert!(b.try_seal(engine, seal).is_ok());
		}
	}

	#[test]
	fn instant_cant_verify() {
		let engine = spec::new_instant().engine;
		let mut header: Header = Header::default();

		assert!(engine.verify_block_basic(&header).is_ok());

		header.set_seal(vec![::rlp::encode(&H520::default())]);

		assert!(engine.verify_block_unordered(&header).is_ok());
	}
}
