use crate::{mock::*, tests_utils::*, RoleInfo};
use frame_support::{assert_noop, assert_ok, Hashable};
use sp_runtime::DispatchError;

#[test]
fn create_role_should_work() {
	new_test_ext().execute_with(|| {
		let role_name = role_name(b"Test");
		assert_ok!(Roles::create_role(
			root(),
			role_name.clone(),
			false,
			crate::RoleDispatchOrigin::Regular
		));

		let role_info =
			RoleInfo::new(System::runtime_version(), false, crate::RoleDispatchOrigin::Regular);
		assert_eq!(Roles::roles(&role_name), Some(role_info));

		System::assert_last_event(RolesEvent::RoleCreated { role_name }.into());
	});
}

#[test]
fn existing_role_should_prevent_create_role() {
	new_test_ext().execute_with(|| {
		let role_name = role_name(b"Test");
		assert_ok!(Roles::create_role(
			root(),
			role_name.clone(),
			false,
			crate::RoleDispatchOrigin::Regular
		));

		assert_noop!(
			Roles::create_role(
				root(),
				role_name.clone(),
				false,
				crate::RoleDispatchOrigin::Regular
			),
			RolesError::RoleExists
		);
	});
}

#[test]
fn add_call_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(default_empty_role()).is_some());
		assert_consumers_counter_eq(&default_empty_role(), 0);
		assert!(!call_set_contains(&remark_metadata(), &default_empty_role()));

		assert_ok!(Roles::add_call(root(), default_empty_role(), remark_call()));
		System::assert_last_event(
			RolesEvent::CallAddedToRole {
				role_name: default_empty_role(),
				call_metadata: remark_metadata(),
			}
			.into(),
		);

		assert_consumers_counter_eq(&default_empty_role(), 1);
		assert!(call_set_contains(&remark_metadata(), &default_empty_role()));
	});
}

#[test]
fn non_existing_role_should_prevent_add_call() {
	new_test_ext().execute_with(|| {
		let role_name = role_name(b"NoRole");
		assert!(Roles::roles(&role_name).is_none());

		assert_noop!(
			Roles::add_call(root(), role_name, remark_call()),
			RolesError::RoleDoesNotExist
		);
	});
}

#[test]
fn already_added_call_should_prevent_add_call() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(remarker_role()).is_some());
		assert!(call_set_contains(&remark_metadata(), &remarker_role()));

		assert_noop!(
			Roles::add_call(root(), remarker_role(), remark_call()),
			RolesError::CallAlreadyAttachedToRole
		);
	});
}

#[test]
fn obsolete_role_should_prevent_add_call() {
	new_test_ext().execute_with(|| {
		let current_version = System::runtime_version();
		assert!(Roles::roles(obsolete_role()).is_some());
		assert!(Roles::roles(obsolete_role()).unwrap().check_version(current_version).is_err());

		assert!(Roles::add_call(root(), obsolete_role(), remark_call()).is_err());
	});
}

#[test]
fn remove_call_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(remarker_role()).is_some());
		assert!(call_set_contains(&remark_metadata(), &remarker_role()));
		assert_consumers_counter_eq(&remarker_role(), 2);

		assert_ok!(Roles::remove_call(root(), remarker_role(), remark_call()));
		System::assert_last_event(
			RolesEvent::CallRemovedFromRole {
				role_name: remarker_role(),
				call_metadata: remark_metadata(),
			}
			.into(),
		);
		assert_consumers_counter_eq(&remarker_role(), 1);
		assert!(!call_set_contains(&remark_metadata(), &remarker_role()));
	});
}

#[test]
fn non_existing_role_should_prevent_remove_call() {
	new_test_ext().execute_with(|| {
		let role_name = role_name(b"NoRole");
		assert!(Roles::roles(&role_name).is_none());

		assert_noop!(
			Roles::remove_call(root(), role_name, remark_call()),
			RolesError::RoleDoesNotExist
		);
	});
}

#[test]
fn non_attached_call_should_prevent_remove_call() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(remarker_role()).is_some());
		assert!(!call_set_contains(&force_set_balance_metadata(), &remarker_role()));

		assert_noop!(
			Roles::remove_call(root(), remarker_role(), force_set_balance_call()),
			RolesError::CallNotAttachedToRole
		);
	});
}

#[test]
fn remove_call_from_obsolete_role_should_work() {
	new_test_ext().execute_with(|| {
		let current_version = System::runtime_version();
		assert!(Roles::roles(obsolete_role()).is_some());
		assert!(Roles::roles(obsolete_role()).unwrap().check_version(current_version).is_err());
		assert_consumers_counter_eq(&obsolete_role(), 2);

		assert_ok!(Roles::remove_call(root(), obsolete_role(), deprecated_call()));
		System::assert_last_event(
			RolesEvent::CallRemovedFromRole {
				role_name: obsolete_role(),
				call_metadata: deprecated_metadata(),
			}
			.into(),
		);
		assert_consumers_counter_eq(&obsolete_role(), 1);
		assert!(!call_set_contains(&deprecated_metadata(), &obsolete_role()));
	});
}

#[test]
fn assign_role_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(default_empty_role()).is_some());
		assert_consumers_counter_eq(&default_empty_role(), 0);
		assert!(!account_set_contains(&ALICE, &default_empty_role()));

		assert_ok!(Roles::assign_role(root(), ALICE, default_empty_role()));
		System::assert_last_event(
			RolesEvent::AccountAssignedToRole { role_name: default_empty_role(), who: ALICE }
				.into(),
		);

		assert_consumers_counter_eq(&default_empty_role(), 1);
		assert!(account_set_contains(&ALICE, &default_empty_role()));
	});
}

#[test]
fn non_existing_role_should_prevent_assign_role() {
	new_test_ext().execute_with(|| {
		let role_name = role_name(b"NoRole");
		assert!(Roles::roles(&role_name).is_none());

		assert_noop!(Roles::assign_role(root(), ALICE, role_name), RolesError::RoleDoesNotExist);
	});
}

#[test]
fn already_assigned_user_should_prevent_assign_role() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(remarker_role()).is_some());
		assert!(account_set_contains(&ALICE, &remarker_role()));

		assert_noop!(
			Roles::assign_role(root(), ALICE, remarker_role()),
			RolesError::RoleAlreadyAssigned
		);
	});
}

#[test]
fn obsolete_role_should_prevent_assign_role() {
	new_test_ext().execute_with(|| {
		let current_version = System::runtime_version();
		assert!(Roles::roles(obsolete_role()).is_some());
		assert!(Roles::roles(obsolete_role()).unwrap().check_version(current_version).is_err());

		assert!(Roles::assign_role(root(), ALICE, obsolete_role()).is_err());
	});
}

#[test]
fn unassign_role_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(remarker_role()).is_some());
		assert_consumers_counter_eq(&remarker_role(), 2);
		assert!(account_set_contains(&ALICE, &remarker_role()));

		assert_ok!(Roles::unassign_role(root(), ALICE, remarker_role()));
		System::assert_last_event(
			RolesEvent::AccountUnassignedFromRole { role_name: remarker_role(), who: ALICE }.into(),
		);

		assert_consumers_counter_eq(&remarker_role(), 1);
		assert!(!account_set_contains(&ALICE, &remarker_role()));
	});
}

#[test]
fn non_existing_role_should_prevent_unassign_role() {
	new_test_ext().execute_with(|| {
		let role_name = role_name(b"NoRole");
		assert!(Roles::roles(&role_name).is_none());

		assert_noop!(Roles::unassign_role(root(), ALICE, role_name), RolesError::RoleDoesNotExist);
	});
}

#[test]
fn non_assigned_user_should_prevent_unassign_role() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(default_empty_role()).is_some());
		assert!(!account_set_contains(&ALICE, &default_empty_role()));

		assert_noop!(
			Roles::unassign_role(root(), ALICE, default_empty_role()),
			RolesError::MissingRole
		);
	});
}

#[test]
fn unassign_obsolete_role_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(obsolete_role()).is_some());
		assert_consumers_counter_eq(&obsolete_role(), 2);
		assert!(account_set_contains(&ALICE, &obsolete_role()));

		assert_ok!(Roles::unassign_role(root(), ALICE, obsolete_role()));
		System::assert_last_event(
			RolesEvent::AccountUnassignedFromRole { role_name: obsolete_role(), who: ALICE }.into(),
		);

		assert_consumers_counter_eq(&obsolete_role(), 1);
		assert!(!account_set_contains(&ALICE, &obsolete_role()));
	});
}

#[test]
fn remove_role_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(default_empty_role()).is_some());
		assert_consumers_counter_eq(&default_empty_role(), 0);

		assert_ok!(Roles::remove_role(root(), default_empty_role()));
		System::assert_last_event(
			RolesEvent::RoleRemoved { role_name: default_empty_role() }.into(),
		);

		assert!(Roles::roles(default_empty_role()).is_none());
	});
}

#[test]
fn non_zero_consumers_counter_should_prevent_remove_role() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(remarker_role()).is_some());
		assert_consumers_counter_eq(&remarker_role(), 2);

		assert_noop!(Roles::remove_role(root(), remarker_role()), DispatchError::ConsumerRemaining,);
	});
}

#[test]
fn non_existing_role_should_prevent_remove_role() {
	new_test_ext().execute_with(|| {
		let role_name = role_name(b"NoRole");
		assert!(Roles::roles(&role_name).is_none());

		assert_noop!(Roles::remove_role(root(), role_name), RolesError::RoleDoesNotExist);
	});
}

#[test]
fn remove_obsolete_role_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(obsolete_role()).is_some());
		assert_consumers_counter_eq(&&obsolete_role(), 2);

		Roles::remove_call(root(), obsolete_role(), deprecated_call())
			.expect("Expected to remove a call from an obsolete role");
		Roles::unassign_role(root(), ALICE, obsolete_role())
			.expect("Expected to unassign a user from an obsolete role");
		assert_consumers_counter_eq(&&obsolete_role(), 0);

		assert_ok!(Roles::remove_role(root(), obsolete_role()));
		System::assert_last_event(RolesEvent::RoleRemoved { role_name: obsolete_role() }.into());

		assert!(Roles::roles(obsolete_role()).is_none());
	});
}

#[test]
fn dispatch_call_with_role_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Roles::roles(remarker_role()).is_some());
		assert!(account_set_contains(&ALICE, &remarker_role()));
		assert!(call_set_contains(&remark_metadata(), &remarker_role()));

		assert_ok!(Roles::dispatch_call_with_role(
			signed_as(ALICE),
			remark_call(),
			remarker_role()
		));
		System::assert_has_event(
			SystemEvent::Remarked { sender: ALICE, hash: sp_core::H256(b"abc".blake2_256()) }
				.into(),
		);
	});
}
