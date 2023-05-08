// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Update `CodeStorage` with the new `determinism` field.

use crate::{
	migration::{IsFinished, Migrate},
	weights::WeightInfo,
	CodeHash, Config, Determinism, Pallet, Weight, LOG_TARGET,
};
use codec::{Decode, Encode};
use frame_support::{
	codec, pallet_prelude::*, storage_alias, BoundedVec, DefaultNoBound, Identity,
};
use sp_std::{marker::PhantomData, prelude::*};

mod old {
	use super::*;

	#[derive(Encode, Decode)]
	pub struct PrefabWasmModule {
		#[codec(compact)]
		pub instruction_weights_version: u32,
		#[codec(compact)]
		pub initial: u32,
		#[codec(compact)]
		pub maximum: u32,
		pub code: Vec<u8>,
	}

	#[storage_alias]
	pub type CodeStorage<T: Config> =
		StorageMap<Pallet<T>, Identity, CodeHash<T>, PrefabWasmModule>;
}

#[cfg(feature = "runtime-benchmarks")]
pub fn store_old_dummy_code<T: Config>(len: usize) {
	use sp_runtime::traits::Hash;
	let module = old::PrefabWasmModule {
		instruction_weights_version: 0,
		initial: 0,
		maximum: 0,
		code: vec![42u8; len],
	};
	let hash = T::Hashing::hash(&module.code);
	old::CodeStorage::<T>::insert(hash, module);
}

#[derive(Encode, Decode)]
struct PrefabWasmModule {
	#[codec(compact)]
	pub instruction_weights_version: u32,
	#[codec(compact)]
	pub initial: u32,
	#[codec(compact)]
	pub maximum: u32,
	pub code: Vec<u8>,
	pub determinism: Determinism,
}

#[storage_alias]
type CodeStorage<T: Config> = StorageMap<Pallet<T>, Identity, CodeHash<T>, PrefabWasmModule>;

#[derive(Encode, Decode, MaxEncodedLen, DefaultNoBound)]
pub struct Migration<T: Config> {
	last_key: Option<BoundedVec<u8, ConstU32<256>>>,
	_phantom: PhantomData<T>,
}

impl<T: Config> Migrate for Migration<T> {
	const VERSION: u16 = 9;

	fn max_step_weight() -> Weight {
		T::WeightInfo::v9_translate_wasm_module(T::MaxCodeLen::get())
	}

	fn step(&mut self) -> (IsFinished, Weight) {
		let mut iter = if let Some(last_key) = self.last_key.take() {
			old::CodeStorage::<T>::iter_from(last_key.to_vec())
		} else {
			old::CodeStorage::<T>::iter()
		};

		if let Some((key, old)) = iter.next() {
			log::debug!(target: LOG_TARGET, "Migrating contract code {:?}", key);
			let len = old.code.len() as u32;
			let module = PrefabWasmModule {
				instruction_weights_version: old.instruction_weights_version,
				initial: old.initial,
				maximum: old.maximum,
				code: old.code,
				determinism: Determinism::Enforced,
			};
			CodeStorage::<T>::insert(key, module);
			self.last_key = Some(iter.last_raw_key().to_vec().try_into().unwrap());
			(IsFinished::No, T::WeightInfo::v9_translate_wasm_module(len))
		} else {
			log::debug!(target: LOG_TARGET, "No more contracts code to migrate");
			(IsFinished::Yes, T::WeightInfo::v9_translate_wasm_module(0))
		}
	}
}