use crate::ModuleCallIndex;
use codec::{Decode, Encode};
use frame_support::dispatch::MaxEncodedLen;
use scale_info::TypeInfo;

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
