use codec::{Decode, Encode};
use frame_support::{
	dispatch::{fmt::Debug, MaxEncodedLen},
	ensure,
	pallet_prelude::{DispatchError, DispatchResult},
	Hashable,
};
use frame_system::RawOrigin;
use scale_info::TypeInfo;
use sp_runtime::traits::Zero;
use sp_version::RuntimeVersion;

pub type ModuleCallIndex = (u64, u8);
pub type RuntimeVersionHash = [u8; 16];

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
pub struct CallMetadata {
	pub function_index: u8,
	pub pallet_index: u64,
}

impl From<ModuleCallIndex> for CallMetadata {
	fn from(value: ModuleCallIndex) -> Self {
		let (pallet_index, function_index) = value;
		Self { function_index, pallet_index }
	}
}

impl CallMetadata {
	pub fn into_inner(self) -> ModuleCallIndex {
		(self.pallet_index, self.function_index)
	}
}

#[derive(TypeInfo, MaxEncodedLen, Encode, Default, Decode, Debug, Clone, PartialEq, Eq)]
pub enum RoleDispatchOrigin<AccountId> {
	#[default]
	Regular,
	SignedAs {
		who: AccountId,
	},
	Root,
}

/// The `RoleInfo` struct holds information about a counter tracking how many consumers are using
/// this role.
#[derive(TypeInfo, MaxEncodedLen, Encode, Decode, PartialEq, Eq, Debug)]
pub struct RoleInfo<AccountId> {
	consumers_counter: u128,
	runtime_version: RuntimeVersionHash,
	dispatch_origin: RoleDispatchOrigin<AccountId>,
	pub allow_filter_bypassing: bool,
}

impl<AccountId: Clone> RoleInfo<AccountId> {
	pub fn new(
		runtime_version: RuntimeVersion,
		allow_filter_bypassing: bool,
		dispatch_origin: RoleDispatchOrigin<AccountId>,
	) -> Self {
		let hashed_runtime_version: RuntimeVersionHash = runtime_version.encode().twox_128();
		Self {
			runtime_version: hashed_runtime_version,
			allow_filter_bypassing,
			dispatch_origin,
			consumers_counter: 0u128,
		}
	}

	/// Increments the consumer counter by one. Returns an error if the operation would cause an
	/// overflow.
	pub fn inc_consumers(&mut self) -> DispatchResult {
		self.consumers_counter =
			self.consumers_counter.checked_add(1).ok_or(DispatchError::TooManyConsumers)?;
		Ok(())
	}

	/// Decrements the consumer counter by one. Returns an error if the operation would cause an
	/// underflow.
	pub fn dec_consumers(&mut self) -> DispatchResult {
		self.consumers_counter =
			self.consumers_counter.checked_sub(1).ok_or(DispatchError::ConsumerRemaining)?;
		Ok(())
	}

	/// Checks if the role is unused, i.e., if the consumers counter is zero. Returns an error if
	/// the role is still being used.
	pub fn check_if_unused(&self) -> DispatchResult {
		self.consumers_counter
			.is_zero()
			.then_some(())
			.ok_or(DispatchError::ConsumerRemaining)
	}

	/// Checks if the runtime version matches the one stored in the `RoleInfo`.
	///
	/// This method verifies that the given `runtime_version` matches the version stored in the
	/// role's information by hashing the given `runtime_version` and comparing it to the stored
	/// hash. This ensures that the role is compatible with the current runtime version.
	///
	/// # Parameters
	/// - `runtime_version`: The runtime version to be checked. It is hashed using the `twox_128`
	///   algorithm and then compared to the stored hash.
	pub fn check_version(&self, runtime_version: RuntimeVersion) -> DispatchResult {
		let hashed_runtime_version: RuntimeVersionHash = runtime_version.encode().twox_128();
		ensure!(
			hashed_runtime_version == self.runtime_version,
			DispatchError::Other("Role runtime version does not match current runtime version")
		);
		Ok(())
	}

	/// Infers the origin for call dispatch based on a role's dispatch origin configuration.
	///
	/// This function determines the appropriate dispatch origin based on the `dispatch_origin`
	/// attribute of the role. It translates the abstract representation of the origin into a
	/// concrete `RawOrigin` variant that can be used in dispatching calls.
	///
	/// # Parameters
	/// - `who`: The account ID of the user attempting to dispatch a call. This parameter is used to
	///   construct a signed origin when the role's dispatch origin is set to `Regular` or
	///   `SignedAs`.
	pub fn infer_origin(&self, who: AccountId) -> RawOrigin<AccountId> {
		match &self.dispatch_origin {
			RoleDispatchOrigin::Regular => RawOrigin::Signed(who),
			RoleDispatchOrigin::SignedAs { who } => RawOrigin::Signed(who.clone()),
			RoleDispatchOrigin::Root => RawOrigin::Root,
		}
	}

	#[cfg(test)]
	pub(crate) fn get_consumers_counter(&self) -> u128 {
		self.consumers_counter
	}

	#[cfg(test)]
	pub(crate) fn new_raw(
		consumers_counter: u128,
		runtime_version: RuntimeVersionHash,
		allow_filter_bypassing: bool,
		dispatch_origin: RoleDispatchOrigin<AccountId>,
	) -> Self {
		Self { consumers_counter, runtime_version, dispatch_origin, allow_filter_bypassing }
	}
}
