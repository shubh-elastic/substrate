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

//! Treasury pallet tests.

#![cfg(test)]

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BadOrigin, BlakeTwo256, IdentityLookup},
};

use frame_support::{
	assert_noop, assert_ok,
	pallet_prelude::GenesisBuild,
	parameter_types,
	traits::{ConstU32, ConstU64, OnInitialize},
	PalletId,
};

use super::*;
use crate as treasury;
use frame_support::traits::AsEnsureOriginWithArg;
use frame_system::EnsureRoot;
use pallet_assets::{self};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Treasury: treasury::{Pallet, Call, Storage, Config, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Config<T>, Storage, Event<T>},
	}
);

type AccountId = u128;

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId; // u64 is not enough to hold bytes used to generate bounty account
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

#[derive(
	Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, MaxEncodedLen, Debug, TypeInfo,
)]
pub enum HoldIdentifier {
	Treasury,
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type HoldIdentifier = HoldIdentifier;
	type MaxHolds = ConstU32<10>;
}

type AssetId = u32;

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	// type Balance = Balance;
	type Balance = u64;
	type AssetId = AssetId;
	type AssetIdParameter = AssetId;
	type Currency = Balances;
	type HoldIdentifier = HoldIdentifier;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = ConstU64<2>;
	type AssetAccountDeposit = ConstU64<2>;
	type MetadataDepositBase = ConstU64<0>;
	type MetadataDepositPerByte = ConstU64<0>;
	type ApprovalDeposit = ConstU64<0>;
	type StringLimit = ConstU32<20>;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<1000>;
	pallet_assets::runtime_benchmarks_enabled! {
		type BenchmarkHelper = ();
	}
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const HoldReason: HoldIdentifier = HoldIdentifier::Treasury;
}
pub struct TestSpendOrigin;
impl frame_support::traits::EnsureOrigin<RuntimeOrigin> for TestSpendOrigin {
	type Success = u64;
	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		Result::<frame_system::RawOrigin<_>, RuntimeOrigin>::from(o).and_then(|o| match o {
			frame_system::RawOrigin::Root => Ok(u64::max_value()),
			frame_system::RawOrigin::Signed(10) => Ok(5),
			frame_system::RawOrigin::Signed(11) => Ok(10),
			frame_system::RawOrigin::Signed(12) => Ok(20),
			frame_system::RawOrigin::Signed(13) => Ok(50),
			r => Err(RuntimeOrigin::from(r)),
		})
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::root())
	}
}

pub struct AssetBalances;
impl crate::AccountBalances<AccountId, u64, AssetId> for AssetBalances {
	fn account_balances(account: AccountId) -> Vec<(u32, u64)> {
		Assets::account_balances(account)
	}
}

impl Config for Test {
	type PalletId = TreasuryPalletId;
	// type Currency = pallet_balances::Pallet<Test>;
	type Assets = pallet_assets::Pallet<Test>;
	type AccountBalances = AssetBalances;
	type AssetId = AssetId;
	type HoldAssetReason = HoldReason;
	type ApproveOrigin = frame_system::EnsureRoot<u128>;
	type RejectOrigin = frame_system::EnsureRoot<u128>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ConstU64<1>;
	type ProposalBondMaximum = ();
	type SpendPeriod = ConstU64<2>;
	type Burn = Burn;
	type BurnDestination = (); // Just gets burned.
	type WeightInfo = ();
	type SpendFunds = ();
	type MaxApprovals = ConstU32<100>;
	type SpendOrigin = TestSpendOrigin;
	type HoldReason = HoldReason;
}

impl<T: Config<I>, I: 'static> SpendFunds<T, I> for () {}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![(0, 100), (1, 98), (2, 1)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	GenesisBuild::<Test>::assimilate_storage(&crate::GenesisConfig, &mut t).unwrap();
	t.into()
}

// #[test]
// fn genesis_config_works() {
// 	new_test_ext().execute_with(|| {
// 		assert_eq!(Treasury::pot(), 0);
// 		assert_eq!(Treasury::proposal_count(), 0);

// 		assert_eq!(Balances::free_balance(0), 100);
// 		assert_eq!(Balances::free_balance(1), 98);
// 		assert_eq!(Balances::free_balance(2), 1);
// 	});
// }

// #[test]
// fn spend_origin_permissioning_works() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(Treasury::spend(RuntimeOrigin::signed(1), 1, 1), BadOrigin);
// 		assert_noop!(
// 			Treasury::spend(RuntimeOrigin::signed(10), 6, 1),
// 			Error::<Test>::InsufficientPermission
// 		);
// 		assert_noop!(
// 			Treasury::spend(RuntimeOrigin::signed(11), 11, 1),
// 			Error::<Test>::InsufficientPermission
// 		);
// 		assert_noop!(
// 			Treasury::spend(RuntimeOrigin::signed(12), 21, 1),
// 			Error::<Test>::InsufficientPermission
// 		);
// 		assert_noop!(
// 			Treasury::spend(RuntimeOrigin::signed(13), 51, 1),
// 			Error::<Test>::InsufficientPermission
// 		);
// 	});
// }

#[test]
fn spend_origin_works() {
	new_test_ext().execute_with(|| {
		// Check that accumulate works when we have Some value in Dummy already.
		// Balances::set_balance(&Treasury::account_id(), 101);
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, Treasury::account_id(), true, 1));
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 1, Treasury::account_id(), true, 1));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Treasury::account_id()),
			0,
			Treasury::account_id(),
			100
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Treasury::account_id()),
			1,
			Treasury::account_id(),
			100
		));
		assert_eq!(Treasury::pot(), vec![(0, 100), (1, 100)]); // The missing 1 unit is the ExistentialDeposit

		assert_ok!(Treasury::spend(RuntimeOrigin::signed(10), 0, 5, 6));
		assert_ok!(Treasury::spend(RuntimeOrigin::signed(10), 0, 5, 6));
		assert_ok!(Treasury::spend(RuntimeOrigin::signed(10), 0, 5, 6));
		assert_ok!(Treasury::spend(RuntimeOrigin::signed(10), 0, 5, 6));
		assert_ok!(Treasury::spend(RuntimeOrigin::signed(11), 0, 10, 6));
		assert_ok!(Treasury::spend(RuntimeOrigin::signed(12), 0, 20, 6));
		assert_ok!(Treasury::spend(RuntimeOrigin::signed(13), 0, 50, 6));
		// // Pot should be unchanged until `on_initialize` and hence `spend_funds` runs.
		assert_eq!(Treasury::pot(), [(0, 100), (1, 100)]);
		assert_eq!(Assets::balance(0, Treasury::account_id()), 100);
		// assert_eq!(Balances::free_balance(6), 0);

		<Treasury as OnInitialize<u64>>::on_initialize(1);
		assert_eq!(Treasury::pot(), [(0, 100), (1, 100)]);
		assert_eq!(Assets::balance(0, Treasury::account_id()), 100);
		// assert_eq!(Balances::free_balance(6), 0);

		<Treasury as OnInitialize<u64>>::on_initialize(2);
		assert_eq!(Treasury::pot(), [(0, 100), (1, 100)]);
		assert_eq!(Assets::balance(0, Treasury::account_id()), 100);
		// assert_eq!(Balances::free_balance(6), 100);
		// assert_eq!(Treasury::pot(), 0);
	});
}

// #[test]
// fn minting_works() {
// 	new_test_ext().execute_with(|| {
// 		// Check that accumulate works when we have Some value in Dummy already.
// 		Balances::set_balance(&Treasury::account_id(), 101);
// 		assert_eq!(Treasury::pot(), 100);
// 	});
// }

// #[test]
// fn spend_proposal_takes_min_deposit() {
// 	new_test_ext().execute_with(|| {
// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 1, 3));
// 		assert_eq!(Balances::free_balance(0), 99);
// 		assert_eq!(Balances::reserved_balance(0), 1);
// 	});
// }

// #[test]
// fn spend_proposal_takes_proportional_deposit() {
// 	new_test_ext().execute_with(|| {
// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 100, 3));
// 		assert_eq!(Balances::free_balance(0), 95);
// 		assert_eq!(Balances::reserved_balance(0), 5);
// 	});
// }

// #[test]
// fn spend_proposal_fails_when_proposer_poor() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(
// 			Treasury::propose_spend(RuntimeOrigin::signed(2), 100, 3),
// 			Error::<Test, _>::InsufficientProposersBalance,
// 		);
// 	});
// }

// #[test]
// fn accepted_spend_proposal_ignored_outside_spend_period() {
// 	new_test_ext().execute_with(|| {
// 		Balances::set_balance(&Treasury::account_id(), 101);

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 100, 3));
// 		assert_ok!(Treasury::approve_proposal(RuntimeOrigin::root(), 0));

// 		<Treasury as OnInitialize<u64>>::on_initialize(1);
// 		assert_eq!(Balances::free_balance(3), 0);
// 		assert_eq!(Treasury::pot(), 100);
// 	});
// }

// #[test]
// fn unused_pot_should_diminish() {
// 	new_test_ext().execute_with(|| {
// 		let init_total_issuance = Balances::total_issuance();
// 		Balances::set_balance(&Treasury::account_id(), 101);
// 		assert_eq!(Balances::total_issuance(), init_total_issuance + 100);

// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Treasury::pot(), 50);
// 		assert_eq!(Balances::total_issuance(), init_total_issuance + 50);
// 	});
// }

// #[test]
// fn rejected_spend_proposal_ignored_on_spend_period() {
// 	new_test_ext().execute_with(|| {
// 		Balances::set_balance(&Treasury::account_id(), 101);

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 100, 3));
// 		assert_ok!(Treasury::reject_proposal(RuntimeOrigin::root(), 0));

// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Balances::free_balance(3), 0);
// 		assert_eq!(Treasury::pot(), 50);
// 	});
// }

// #[test]
// fn reject_already_rejected_spend_proposal_fails() {
// 	new_test_ext().execute_with(|| {
// 		Balances::set_balance(&Treasury::account_id(), 101);

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 100, 3));
// 		assert_ok!(Treasury::reject_proposal(RuntimeOrigin::root(), 0));
// 		assert_noop!(
// 			Treasury::reject_proposal(RuntimeOrigin::root(), 0),
// 			Error::<Test, _>::InvalidIndex
// 		);
// 	});
// }

// #[test]
// fn reject_non_existent_spend_proposal_fails() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(
// 			Treasury::reject_proposal(RuntimeOrigin::root(), 0),
// 			Error::<Test, _>::InvalidIndex
// 		);
// 	});
// }

// #[test]
// fn accept_non_existent_spend_proposal_fails() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(
// 			Treasury::approve_proposal(RuntimeOrigin::root(), 0),
// 			Error::<Test, _>::InvalidIndex
// 		);
// 	});
// }

// #[test]
// fn accept_already_rejected_spend_proposal_fails() {
// 	new_test_ext().execute_with(|| {
// 		Balances::set_balance(&Treasury::account_id(), 101);

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 100, 3));
// 		assert_ok!(Treasury::reject_proposal(RuntimeOrigin::root(), 0));
// 		assert_noop!(
// 			Treasury::approve_proposal(RuntimeOrigin::root(), 0),
// 			Error::<Test, _>::InvalidIndex
// 		);
// 	});
// }

// #[test]
// fn accepted_spend_proposal_enacted_on_spend_period() {
// 	new_test_ext().execute_with(|| {
// 		Balances::set_balance(&Treasury::account_id(), 101);
// 		assert_eq!(Treasury::pot(), 100);

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 100, 3));
// 		assert_ok!(Treasury::approve_proposal(RuntimeOrigin::root(), 0));

// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Balances::free_balance(3), 100);
// 		assert_eq!(Treasury::pot(), 0);
// 	});
// }

// #[test]
// fn pot_underflow_should_not_diminish() {
// 	new_test_ext().execute_with(|| {
// 		Balances::set_balance(&Treasury::account_id(), 101);
// 		assert_eq!(Treasury::pot(), 100);

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 150, 3));
// 		assert_ok!(Treasury::approve_proposal(RuntimeOrigin::root(), 0));

// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Treasury::pot(), 100); // Pot hasn't changed

// 		let _ = Balances::mint_into(&Treasury::account_id(), 100).unwrap();
// 		<Treasury as OnInitialize<u64>>::on_initialize(4);
// 		assert_eq!(Balances::free_balance(3), 150); // Fund has been spent
// 		assert_eq!(Treasury::pot(), 25); // Pot has finally changed
// 	});
// }

// // Treasury account doesn't get deleted if amount approved to spend is all its free balance.
// // i.e. pot should not include existential deposit needed for account survival.
// #[test]
// fn treasury_account_doesnt_get_deleted() {
// 	new_test_ext().execute_with(|| {
// 		Balances::set_balance(&Treasury::account_id(), 101);
// 		assert_eq!(Treasury::pot(), 100);
// 		let treasury_balance = Balances::free_balance(&Treasury::account_id());

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), treasury_balance, 3));
// 		assert_ok!(Treasury::approve_proposal(RuntimeOrigin::root(), 0));

// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Treasury::pot(), 100); // Pot hasn't changed

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), Treasury::pot(), 3));
// 		assert_ok!(Treasury::approve_proposal(RuntimeOrigin::root(), 1));

// 		<Treasury as OnInitialize<u64>>::on_initialize(4);
// 		assert_eq!(Treasury::pot(), 0); // Pot is emptied
// 		assert_eq!(Balances::free_balance(Treasury::account_id()), 1); // but the account is still there
// 	});
// }

// // In case treasury account is not existing then it works fine.
// // This is useful for chain that will just update runtime.
// #[test]
// fn inexistent_account_works() {
// 	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
// 	pallet_balances::GenesisConfig::<Test> { balances: vec![(0, 100), (1, 99), (2, 1)] }
// 		.assimilate_storage(&mut t)
// 		.unwrap();
// 	// Treasury genesis config is not build thus treasury account does not exist
// 	let mut t: sp_io::TestExternalities = t.into();

// 	t.execute_with(|| {
// 		assert_eq!(Balances::free_balance(Treasury::account_id()), 0); // Account does not exist
// 		assert_eq!(Treasury::pot(), 0); // Pot is empty

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 99, 3));
// 		assert_ok!(Treasury::approve_proposal(RuntimeOrigin::root(), 0));
// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 1, 3));
// 		assert_ok!(Treasury::approve_proposal(RuntimeOrigin::root(), 1));
// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Treasury::pot(), 0); // Pot hasn't changed
// 		assert_eq!(Balances::free_balance(3), 0); // Balance of `3` hasn't changed

// 		Balances::set_balance(&Treasury::account_id(), 100);
// 		assert_eq!(Treasury::pot(), 99); // Pot now contains funds
// 		assert_eq!(Balances::free_balance(Treasury::account_id()), 100); // Account does exist

// 		<Treasury as OnInitialize<u64>>::on_initialize(4);

// 		assert_eq!(Treasury::pot(), 0); // Pot has changed
// 		assert_eq!(Balances::free_balance(3), 99); // Balance of `3` has changed
// 	});
// }

// #[test]
// fn genesis_funding_works() {
// 	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
// 	let initial_funding = 100;
// 	pallet_balances::GenesisConfig::<Test> {
// 		// Total issuance will be 200 with treasury account initialized with 100.
// 		balances: vec![(0, 100), (Treasury::account_id(), initial_funding)],
// 	}
// 	.assimilate_storage(&mut t)
// 	.unwrap();
// 	GenesisBuild::<Test>::assimilate_storage(&crate::GenesisConfig, &mut t).unwrap();
// 	let mut t: sp_io::TestExternalities = t.into();

// 	t.execute_with(|| {
// 		assert_eq!(Balances::free_balance(Treasury::account_id()), initial_funding);
// 		assert_eq!(Treasury::pot(), initial_funding - <Balances as fungible::Inspect<<Test as
// frame_system::Config>::AccountId>>::minimum_balance()); 	});
// }

// #[test]
// fn max_approvals_limited() {
// 	new_test_ext().execute_with(|| {
// 		Balances::set_balance(&Treasury::account_id(), u32::MAX.into());
// 		Balances::set_balance(&0, u32::MAX.into());

// 		for _ in 0..<Test as Config>::MaxApprovals::get() {
// 			assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 100, 3));
// 			assert_ok!(Treasury::approve_proposal(RuntimeOrigin::root(), 0));
// 		}

// 		// One too many will fail
// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 100, 3));
// 		assert_noop!(
// 			Treasury::approve_proposal(RuntimeOrigin::root(), 0),
// 			Error::<Test, _>::TooManyApprovals
// 		);
// 	});
// }

// #[test]
// fn remove_already_removed_approval_fails() {
// 	new_test_ext().execute_with(|| {
// 		Balances::set_balance(&Treasury::account_id(), 101);

// 		assert_ok!(Treasury::propose_spend(RuntimeOrigin::signed(0), 100, 3));
// 		assert_ok!(Treasury::approve_proposal(RuntimeOrigin::root(), 0));
// 		assert_eq!(Treasury::approvals(), vec![0]);
// 		assert_ok!(Treasury::remove_approval(RuntimeOrigin::root(), 0));
// 		assert_eq!(Treasury::approvals(), vec![]);

// 		assert_noop!(
// 			Treasury::remove_approval(RuntimeOrigin::root(), 0),
// 			Error::<Test, _>::ProposalNotApproved
// 		);
// 	});
// }