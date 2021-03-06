use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::dispatch::Vec;
use frame_support::weights::Weight;

use crate::{Config, File, LegalOfficerCaseOf, LocLink, LocMap, LocType, MetadataItem, pallet, PalletStorageVersion, StorageVersion};

pub fn migrate<T: Config>() -> Weight {
	do_migrate::<T, _>(StorageVersion::V5Collection, v5::migrate::<T>)
}

fn do_migrate<T: Config, F>(from: StorageVersion, migration_fn: F) -> Weight
	where F: FnOnce() -> Weight {
	let stored_version = <PalletStorageVersion<T>>::try_get();
	let to: StorageVersion = Default::default();
	if stored_version.is_err() || stored_version.unwrap() == from {
		log::info!("Starting to migrate from {:?} to {:?}", from, &to);
		let weight = migration_fn();
		<PalletStorageVersion<T>>::put(&to);
		log::info!("Migration ended.");
		weight
	} else {
		log::info!("The migration {:?} to {:?} was already applied.", from, &to);
		0
	}
}

mod v5 {
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

	pub(crate) fn migrate<T: Config>() -> Weight {
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
