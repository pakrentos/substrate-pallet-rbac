use sp_runtime::transaction_validity::TransactionValidityError;

pub trait CallValidator<CallMetadata, AccountId> {
	fn validate_by_metadata(
		call: CallMetadata,
		who: &AccountId,
	) -> Result<(), TransactionValidityError>;
}

pub trait GetCallMetadataIndecies {
	fn get_call_metadata_indicies(&self) -> (u64, u8);
}
