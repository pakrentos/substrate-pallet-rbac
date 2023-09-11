use crate::{
	mock::{AccountId, BalancesCall, RawOrigin, Roles, RuntimeCall, SystemCall, Test, ALICE},
	primitives::CallMetadata,
	traits::GetCallMetadataIndecies,
	Config, Error, Event, RoleNameOf,
};
use frame_system::pallet_prelude::OriginFor;
use sp_runtime::BoundedBTreeSet;

pub(crate) type RolesEvent = Event<Test>;
pub(crate) type RolesError = Error<Test>;
pub(crate) type SystemEvent = frame_system::Event<Test>;
pub(crate) type CallRolesSet =
	BoundedBTreeSet<RoleNameOf<Test>, <Test as Config>::RolesPerCallLimit>;

pub(crate) fn role_name(name: &[u8]) -> RoleNameOf<Test> {
	name.to_vec().try_into().expect("Expected to generate a role name")
}

pub(crate) fn remarker_role() -> RoleNameOf<Test> {
	role_name(b"Remarker")
}

pub(crate) fn balancer_role() -> RoleNameOf<Test> {
	role_name(b"Balancer")
}

pub(crate) fn default_empty_role() -> RoleNameOf<Test> {
	role_name(b"Empty")
}

pub(crate) fn obsolete_role() -> RoleNameOf<Test> {
	role_name(b"Obsolete")
}

pub(crate) fn remark_call() -> Box<RuntimeCall> {
	RuntimeCall::System(SystemCall::remark_with_event { remark: b"abc".to_vec() }).into()
}
pub(crate) fn remark_metadata() -> CallMetadata {
	remark_call().get_call_metadata_indicies().into()
}

pub(crate) fn force_set_balance_call() -> Box<RuntimeCall> {
	RuntimeCall::Balances(BalancesCall::force_set_balance { who: ALICE, new_free: 0 }).into()
}

pub(crate) fn force_set_balance_metadata() -> CallMetadata {
	force_set_balance_call().get_call_metadata_indicies().into()
}

pub(crate) fn deprecated_call() -> Box<RuntimeCall> {
	RuntimeCall::Balances(BalancesCall::set_balance_deprecated {
		who: ALICE,
		new_free: 0,
		old_reserved: 0,
	})
	.into()
}

pub(crate) fn deprecated_metadata() -> CallMetadata {
	deprecated_call().get_call_metadata_indicies().into()
}

pub(crate) fn call_set_with(names: Vec<RoleNameOf<Test>>) -> Option<CallRolesSet> {
	let mut new_set = CallRolesSet::new();
	names.into_iter().for_each(|name| {
		new_set.try_insert(name).unwrap();
	});
	Some(new_set)
}

pub(crate) fn account_set_contains(acc: &AccountId, name: &RoleNameOf<Test>) -> bool {
	Roles::account_roles(acc).unwrap_or_default().contains(name)
}

pub(crate) fn call_set_contains(call: &CallMetadata, name: &RoleNameOf<Test>) -> bool {
	Roles::call_roles(call).unwrap_or_default().contains(name)
}

pub(crate) fn assert_consumers_counter_eq(name: &RoleNameOf<Test>, counter: u128) {
	assert_eq!(Roles::roles(name).unwrap().get_consumers_counter(), counter);
}

pub(crate) fn root() -> OriginFor<Test> {
	RawOrigin::Root.into()
}

pub(crate) fn signed_as(who: AccountId) -> OriginFor<Test> {
	RawOrigin::Signed(who).into()
}
