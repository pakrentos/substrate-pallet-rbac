//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::{
	primitives::RoleDispatchOrigin, traits::GetCallMetadataIndecies, Call as RolesCall, Config,
	Event, Pallet, Roles,
};
// use crate::tests_utils::
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

fn role_name_of<T: Config>(name: &[u8]) -> RoleNameOf<T> {
	name.to_vec().try_into().expect("Expected to generate a role name")
}

fn sample_call<T: Config>() -> Box<T::ExtendedRuntimeCall> {
	let call: T::ExtendedRuntimeCall = Call::create_role {
		role_name: vec![0u8].try_into().unwrap(),
		allow_filter_bypassing: true,
		allow_dispatch_as: RoleDispatchOrigin::Root,
	}
	.into();
	call.into()
}

fn sample_other_call<T: Config>() -> Box<T::ExtendedRuntimeCall> {
	let call: T::ExtendedRuntimeCall = Call::create_role {
		role_name: vec![1u8].try_into().unwrap(),
		allow_filter_bypassing: false,
		allow_dispatch_as: RoleDispatchOrigin::Root,
	}
	.into();
	call.into()
}

fn sample_call_metadata<T: Config>() -> T::CallMetadata {
	let call: T::ExtendedRuntimeCall = Call::create_role {
		role_name: vec![0u8].try_into().unwrap(),
		allow_filter_bypassing: false,
		allow_dispatch_as: RoleDispatchOrigin::Root,
	}
	.into();
	call.get_call_metadata_indicies().into()
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_role() {
		let role_name = role_name_of::<T>(b"NoRole");
		#[extrinsic_call]
		_(RawOrigin::Root, role_name.clone(), false, RoleDispatchOrigin::Regular);
		assert_last_event::<T>(Event::<T>::RoleCreated { role_name: role_name.clone() }.into());
	}

	#[benchmark]
	fn remove_role() {
		let role_name = role_name_of::<T>(b"NoRole");
		Pallet::<T>::create_role(
			RawOrigin::Root.into(),
			role_name.clone(),
			false,
			RoleDispatchOrigin::Regular,
		)
		.expect("Expected to create a role");

		#[extrinsic_call]
		_(RawOrigin::Root, role_name.clone());
		assert_last_event::<T>(Event::<T>::RoleRemoved { role_name: role_name.clone() }.into());
	}

	#[benchmark]
	fn add_call() {
		let role_name = role_name_of::<T>(b"NoRole");
		Pallet::<T>::create_role(
			RawOrigin::Root.into(),
			role_name.clone(),
			false,
			RoleDispatchOrigin::Regular,
		)
		.expect("Expected to create a role");

		#[extrinsic_call]
		_(RawOrigin::Root, role_name.clone(), sample_call::<T>());
		assert_last_event::<T>(
			Event::<T>::CallAddedToRole {
				role_name: role_name.clone(),
				call_metadata: sample_call_metadata::<T>(),
			}
			.into(),
		);
	}

	#[benchmark]
	fn remove_call() {
		let role_name = role_name_of::<T>(b"NoRole");
		Pallet::<T>::create_role(
			RawOrigin::Root.into(),
			role_name.clone(),
			false,
			RoleDispatchOrigin::Regular,
		)
		.expect("Expected to create a role");
		Pallet::<T>::add_call(RawOrigin::Root.into(), role_name.clone(), sample_call::<T>())
			.expect("Expected to add a call to a role");

		#[extrinsic_call]
		_(RawOrigin::Root, role_name.clone(), sample_call::<T>());
		assert_last_event::<T>(
			Event::<T>::CallRemovedFromRole {
				role_name: role_name.clone(),
				call_metadata: sample_call_metadata::<T>(),
			}
			.into(),
		);
	}

	#[benchmark]
	fn assign_role() {
		let role_name = role_name_of::<T>(b"NoRole");
		Pallet::<T>::create_role(
			RawOrigin::Root.into(),
			role_name.clone(),
			false,
			RoleDispatchOrigin::Regular,
		)
		.expect("Expected to create a role");

		#[extrinsic_call]
		_(RawOrigin::Root, whitelisted_caller(), role_name.clone());
		assert_last_event::<T>(
			Event::<T>::AccountAssignedToRole {
				role_name: role_name.clone(),
				who: whitelisted_caller(),
			}
			.into(),
		);
	}

	#[benchmark]
	fn unassign_role() {
		let role_name = role_name_of::<T>(b"NoRole");
		Pallet::<T>::create_role(
			RawOrigin::Root.into(),
			role_name.clone(),
			false,
			RoleDispatchOrigin::Regular,
		)
		.expect("Expected to create a role");
		Pallet::<T>::assign_role(RawOrigin::Root.into(), whitelisted_caller(), role_name.clone())
			.expect("Expected to assign a role");

		#[extrinsic_call]
		_(RawOrigin::Root, whitelisted_caller(), role_name.clone());
		assert_last_event::<T>(
			Event::<T>::AccountUnassignedFromRole {
				role_name: role_name.clone(),
				who: whitelisted_caller(),
			}
			.into(),
		);
	}

	#[benchmark]
	fn dispatch_call_with_role() {
		let role_name = role_name_of::<T>(b"NoRole");
		Pallet::<T>::create_role(
			RawOrigin::Root.into(),
			role_name.clone(),
			false,
			RoleDispatchOrigin::Root,
		)
		.expect("Expected to create a role");
		Pallet::<T>::assign_role(RawOrigin::Root.into(), whitelisted_caller(), role_name.clone())
			.expect("Expected to assign a role");
		Pallet::<T>::add_call(RawOrigin::Root.into(), role_name.clone(), sample_call::<T>())
			.expect("Expected to add a call to a role");

		#[extrinsic_call]
		_(RawOrigin::Signed(whitelisted_caller()), sample_other_call::<T>(), role_name.clone());
		assert_last_event::<T>(
			Event::<T>::CallDispatchedWithRole {
				role_name: role_name.clone(),
				who: whitelisted_caller(),
				call_metadata: sample_call_metadata::<T>(),
			}
			.into(),
		);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
