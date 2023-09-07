#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::{FullCodec, MaxEncodedLen};
use frame_support::pallet_prelude::{Decode, DispatchResult, Encode};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, BoundedBTreeSet, BoundedVec, DispatchError};
use sp_std::default::Default;
// pub use weights::*;
use sp_weights::Weight;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
// pub mod weights;

/// The `RoleInfo` struct holds information about a counter tracking how many consumers are using this role.
#[derive(TypeInfo, MaxEncodedLen, Default, Encode, Decode)]
pub struct RoleInfo {
	consumers_counter: u128,
}

impl RoleInfo {
	/// Increments the consumer counter by one. Returns an error if the operation would cause an overflow.
	pub fn inc_consumers(&mut self) -> DispatchResult {
		self.consumers_counter =
			self.consumers_counter.checked_add(1).ok_or(DispatchError::TooManyConsumers)?;
		Ok(())
	}

	/// Decrements the consumer counter by one. Returns an error if the operation would cause an underflow.
	pub fn dec_consumers(&mut self) -> DispatchResult {
		self.consumers_counter =
			self.consumers_counter.checked_sub(1).ok_or(DispatchError::ConsumerRemaining)?;
		Ok(())
	}

	/// Checks if the role is unused, i.e., if the consumers counter is zero. Returns an error if the role is still being used.
	pub fn check_if_unused(&self) -> DispatchResult {
		self.consumers_counter
			.is_zero()
			.then_some(())
			.ok_or(DispatchError::ConsumerRemaining)
	}
}

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type RoleNameOf<T> = BoundedVec<u8, <T as Config>::StringLimit>;
type RoleListOf<T> = BoundedBTreeSet<RoleNameOf<T>, <T as Config>::RolesPerCallLimit>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{ensure, pallet_prelude::*, traits::Get, Blake2_128Concat, Parameter};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Defines who can manage roles
		type ManageOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Defines the limit for the length of role names.
		type StringLimit: Get<u32>;
		/// Defines the maximum number of roles that can be associated with a particular call.
		type RolesPerCallLimit: Get<u32>;
		/// Type representing the weight of this pallet
		// type WeightInfo: WeightInfo;
		/// Describes the metadata of a call, which is associated with roles to define permissions.
		type CallMetadata: FullCodec + MaxEncodedLen + TypeInfo + Parameter;
	}

	/// Holds the role information for each role name.
	#[pallet::storage]
	#[pallet::getter(fn roles)]
	pub type Roles<T: Config> =
		StorageMap<_, Blake2_128Concat, RoleNameOf<T>, RoleInfo, OptionQuery>;

	/// Holds account's roles
	#[pallet::storage]
	#[pallet::getter(fn account_roles)]
	pub type AccountRoles<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		Blake2_128Concat,
		RoleNameOf<T>,
		bool,
		ValueQuery,
	>;

	/// Holds call's associated roles
	#[pallet::storage]
	#[pallet::getter(fn call_roles)]
	pub type CallRoles<T: Config> =
		StorageMap<_, Blake2_128Concat, T::CallMetadata, RoleListOf<T>, OptionQuery>;

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
		/// The call cannot be added to the role because it is already part of the role's permissions.
		CallAlreadyAttachedToRole,
		/// The operation cannot be completed because adding this role would exceed the allowed 
		/// number of roles per call.
		TooManyRolesPerCall,
		/// The call cannot be removed from the role because it is not part of the role's permissions.
		CallNotAttachedToRole,
		/// The role cannot be assigned because it is already assigned to the account.
		RoleAlreadyAssigned,
		/// The operation cannot be completed because a role needed for it was not found.
		MissingRole,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new role with the specified name.
		/// Only callable by accounts with the appropriate management origin.
		///
		/// # Parameters
		/// - `role_name`: The name of the new role to create.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn create_role(
			origin: OriginFor<T>,
			role_name: RoleNameOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			ensure!(Self::roles(&role_name).is_none(), Error::<T>::RoleExists);

			Roles::<T>::insert(&role_name, RoleInfo::default());
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
		#[pallet::weight(Weight::zero())]
		pub fn add_call(
			origin: OriginFor<T>,
			role_name: RoleNameOf<T>,
			call: T::CallMetadata,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			ensure!(Self::roles(&role_name).is_some(), Error::<T>::RoleDoesNotExist);

			CallRoles::<T>::mutate(&call, |call_roles| {
				let call_roles = call_roles.get_or_insert(RoleListOf::<T>::default());
				ensure!(!call_roles.contains(&role_name), Error::<T>::CallAlreadyAttachedToRole);
				call_roles
					.try_insert(role_name.clone())
					.map_err(|_| Error::<T>::TooManyRolesPerCall)
			})?;
			Self::inc_role_consumers(&role_name)?;
			Self::deposit_event(Event::<T>::CallAddedToRole { role_name, call_metadata: call });

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
		#[pallet::weight(Weight::zero())]
		pub fn remove_call(
			origin: OriginFor<T>,
			role_name: RoleNameOf<T>,
			call: T::CallMetadata,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			ensure!(Self::roles(&role_name).is_some(), Error::<T>::RoleDoesNotExist);

			CallRoles::<T>::mutate(&call, |call_roles| {
				if let Some(call_roles) = call_roles.as_mut() {
					ensure!(call_roles.contains(&role_name), Error::<T>::CallNotAttachedToRole);
					call_roles.remove(&role_name);
					Ok(())
				} else {
					Err(Error::<T>::CallNotAttachedToRole)
				}
			})?;
			Self::dec_role_consumers(&role_name)?;
			Self::deposit_event(Event::<T>::CallRemovedFromRole { role_name, call_metadata: call });

			Ok(().into())
		}

		/// Assigns a role to an account, granting it the permissions defined for the role.
		/// Only callable by accounts with the appropriate management origin.
		///
		/// # Parameters
		/// - `who`: The account to assign the role to.
		/// - `role_name`: The name of the role to assign.
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::zero())]
		pub fn assign_role(
			origin: OriginFor<T>,
			who: AccountIdOf<T>,
			role_name: RoleNameOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			ensure!(Self::roles(&role_name).is_some(), Error::<T>::RoleDoesNotExist);

			ensure!(!Self::account_roles(&who, &role_name), Error::<T>::RoleAlreadyAssigned,);
			AccountRoles::<T>::insert(&who, &role_name, true);
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
		#[pallet::weight(Weight::zero())]
		pub fn unassign_role(
			origin: OriginFor<T>,
			who: AccountIdOf<T>,
			role_name: RoleNameOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ManageOrigin::ensure_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
			ensure!(Self::roles(&role_name).is_some(), Error::<T>::RoleDoesNotExist);

			ensure!(Self::account_roles(&who, &role_name), Error::<T>::MissingRole,);
			AccountRoles::<T>::remove(&who, &role_name);
			Self::dec_role_consumers(&role_name)?;
			Self::deposit_event(Event::<T>::AccountUnassignedFromRole { role_name, who });

			Ok(().into())
		}

		/// Removes a role definition entirely, unassigning it from all accounts and removing all calls
		/// associated with it. This is a sensitive operation and should be used with caution.
		/// Only callable by accounts with the appropriate management origin.
		///
		/// # Parameters
		/// - `role_name`: The name of the role to remove.
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::zero())]
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
}
