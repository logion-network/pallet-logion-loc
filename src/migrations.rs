use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::dispatch::Vec;
use frame_support::weights::Weight;
use frame_support::traits::OnRuntimeUpgrade;

use crate::{Config, LegalOfficerCaseOf, pallet, PalletStorageVersion, pallet::StorageVersion};

pub mod v9 {
	use super::*;
	use crate::{CollectionItemFile, CollectionItemsMap, CollectionItemOf, CollectionItemToken};

	#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
	struct CollectionItemV8<Hash> {
		description: Vec<u8>,
		files: Vec<CollectionItemFile<Hash>>,
		token: Option<CollectionItemToken>,
		restricted_delivery: bool,
	}

	type CollectionItemV8Of<T> = CollectionItemV8<<T as pallet::Config>::Hash>;

	pub struct AddTermsAndConditionsToCollectionItem<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for AddTermsAndConditionsToCollectionItem<T> {

		fn on_runtime_upgrade() -> Weight {
			super::do_storage_upgrade::<T, _>(
				StorageVersion::V8AddSeal,
				StorageVersion::V9TermsAndConditions,
				"AddTermsAndConditionsToCollectionItem",
				|| {
					CollectionItemsMap::<T>::translate(|_loc_id: T::LocId, _item_id: T::CollectionItemId, item: CollectionItemV8Of<T>| {
						let new_item = CollectionItemOf::<T> {
							description: item.description.clone(),
							files: item.files.clone(),
							token: item.token.clone(),
							restricted_delivery: item.restricted_delivery.clone(),
							terms_and_conditions: Vec::new(),
						};
						Some(new_item)
					});
				}
			)
		}
	}
}

pub mod v8 {
	use super::*;
	use crate::*;

	#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
	pub struct LegalOfficerCaseV7<AccountId, Hash, LocId, BlockNumber> {
		owner: AccountId,
		requester: Requester<AccountId, LocId>,
		metadata: Vec<MetadataItem<AccountId>>,
		files: Vec<File<Hash, AccountId>>,
		closed: bool,
		loc_type: LocType,
		links: Vec<LocLink<LocId>>,
		void_info: Option<LocVoidInfo<LocId>>,
		replacer_of: Option<LocId>,
		collection_last_block_submission: Option<BlockNumber>,
		collection_max_size: Option<CollectionSize>,
		collection_can_upload: bool,
	}

	type LegalOfficerCaseOfV7<T> = LegalOfficerCaseV7<<T as frame_system::Config>::AccountId, <T as pallet::Config>::Hash, <T as pallet::Config>::LocId, <T as frame_system::Config>::BlockNumber>;

	pub struct AddSealToLoc<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for AddSealToLoc<T> {

		fn on_runtime_upgrade() -> Weight {
			super::do_storage_upgrade::<T, _>(
				StorageVersion::V7ItemToken, 
				StorageVersion::V8AddSeal, 
				"AddSealToLoc",
				|| {
					LocMap::<T>::translate_values(|loc: LegalOfficerCaseOfV7<T>| {
						Some(LegalOfficerCaseOf::<T> {
							owner: loc.owner,
							requester: loc.requester,
							metadata: loc.metadata,
							files: loc.files,
							closed: loc.closed,
							loc_type: loc.loc_type,
							links: loc.links,
							void_info: loc.void_info,
							replacer_of: loc.replacer_of,
							collection_last_block_submission: loc.collection_last_block_submission,
							collection_max_size: loc.collection_max_size,
							collection_can_upload: loc.collection_can_upload,
							seal: Option::None,
						})
					})
				}
			)
		}
	}
}

fn do_storage_upgrade<T: Config, F>(expected_version: StorageVersion, target_version: StorageVersion, migration_name: &str, migration: F) -> Weight
where F: FnOnce() -> () {
	let storage_version = PalletStorageVersion::<T>::get();
	if storage_version == expected_version {
		migration();

		PalletStorageVersion::<T>::set(target_version);
		log::info!("✅ {:?} migration successfully executed", migration_name);
		T::BlockWeights::get().max_block
	} else {
		if storage_version != target_version {
			log::warn!("❗ {:?} cannot run migration with storage version {:?} (expected {:?})", migration_name, storage_version, expected_version);
		} else {
			log::info!("❎ {:?} execution skipped, already at target version {:?}", migration_name, target_version);
		}
		T::DbWeight::get().reads(1)
	}
}
