use crate::RuntimeCall;
use codec::{Decode, Encode};
use frame_support::dispatch::fmt::Debug;
use pallet_rbac::{traits::GetCallMetadataIndecies, CallValidator, Config, Pallet};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, SignedExtension},
	transaction_validity::TransactionValidityError,
};
use sp_std::marker::PhantomData;

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckRole<T: Config>(PhantomData<T>);

impl<T: Config> Debug for CheckRole<T> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_tuple("CheckRole").finish()
	}
}

impl<T: Config> CheckRole<T> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<T: Config + Send + Sync> SignedExtension for CheckRole<T> {
	type AdditionalSigned = ();
	type Call = RuntimeCall;
	type AccountId = T::AccountId;
	type Pre = ();
	const IDENTIFIER: &'static str = "CheckRole";

	fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
		Ok(())
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		let call_metadata: T::CallMetadata = call.get_call_metadata_indicies().into();
		Pallet::<T>::validate_by_metadata(call_metadata, who)?;
		Ok(())
	}
}
