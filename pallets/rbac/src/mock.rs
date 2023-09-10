use crate::{
	self as pallet_rbac,
	primitives::{CallMetadata, ModuleCallIndex},
	tests_utils::*,
	traits::GetCallMetadataIndecies,
	AccountRoles, AccountRolesListOf, CallRoles, RoleDispatchOrigin, RoleInfoOf, Roles as RolesMap,
};
use codec::Encode;
use frame_support::{
	traits::{ConstU128, ConstU16, ConstU32, ConstU64, GetCallIndex, PalletInfoAccess},
	Hashable,
};
pub(crate) use frame_system::{Call as SystemCall, EnsureRoot, RawOrigin};
pub(crate) use pallet_balances::Call as BalancesCall;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type AccountId = u64;
type Balance = u128;

pub(crate) const ALICE: AccountId = 1;
pub(crate) const BOB: AccountId = 2;
pub(crate) const NICK: AccountId = 3;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Roles: pallet_rbac,
		Balances: pallet_balances,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}

impl pallet_rbac::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	// type WeightInfo = ();
	type ManageOrigin = EnsureRoot<AccountId>;
	type RoleNameLengthLimit = ConstU32<50>;
	type RolesPerAccountLimit = ConstU32<20>;
	type RolesPerCallLimit = ConstU32<20>;
	type CallMetadata = CallMetadata;
	type ExtendedRuntimeCall = RuntimeCall;
}

impl GetCallMetadataIndecies for RuntimeCall {
	fn get_call_metadata_indicies(&self) -> ModuleCallIndex {
		match self {
			Self::System(call) => (System::index() as u64, call.get_call_index()),
			Self::Roles(call) => (Roles::index() as u64, call.get_call_index()),
			Self::Balances(call) => (Balances::index() as u64, call.get_call_index()),
		}
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> { balances: vec![(ALICE, EXISTENTIAL_DEPOSIT + 1)] }
		.assimilate_storage(&mut t)
		.unwrap();

	pallet_rbac::GenesisConfig::<Test> {
		roles: vec![
			(remarker_role(), false, false),
			(balancer_role(), true, true),
			(default_empty_role(), false, false),
		],
		calls: vec![
			(remarker_role(), remark_metadata()),
			(balancer_role(), force_set_balance_metadata()),
		],
		users: vec![(remarker_role(), ALICE), (balancer_role(), ALICE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext: sp_io::TestExternalities = t.into();
	ext.execute_with(|| {
		let obsolete_role_info =
			RoleInfoOf::<Test>::new_raw(0, [1; 16], false, RoleDispatchOrigin::Regular);
		assert_ne!(System::runtime_version().encode().twox_128(), [1; 16]);
		RolesMap::<Test>::insert(obsolete_role(), obsolete_role_info);
		CallRoles::<Test>::insert(
			deprecated_metadata(),
			call_set_with(vec![obsolete_role()]).unwrap(),
		);
		Roles::inc_role_consumers(&obsolete_role()).unwrap();

		AccountRoles::<Test>::mutate(ALICE, |role_set| {
			// account_set_with(vec![obsolete_role(), ]).unwrap());
			let role_set = role_set.get_or_insert(AccountRolesListOf::<Test>::default());
			role_set.try_insert(obsolete_role()).unwrap();
			Roles::inc_role_consumers(&obsolete_role()).unwrap();
		});

		System::set_block_number(1);
	});

	ext
}
