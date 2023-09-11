#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::{
	extension::CheckRole,
	primitives::{RoleDispatchOrigin, RoleInfo},
	traits::CallValidator,
};
pub use pallet::*;

use codec::{FullCodec, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::DispatchResult};
use frame_system::Pallet as System;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::Zero,
	transaction_validity::{InvalidTransaction, TransactionValidityError},
	BoundedBTreeSet, BoundedVec, DispatchError,
};
use sp_std::default::Default;
pub use weights::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_utils;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub mod extension;
pub mod primitives;
pub mod traits;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type RoleNameOf<T> = BoundedVec<u8, <T as Config>::RoleNameLengthLimit>;
type RoleInfoOf<T> = RoleInfo<<T as frame_system::Config>::AccountId>;
type CallRolesListOf<T> = BoundedBTreeSet<RoleNameOf<T>, <T as Config>::RolesPerCallLimit>;
type AccountRolesListOf<T> = BoundedBTreeSet<RoleNameOf<T>, <T as Config>::RolesPerAccountLimit>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::{primitives::ModuleCallIndex, traits::GetCallMetadataIndecies};
	use frame_support::{
		dispatch::{
			fmt::Debug, Dispatchable, GetDispatchInfo, PostDispatchInfo, UnfilteredDispatchable,
			Vec,
		},
		pallet_prelude::*,
		traits::{BuildGenesisConfig, Get},
		Blake2_128Concat, Parameter,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::boxed::Box;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Defines who can manage roles
		type ManageOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Defines the limit for the length of role names.
		type RoleNameLengthLimit: Get<u32>;
		/// Defines the maximum number of roles that can be associated with a particular call.
		type RolesPerCallLimit: Get<u32>;
		/// Defines the maximum number of roles that can be associated with a particular account.
		type RolesPerAccountLimit: Get<u32>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		/// Describes the metadata of a call, which is associated with roles to define permissions.
		type CallMetadata: FullCodec + MaxEncodedLen + TypeInfo + Parameter + From<(u64, u8)>;
		/// RuntimeCall time extended with GetCallMetadataIndecies trait
		type ExtendedRuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
			+ Debug
			+ From<Call<Self>>
			+ GetCallMetadataIndecies
			+ UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ GetDispatchInfo;
	}

	/// Holds the role information for each role name.
	#[pallet::storage]
	#[pallet::getter(fn roles)]
	pub type Roles<T: Config> =
		StorageMap<_, Blake2_128Concat, RoleNameOf<T>, RoleInfoOf<T>, OptionQuery>;

	/// Holds account's roles
	#[pallet::storage]
	#[pallet::getter(fn account_roles)]
	pub type AccountRoles<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, AccountRolesListOf<T>, OptionQuery>;

	/// Holds call's associated roles
	#[pallet::storage]
	#[pallet::getter(fn call_roles)]
	pub type CallRoles<T: Config> =
		StorageMap<_, Blake2_128Concat, T::CallMetadata, CallRolesListOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new role was created.
		RoleCreated { role_name: RoleNameOf<T> },
		/// A role was removed.
		RoleRemoved { role_name: RoleNameOf<T> },
		/// An account was assigned a role.
		AccountAssignedToRole { role_name: RoleNameOf<T>, who: AccountIdOf<T> },
		/// A role was unassigned from an account.
		AccountUnassignedFromRole { role_name: RoleNameOf<T>, who: AccountIdOf<T> },
		/// A call was added to a role's permission.
		CallAddedToRole { role_name: RoleNameOf<T>, call_metadata: T::CallMetadata },
		/// A call was removed from a role's permissions.
		CallRemovedFromRole { role_name: RoleNameOf<T>, call_metadata: T::CallMetadata },
		/// A call was dispatched using a role with a `role_name`
		CallDispatchedWithRole {
			role_name: RoleNameOf<T>,
			who: AccountIdOf<T>,
			call_metadata: T::CallMetadata,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The operation could not be completed because the origin of the call is not allowed to
		/// perform it.
		BadOrigin,
		/// The role cannot be created because a role with the same name already exists.
		RoleExists,
		/// The role cannot be found, it might have been removed or it does not exist.
		RoleDoesNotExist,
		/// The call cannot be added to the role because it is already part of the role's
		/// permissions.
		CallAlreadyAttachedToRole,
		/// The operation cannot be completed because adding this role would exceed the allowed
		/// number of roles per call.
		TooManyRolesPerCall,
		/// The operation cannot be completed because adding this role would exceed the allowed
		/// number of roles per account.
		TooManyRolesPerAccount,
		/// The call cannot be removed from the role because it is not part of the role's
		/// permissions.
		CallNotAttachedToRole,
		/// The role cannot be assigned because it is already assigned to the account.
		RoleAlreadyAssigned,
		/// The operation cannot be completed because a role needed for it was not found.
		MissingRole,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// [role_name, allow_filter_bypassing, is_root]
		pub roles: Vec<(RoleNameOf<T>, bool, bool)>,
		/// [role_name, call_metadata]
		pub calls: Vec<(RoleNameOf<T>, ModuleCallIndex)>,
		/// [role_name, assgined_account]
		pub users: Vec<(RoleNameOf<T>, AccountIdOf<T>)>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { roles: Default::default(), calls: Default::default(), users: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			self.roles.iter().cloned().for_each(|(name, allow_filter_bypass, is_root)| {
				let role_origin = is_root
					.then(|| RoleDispatchOrigin::Root)
					.unwrap_or(RoleDispatchOrigin::Regular);
				Roles::<T>::insert(
					&name,
					RoleInfoOf::<T>::new(
						System::<T>::runtime_version(),
						allow_filter_bypass,
						role_origin,
					),
				);
			});
			self.calls.iter().cloned().for_each(|(name, call_metadata)| {
				let call_metadata: T::CallMetadata = call_metadata.into();
				CallRoles::<T>::mutate(call_metadata, |call_roles| {
					let call_roles = call_roles.get_or_insert(CallRolesListOf::<T>::default());
					Pallet::<T>::inc_role_consumers(&name)
						.expect("Expected to increase consumers counter during genesis build");
					call_roles
						.try_insert(name)
						.expect("Epected to insert a role name to call's roles set");
				});
			});
			self.users.iter().cloned().for_each(|(name, user)| {
				AccountRoles::<T>::mutate(user, |account_roles| {
					let account_roles =
						account_roles.get_or_insert(AccountRolesListOf::<T>::default());
					Pallet::<T>::inc_role_consumers(&name)
						.expect("Expected to increase consumers counter during genesis build");
					account_roles
						.try_insert(name)
						.expect("Epected to insert a role name to account's roles set");
				});
			});
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new role with the specified name.
		/// Only callable by accounts with the appropriate management origin.
		///
		/// # Parameters
		/// - `role_name`: The name of the new role to create.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_role())]
		pub fn create_role(
			origin: OriginFor<T>,
			role_name: RoleNameOf<T>,
			allow_filter_bypassing: bool,
			allow_dispatch_as: RoleDispatchOrigin<<T as frame_system::Config>::AccountId>,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			ensure!(Self::roles(&role_name).is_none(), Error::<T>::RoleExists);

			let current_runtime_version = System::<T>::runtime_version();
			Roles::<T>::insert(
				&role_name,
				RoleInfoOf::<T>::new(
					current_runtime_version,
					allow_filter_bypassing,
					allow_dispatch_as,
				),
			);
			Self::deposit_event(Event::<T>::RoleCreated { role_name });

			Ok(().into())
		}

		/// Adds a call to a role's allowed calls list.
		/// This means that accounts with the role can execute the call.
		/// Only callable by accounts with the appropriate management origin.
		///
		/// # Parameters
		/// - `role_name`: The name of the role to modify.
		/// - `call`: The metadata of the call to add to the role's allowed list.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::add_call())]
		pub fn add_call(
			origin: OriginFor<T>,
			role_name: RoleNameOf<T>,
			call: Box<T::ExtendedRuntimeCall>,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			Self::check_role_existance_and_version(&role_name)?;

			let call_metadata: T::CallMetadata = call.get_call_metadata_indicies().into();
			CallRoles::<T>::mutate(&call_metadata, |call_roles| {
				let call_roles = call_roles.get_or_insert(CallRolesListOf::<T>::default());
				ensure!(!call_roles.contains(&role_name), Error::<T>::CallAlreadyAttachedToRole);
				call_roles
					.try_insert(role_name.clone())
					.map_err(|_| Error::<T>::TooManyRolesPerCall)
			})?;
			Self::inc_role_consumers(&role_name)?;
			Self::deposit_event(Event::<T>::CallAddedToRole { role_name, call_metadata });

			Ok(().into())
		}

		/// Removes a call from a role's allowed calls list.
		/// This means that accounts with the role will no longer be able to execute the call.
		/// Only callable by accounts with the appropriate management origin.
		///
		/// # Parameters
		/// - `role_name`: The name of the role to modify.
		/// - `call`: The metadata of the call to remove from the role's allowed list.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::remove_call())]
		pub fn remove_call(
			origin: OriginFor<T>,
			role_name: RoleNameOf<T>,
			call: Box<T::ExtendedRuntimeCall>,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			ensure!(Self::roles(&role_name).is_some(), Error::<T>::RoleDoesNotExist);

			let call_metadata: T::CallMetadata = call.get_call_metadata_indicies().into();
			CallRoles::<T>::mutate(&call_metadata, |call_roles| {
				if let Some(call_roles) = call_roles.as_mut() {
					ensure!(call_roles.contains(&role_name), Error::<T>::CallNotAttachedToRole);
					call_roles.remove(&role_name);
					Ok(())
				} else {
					Err(Error::<T>::CallNotAttachedToRole)
				}
			})?;
			Self::dec_role_consumers(&role_name)?;
			Self::deposit_event(Event::<T>::CallRemovedFromRole { role_name, call_metadata });

			Ok(().into())
		}

		/// Assigns a role to an account, granting it the permissions defined for the role.
		/// Only callable by accounts with the appropriate management origin.
		///
		/// # Parameters
		/// - `who`: The account to assign the role to.
		/// - `role_name`: The name of the role to assign.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::assign_role())]
		pub fn assign_role(
			origin: OriginFor<T>,
			who: AccountIdOf<T>,
			role_name: RoleNameOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			Self::check_role_existance_and_version(&role_name)?;

			AccountRoles::<T>::mutate(&who, |account_roles| {
				let account_roles = account_roles.get_or_insert(AccountRolesListOf::<T>::default());
				ensure!(!account_roles.contains(&role_name), Error::<T>::RoleAlreadyAssigned);
				account_roles
					.try_insert(role_name.clone())
					.map_err(|_| Error::<T>::TooManyRolesPerAccount)
			})?;
			Self::inc_role_consumers(&role_name)?;
			Self::deposit_event(Event::<T>::AccountAssignedToRole { role_name, who });

			Ok(().into())
		}

		/// Unassigns a role from an account, revoking its permissions defined for the role.
		/// Only callable by accounts with the appropriate management origin.
		///
		/// # Parameters
		/// - `who`: The account to unassign the role from.
		/// - `role_name`: The name of the role to unassign.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::unassign_role())]
		pub fn unassign_role(
			origin: OriginFor<T>,
			who: AccountIdOf<T>,
			role_name: RoleNameOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			ensure!(Self::roles(&role_name).is_some(), Error::<T>::RoleDoesNotExist);

			AccountRoles::<T>::mutate(&who, |account_roles| {
				if let Some(account_roles) = account_roles.as_mut() {
					ensure!(account_roles.contains(&role_name), Error::<T>::MissingRole);
					account_roles.remove(&role_name);
					Ok(())
				} else {
					Err(Error::<T>::MissingRole)
				}
			})?;
			Self::dec_role_consumers(&role_name)?;
			Self::deposit_event(Event::<T>::AccountUnassignedFromRole { role_name, who });

			Ok(().into())
		}

		/// Removes a role definition entirely, unassigning it from all accounts and removing all
		/// calls associated with it. This is a sensitive operation and should be used with caution.
		/// Only callable by accounts with the appropriate management origin.
		///
		/// # Parameters
		/// - `role_name`: The name of the role to remove.
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove_role())]
		pub fn remove_role(
			origin: OriginFor<T>,
			role_name: RoleNameOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;

			Self::roles(&role_name).ok_or(Error::<T>::RoleDoesNotExist)?.check_if_unused()?;
			Roles::<T>::remove(&role_name);
			Self::deposit_event(Event::<T>::RoleRemoved { role_name });

			Ok(().into())
		}

		/// Dispatches a call with a specified role.
		///
		/// This extrinsic allows the caller to execute a runtime call with a given role.
		/// The role is used to infer the appropriate origin for the call dispatch and to
		/// handle other role-specific logic.
		///
		/// # Parameters
		/// - `origin`: The origin of the call, expected to be a signed origin.
		/// - `call`: The runtime call to be dispatched, wrapped in a `Box` to manage the call's
		///   size at runtime.
		/// - `with_role`: The role with which the call should be dispatched.
		#[pallet::call_index(6)]
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(
				T::WeightInfo::dispatch_call_with_role().saturating_add(dispatch_info.weight),
				dispatch_info.class
			)
		})]
		pub fn dispatch_call_with_role(
			origin: OriginFor<T>,
			call: Box<T::ExtendedRuntimeCall>,
			with_role: RoleNameOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			let role_info = Self::check_role_existance_and_version(&with_role)?;
			ensure!(
				Self::account_roles(&who).unwrap_or_default().contains(&with_role),
				Error::<T>::MissingRole,
			);
			let call_metadata: T::CallMetadata = call.get_call_metadata_indicies().into();
			ensure!(
				Self::call_roles(&call_metadata).unwrap_or_default().contains(&with_role),
				Error::<T>::CallNotAttachedToRole,
			);
			let origin_for_dispatch = role_info.infer_origin(who.clone());
			let dispatch_result = if role_info.allow_filter_bypassing {
				call.dispatch_bypass_filter(origin_for_dispatch.into())
			} else {
				call.dispatch(origin_for_dispatch.into())
			};
			Self::deposit_event(Event::<T>::CallDispatchedWithRole {
				role_name: with_role,
				who,
				call_metadata,
			});
			dispatch_result
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Internal utility function to increase the consumer counter for a role.
	///
	/// # Parameters
	/// - `role_name`: The name of the role to increment the consumer counter for.
	pub fn inc_role_consumers(role_name: &RoleNameOf<T>) -> DispatchResult {
		Roles::<T>::mutate(role_name, |info| {
			if let Some(info) = info.as_mut() {
				info.inc_consumers()
			} else {
				Err(Error::<T>::RoleDoesNotExist.into())
			}
		})
	}

	/// Internal utility function to decrease the consumer counter for a role.
	///
	/// # Parameters
	/// - `role_name`: The name of the role to decrement the consumer counter for.
	pub fn dec_role_consumers(role_name: &RoleNameOf<T>) -> DispatchResult {
		Roles::<T>::mutate(role_name, |info| {
			if let Some(info) = info.as_mut() {
				info.dec_consumers()
			} else {
				Err(Error::<T>::RoleDoesNotExist.into())
			}
		})
	}

	/// Checks the existence and version compatibility of a specified role.
	///
	/// Given a role name, this function performs two essential checks:
	///
	/// 1. It verifies the existence of the role in the storage. If the role does not exist, it
	///    returns a `RoleDoesNotExist` error.
	/// 2. It verifies the compatibility of the role's version with the current runtime version. If
	///    the versions are incompatible, it returns a custom error.
	///
	/// # Parameters
	/// - `role_name`: A reference to the name of the role to check.
	pub fn check_role_existance_and_version(
		role_name: &RoleNameOf<T>,
	) -> Result<RoleInfoOf<T>, DispatchError> {
		let role_info = Self::roles(role_name).ok_or(Error::<T>::RoleDoesNotExist)?;
		let current_version = System::<T>::runtime_version();
		role_info.check_version(current_version)?;
		Ok(role_info)
	}
}

impl<T: Config> CallValidator<T::CallMetadata, AccountIdOf<T>> for Pallet<T> {
	fn validate_by_metadata(
		call: T::CallMetadata,
		who: &AccountIdOf<T>,
	) -> Result<(), TransactionValidityError> {
		let call_roles = match Self::call_roles(&call) {
			Some(call_roles) if !call_roles.len().is_zero() => call_roles,
			_ => return Ok(()),
		};
		let account_roles = match Self::account_roles(who) {
			Some(account_roles) if !account_roles.len().is_zero() => account_roles,
			_ => return Err(TransactionValidityError::Invalid(InvalidTransaction::Call)),
		};
		let role_name = call_roles
			.intersection(&account_roles)
			.next()
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::Call))?;
		Self::check_role_existance_and_version(role_name)
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;
		Ok(())
	}
}
