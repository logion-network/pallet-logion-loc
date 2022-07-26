use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::dispatch::Vec;
use frame_support::weights::Weight;
use frame_support::traits::OnRuntimeUpgrade;

use crate::{Config, File, LegalOfficerCaseOf, LocLink, LocMap, LocType, MetadataItem, pallet, PalletStorageVersion, StorageVersion};

pub mod v7 {
	use super::*;
	use crate::{CollectionItemFile, CollectionItemsMap, CollectionItemOf};

	#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
	struct CollectionItemV6<Hash> {
		description: Vec<u8>,
		files: Vec<CollectionItemFile<Hash>>,
	}

	type CollectionItemV6Of<T> = CollectionItemV6<<T as pallet::Config>::Hash>;

	pub struct AddTokenToCollectionItem<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for AddTokenToCollectionItem<T> {

		fn on_runtime_upgrade() -> Weight {
			super::do_storage_upgrade::<T, _>(
				StorageVersion::V6ItemUpload, 
				StorageVersion::V7ItemToken, 
				"AddTokenToCollectionItem",
				|| {
					CollectionItemsMap::<T>::translate(|_loc_id: T::LocId, _item_id: T::CollectionItemId, item: CollectionItemV6Of<T>| {
						let new_item = CollectionItemOf::<T> {
							description: item.description.clone(),
							files: item.files.clone(),
							token: Option::None,
						};
						Some(new_item)
					});
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
		log::info!("✅ {:?} migration successfully exected", migration_name);
		T::BlockWeights::get().max_block
	} else {
		if storage_version != target_version {
			log::warn!("❗ {:?} cannot run migration with storage version {:?} (expected {:?})", migration_name, storage_version, expected_version);
		} else {
			log::warn!("❎ {:?} execution skipped, was already applied", migration_name);
		}
		0
	}
}

pub mod v6 {
	use crate::{LocVoidInfo, Requester, CollectionSize};

	use super::*;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
	struct LegalOfficerCaseV5<AccountId, Hash, LocId, BlockNumber> {
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
	}

	type LegalOfficerCaseOfV5<T> = LegalOfficerCaseV5<<T as frame_system::Config>::AccountId, <T as pallet::Config>::Hash, <T as pallet::Config>::LocId, <T as frame_system::Config>::BlockNumber>;

	pub fn migrate<T: Config>() -> Weight {
		<LocMap<T>>::translate::<LegalOfficerCaseOfV5<T>, _>(
			|loc_id: T::LocId, loc: LegalOfficerCaseOfV5<T>| {
				log::info!("Migrating LOC: {:?}", loc_id);
				log::info!("From: {:?}", loc);
				let new_loc = LegalOfficerCaseOf::<T> {
					owner: loc.owner.clone(),
					requester: loc.requester.clone(),
					metadata: loc.metadata.clone(),
					files: loc.files.clone(),
					closed: loc.closed.clone(),
					loc_type: loc.loc_type.clone(),
					links: loc.links.clone(),
					void_info: loc.void_info.clone(),
					replacer_of: loc.replacer_of.clone(),
					collection_last_block_submission: loc.collection_last_block_submission.clone(),
					collection_max_size: loc.collection_max_size.clone(),
					collection_can_upload: false,
				};
				log::info!("To: {:?}", new_loc);
				Some(new_loc)
			}
		);
		let count = <LocMap<T>>::iter().count();
		T::DbWeight::get().reads_writes(count as Weight + 1, count as Weight + 1)
	}
}
