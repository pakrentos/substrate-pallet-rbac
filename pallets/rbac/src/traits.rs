use sp_runtime::transaction_validity::TransactionValidityError;

pub trait CallValidator<CallMetadata, AccountId> {
	/// Validates the transaction call based on the associated metadata and the account making the
	/// call.
	///
	/// # Parameters
	///
	/// - `call`: The metadata of the call being validated. The roles associated with this call
	///   metadata are fetched and considered during validation.
	/// - `who`: A reference to the account ID making the call. The roles associated with this
	///   account are fetched and considered during validation.
	fn validate_by_metadata(
		call: CallMetadata,
		who: &AccountId,
	) -> Result<(), TransactionValidityError>;
}

pub trait GetCallMetadataIndecies {
	fn get_call_metadata_indicies(&self) -> (u64, u8);
}
