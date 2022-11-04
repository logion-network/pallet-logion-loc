use frame_support::{assert_err, assert_ok};
use frame_support::error::BadOrigin;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::Hash;

use logion_shared::LocQuery;

use crate::{File, LegalOfficerCase, LocLink, LocType, MetadataItem, CollectionItem, CollectionItemFile, CollectionItemToken, mock::*, TermsAndConditionsElement};
use crate::Error;

const LOC_ID: u32 = 0;
const OTHER_LOC_ID: u32 = 1;
const LOGION_CLASSIFICATION_LOC_ID: u32 = 2;
const ADDITIONAL_TC_LOC_ID: u32 = 3;

#[test]
fn it_creates_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_eq!(LogionLoc::loc(LOC_ID), Some(LegalOfficerCase::<<Test as frame_system::Config>::AccountId, <Test as crate::Config>::Hash, <Test as crate::Config>::LocId, <Test as frame_system::Config>::BlockNumber> {
			owner: LOC_OWNER1,
			requester: LOC_REQUESTER,
			metadata: vec![],
			files: vec![],
			closed: false,
			loc_type: LocType::Transaction,
			links: vec![],
			void_info: None,
			replacer_of: None,
			collection_last_block_submission: Option::None,
			collection_max_size: Option::None,
			collection_can_upload: false,
			seal: Option::None,
		}));
	});
}

#[test]
fn it_makes_existing_loc_void() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let void_info = LogionLoc::loc(LOC_ID).unwrap().void_info;
		assert!(void_info.is_some());
		assert!(!void_info.unwrap().replacer.is_some());
	});
}

#[test]
fn it_makes_existing_loc_void_and_replace_it() {
	new_test_ext().execute_with(|| {
		create_closed_loc();

		const REPLACER_LOC_ID: u32 = OTHER_LOC_ID;
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), REPLACER_LOC_ID, LOC_REQUESTER_ID));

		assert_ok!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, REPLACER_LOC_ID));

		let void_info = LogionLoc::loc(LOC_ID).unwrap().void_info;
		assert!(void_info.is_some());
		let replacer: Option<u32> = void_info.unwrap().replacer;
		assert!(replacer.is_some());
		assert_eq!(replacer.unwrap(), REPLACER_LOC_ID);

		let replacer_loc = LogionLoc::loc(REPLACER_LOC_ID).unwrap();
		assert!(replacer_loc.replacer_of.is_some());
		assert_eq!(replacer_loc.replacer_of.unwrap(), LOC_ID)
	});
}

#[test]
fn it_fails_making_existing_loc_void_for_unauthorized_caller() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_err!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID), Error::<Test>::Unauthorized);
		let void_info = LogionLoc::loc(LOC_ID).unwrap().void_info;
		assert!(!void_info.is_some());
	});
}

#[test]
fn it_fails_making_existing_loc_void_for_already_void_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
		assert_err!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID), Error::<Test>::AlreadyVoid);
	});
}

#[test]
fn it_fails_replacing_with_non_existent_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_err!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, OTHER_LOC_ID), Error::<Test>::ReplacerLocNotFound);
	});
}

#[test]
fn it_fails_replacing_with_void_loc() {
	new_test_ext().execute_with(|| {
		const REPLACER_LOC_ID: u32 = OTHER_LOC_ID;
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID));
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_err!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, REPLACER_LOC_ID), Error::<Test>::ReplacerLocAlreadyVoid);
	});
}

#[test]
fn it_fails_replacing_with_loc_already_replacing_another_loc() {
	new_test_ext().execute_with(|| {
		const REPLACER_LOC_ID: u32 = 2;
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), REPLACER_LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, REPLACER_LOC_ID));
		assert_err!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, REPLACER_LOC_ID), Error::<Test>::ReplacerLocAlreadyReplacing);
	});
}

#[test]
fn it_fails_replacing_with_wrongly_typed_loc() {
	new_test_ext().execute_with(|| {
		const REPLACER_LOC_ID: u32 = 2;
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), REPLACER_LOC_ID, LOC_REQUESTER_ID));
		assert_err!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, REPLACER_LOC_ID), Error::<Test>::ReplacerLocWrongType);
	});
}

#[test]
fn it_adds_metadata_when_submitter_is_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let metadata = MetadataItem {
			name: vec![1, 2, 3],
			value: vec![4, 5, 6],
			submitter: LOC_OWNER1,
		};
		assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()));
		let loc = LogionLoc::loc(LOC_ID).unwrap();
		assert_eq!(loc.metadata[0], metadata);
	});
}

#[test]
fn it_adds_metadata_when_submitter_is_requester() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let metadata = MetadataItem {
			name: vec![1, 2, 3],
			value: vec![4, 5, 6],
			submitter: LOC_REQUESTER_ID,
		};
		assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()));
		let loc = LogionLoc::loc(LOC_ID).unwrap();
		assert_eq!(loc.metadata[0], metadata);
	});
}

#[test]
fn it_fails_adding_metadata_for_unauthorized_caller() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let metadata = MetadataItem {
			name: vec![1, 2, 3],
			value: vec![4, 5, 6],
			submitter: LOC_OWNER1,
		};
		assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, metadata.clone()), Error::<Test>::Unauthorized);
	});
}

#[test]
fn it_fails_adding_metadata_when_closed() {
	new_test_ext().execute_with(|| {
		create_closed_loc();
		let metadata = MetadataItem {
			name: vec![1, 2, 3],
			value: vec![4, 5, 6],
			submitter: LOC_OWNER1,
		};
		assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()), Error::<Test>::CannotMutate);
	});
}

#[test]
fn it_fails_adding_metadata_on_polkadot_transaction_loc_when_submitter_is_invalid() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let metadata = MetadataItem {
			name: vec![1, 2, 3],
			value: vec![4, 5, 6],
			submitter: LOC_OWNER2,
		};
		assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()), Error::<Test>::InvalidSubmitter);
	});
}

#[test]
fn it_fails_adding_metadata_on_logion_identity_loc_for_when_submitter_is_invalid() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
		let metadata = MetadataItem {
			name: vec![1, 2, 3],
			value: vec![4, 5, 6],
			submitter: LOC_OWNER2,
		};
		assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()), Error::<Test>::InvalidSubmitter);
	});
}

fn create_closed_loc() {
	assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
	assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
}

#[test]
fn it_adds_file_when_submitter_is_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let file = File {
			hash: BlakeTwo256::hash_of(&"test".as_bytes().to_vec()),
			nature: "test-file-nature".as_bytes().to_vec(),
			submitter: LOC_OWNER1,
		};
		assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()));
		let loc = LogionLoc::loc(LOC_ID).unwrap();
		assert_eq!(loc.files[0], file);
	});
}

#[test]
fn it_adds_file_when_submitter_is_requester() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let file = File {
			hash: BlakeTwo256::hash_of(&"test".as_bytes().to_vec()),
			nature: "test-file-nature".as_bytes().to_vec(),
			submitter: LOC_REQUESTER_ID,
		};
		assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()));
		let loc = LogionLoc::loc(LOC_ID).unwrap();
		assert_eq!(loc.files[0], file);
	});
}

#[test]
fn it_fails_adding_file_for_unauthorized_caller() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let file = File {
			hash: BlakeTwo256::hash_of(&"test".as_bytes().to_vec()),
			nature: "test-file-nature".as_bytes().to_vec(),
			submitter: LOC_OWNER1,
		};
		assert_err!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, file.clone()), Error::<Test>::Unauthorized);
	});
}

#[test]
fn it_fails_adding_file_when_closed() {
	new_test_ext().execute_with(|| {
		create_closed_loc();
		let file = File {
			hash: BlakeTwo256::hash_of(&"test".as_bytes().to_vec()),
			nature: "test-file-nature".as_bytes().to_vec(),
			submitter: LOC_OWNER1,
		};
		assert_err!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()), Error::<Test>::CannotMutate);
	});
}

#[test]
fn it_fails_adding_file_on_polkadot_transaction_loc_when_submitter_is_invalid() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let file = File {
			hash: BlakeTwo256::hash_of(&"test".as_bytes().to_vec()),
			nature: "test-file-nature".as_bytes().to_vec(),
			submitter: LOC_OWNER2,
		};
		assert_err!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()), Error::<Test>::InvalidSubmitter);
	});
}

#[test]
fn it_fails_adding_file_on_logion_identity_loc_when_submitter_is_invalid() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
		let file = File {
			hash: BlakeTwo256::hash_of(&"test".as_bytes().to_vec()),
			nature: "test-file-nature".as_bytes().to_vec(),
			submitter: LOC_OWNER2,
		};
		assert_err!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()), Error::<Test>::InvalidSubmitter);
	});
}

#[test]
fn it_adds_link() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOC_REQUESTER_ID));
		let link = LocLink {
			id: OTHER_LOC_ID,
			nature: "test-link-nature".as_bytes().to_vec()
		};
		assert_ok!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, link.clone()));
		let loc = LogionLoc::loc(LOC_ID).unwrap();
		assert_eq!(loc.links[0], link);
	});
}

#[test]
fn it_fails_adding_link_for_unauthorized_caller() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOC_REQUESTER_ID));
		let link = LocLink {
			id: OTHER_LOC_ID,
			nature: "test-link-nature".as_bytes().to_vec()
		};
		assert_err!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, link.clone()), Error::<Test>::Unauthorized);
	});
}

#[test]
fn it_fails_adding_link_when_closed() {
	new_test_ext().execute_with(|| {
		create_closed_loc();
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOC_REQUESTER_ID));
		let link = LocLink {
			id: OTHER_LOC_ID,
			nature: "test-link-nature".as_bytes().to_vec()
		};
		assert_err!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, link.clone()), Error::<Test>::CannotMutate);
	});
}

#[test]
fn it_fails_adding_wrong_link() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let link = LocLink {
			id: OTHER_LOC_ID,
			nature: "test-link-nature".as_bytes().to_vec()
		};
		assert_err!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, link.clone()), Error::<Test>::LinkedLocNotFound);
	});
}

#[test]
fn it_closes_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
		let loc = LogionLoc::loc(LOC_ID).unwrap();
		assert!(loc.closed);
		assert!(loc.seal.is_none());
	});
}

#[test]
fn it_fails_closing_loc_for_unauthorized_caller() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_err!(LogionLoc::close(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID), Error::<Test>::Unauthorized);
	});
}

#[test]
fn it_fails_closing_loc_for_already_closed() {
	new_test_ext().execute_with(|| {
		create_closed_loc();
		assert_err!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID), Error::<Test>::AlreadyClosed);
	});
}

#[test]
fn it_links_locs_to_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOC_REQUESTER_ID));
		assert!(LogionLoc::account_locs(LOC_REQUESTER_ID).is_some());
		assert!(LogionLoc::account_locs(LOC_REQUESTER_ID).unwrap().len() == 2);
		assert_eq!(LogionLoc::account_locs(LOC_REQUESTER_ID).unwrap()[0], LOC_ID);
		assert_eq!(LogionLoc::account_locs(LOC_REQUESTER_ID).unwrap()[1], OTHER_LOC_ID);
	});
}

#[test]
fn it_fails_creating_loc_for_unauthorized_caller() {
	new_test_ext().execute_with(|| {
		assert_err!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_REQUESTER_ID), BadOrigin);
	});
}

#[test]
fn it_detects_existing_identity_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_OWNER2), OTHER_LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER2), OTHER_LOC_ID));

		let legal_officers = Vec::from([LOC_OWNER1, LOC_OWNER2]);
		assert!(LogionLoc::has_closed_identity_locs(&LOC_REQUESTER_ID, &legal_officers));
	});
}

#[test]
fn it_creates_logion_identity_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));

		assert!(LogionLoc::loc(LOGION_IDENTITY_LOC_ID).is_some());
		assert!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).is_none());
	});
}

#[test]
fn it_creates_and_links_logion_locs_to_identity_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));

		assert_ok!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOGION_IDENTITY_LOC_ID));
		assert_ok!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOGION_IDENTITY_LOC_ID));

		assert!(LogionLoc::loc(LOC_ID).is_some());
		assert!(LogionLoc::loc(OTHER_LOC_ID).is_some());
		assert!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).is_some());
		assert!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).unwrap().len() == 2);
		assert_eq!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).unwrap()[0], LOC_ID);
		assert_eq!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).unwrap()[1], OTHER_LOC_ID);
	});
}

#[test]
fn it_fails_creating_logion_loc_with_polkadot_identity_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOC_REQUESTER_ID));

		assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, OTHER_LOC_ID), Error::<Test>::UnexpectedRequester);
	});
}

#[test]
fn it_fails_creating_logion_loc_with_polkadot_transaction_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOC_REQUESTER_ID));

		assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, OTHER_LOC_ID), Error::<Test>::UnexpectedRequester);
	});
}

#[test]
fn it_fails_creating_logion_loc_with_logion_transaction_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
		assert_ok!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOGION_IDENTITY_LOC_ID));

		assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, OTHER_LOC_ID), Error::<Test>::UnexpectedRequester);
	});
}

#[test]
fn it_fails_creating_logion_loc_with_open_logion_identity_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));

		assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOGION_IDENTITY_LOC_ID), Error::<Test>::UnexpectedRequester);
	});
}

#[test]
fn it_fails_creating_logion_loc_with_closed_void_logion_identity_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
		assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));

		assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOGION_IDENTITY_LOC_ID), Error::<Test>::UnexpectedRequester);
	});
}

#[test]
fn it_creates_collection_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(10), false));
		assert_eq!(LogionLoc::loc(LOC_ID), Some(LegalOfficerCase::<<Test as frame_system::Config>::AccountId, <Test as crate::Config>::Hash, <Test as crate::Config>::LocId, <Test as frame_system::Config>::BlockNumber> {
			owner: LOC_OWNER1,
			requester: LOC_REQUESTER,
			metadata: vec![],
			files: vec![],
			closed: false,
			loc_type: LocType::Collection,
			links: vec![],
			void_info: None,
			replacer_of: None,
			collection_last_block_submission: Option::None,
			collection_max_size: Option::Some(10),
			collection_can_upload: false,
			seal: Option::None,
		}));
	});
}

#[test]
fn it_fails_creating_collection_loc_without_limit() {
	new_test_ext().execute_with(|| {
		assert_err!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::None, false), Error::<Test>::CollectionHasNoLimit);
	});
}

#[test]
fn it_fails_adding_item_to_open_collection_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(10), false));
		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, collection_item_id, collection_item_description, vec![], Option::None, false), Error::<Test>::WrongCollectionLoc);
	});
}

#[test]
fn it_adds_item_to_closed_collection_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(10), false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], Option::None, false));
		assert_eq!(LogionLoc::collection_items(LOC_ID, collection_item_id), Some(CollectionItem {
			description: collection_item_description,
			files: vec![],
			token: Option::None,
			restricted_delivery: false,
			terms_and_conditions: vec![],
		}));
		assert_eq!(LogionLoc::collection_size(LOC_ID), Some(1));
	});
}

#[test]
fn it_fails_to_item_with_terms_and_conditions_when_non_existent_tc_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(10), false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let terms_and_conditions_details = "ITEM-A, ITEM-B".as_bytes().to_vec();
		let terms_and_conditions = vec![TermsAndConditionsElement {
			tc_type: "Logion".as_bytes().to_vec(),
			tc_loc: LOGION_CLASSIFICATION_LOC_ID,
			details: terms_and_conditions_details.clone()
		}];
		assert_err!(LogionLoc::add_collection_item_with_terms_and_conditions(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], Option::None, false, terms_and_conditions), Error::<Test>::TermsAndConditionsLocNotFound);
	});
}

#[test]
fn it_fails_to_item_with_terms_and_conditions_when_open_tc_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(10), false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_CLASSIFICATION_LOC_ID, LOC_REQUESTER_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let terms_and_conditions_details = "ITEM-A, ITEM-B".as_bytes().to_vec();
		let terms_and_conditions = vec![TermsAndConditionsElement {
			tc_type: "Logion".as_bytes().to_vec(),
			tc_loc: LOGION_CLASSIFICATION_LOC_ID,
			details: terms_and_conditions_details.clone()
		}];
		assert_err!(LogionLoc::add_collection_item_with_terms_and_conditions(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], Option::None, false, terms_and_conditions), Error::<Test>::TermsAndConditionsLocNotClosed);
	});
}

#[test]
fn it_fails_to_item_with_terms_and_conditions_when_void_tc_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(10), false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_CLASSIFICATION_LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOGION_CLASSIFICATION_LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let terms_and_conditions_details = "ITEM-A, ITEM-B".as_bytes().to_vec();
		let terms_and_conditions = vec![TermsAndConditionsElement {
			tc_type: "Logion".as_bytes().to_vec(),
			tc_loc: LOGION_CLASSIFICATION_LOC_ID,
			details: terms_and_conditions_details.clone()
		}];
		assert_err!(LogionLoc::add_collection_item_with_terms_and_conditions(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], Option::None, false, terms_and_conditions), Error::<Test>::TermsAndConditionsLocVoid);
	});
}

#[test]
fn it_adds_item_with_terms_and_conditions_to_closed_collection_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(10), false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_CLASSIFICATION_LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOGION_CLASSIFICATION_LOC_ID));
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), ADDITIONAL_TC_LOC_ID, LOC_REQUESTER_ID));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), ADDITIONAL_TC_LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let tc1 = TermsAndConditionsElement {
			tc_type: "Logion".as_bytes().to_vec(),
			tc_loc: LOGION_CLASSIFICATION_LOC_ID,
			details: "ITEM-A, ITEM-B".as_bytes().to_vec().clone()
		};
		let tc2 = TermsAndConditionsElement {
			tc_type: "Specific".as_bytes().to_vec(),
			tc_loc: ADDITIONAL_TC_LOC_ID,
			details: "Some more details".as_bytes().to_vec().clone()
		};
		let terms_and_conditions = vec![tc1, tc2];
		assert_ok!(LogionLoc::add_collection_item_with_terms_and_conditions(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], Option::None, false, terms_and_conditions.clone()));
		assert_eq!(LogionLoc::collection_items(LOC_ID, collection_item_id), Some(CollectionItem {
			description: collection_item_description,
			files: vec![],
			token: None,
			restricted_delivery: false,
			terms_and_conditions: terms_and_conditions.clone(),
		}));
		assert_eq!(LogionLoc::collection_size(LOC_ID), Some(1));
	});
}

#[test]
fn it_fails_adding_item_to_collection_loc_if_not_requester() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(10), false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, collection_item_id, collection_item_description, vec![], Option::None, false), Error::<Test>::WrongCollectionLoc);
	});
}

#[test]
fn it_fails_adding_item_if_duplicate_key() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(10), false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id.clone(), collection_item_description.clone(), vec![], Option::None, false));
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, vec![], Option::None, false), Error::<Test>::CollectionItemAlreadyExists);
	});
}

#[test]
fn it_fails_adding_item_if_size_limit_reached() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id.clone(), collection_item_description.clone(), vec![], Option::None, false));
		let collection_item_id2 = BlakeTwo256::hash_of(&"item-id2".as_bytes().to_vec());
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id2, collection_item_description, vec![], Option::None, false), Error::<Test>::CollectionLimitsReached);
	});
}

#[test]
fn it_fails_adding_item_if_block_limit_reached() {
	let current_block: u64 = 10;
	new_test_ext_at_block(current_block).execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::Some(current_block - 1), Option::None, false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, vec![], Option::None, false), Error::<Test>::CollectionLimitsReached);
	});
}

#[test]
fn it_fails_adding_item_if_collection_void() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), false));
		assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, vec![], Option::None, false), Error::<Test>::WrongCollectionLoc);
	});
}

#[test]
fn it_fails_adding_item_if_files_attached_but_upload_not_enabled() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), false));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let collection_item_files = vec![CollectionItemFile {
			name: "picture.png".as_bytes().to_vec(),
			content_type: "image/png".as_bytes().to_vec(),
			hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
			size: 123456,
		}];
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Option::None, false), Error::<Test>::CannotUpload);
	});
}

#[test]
fn it_fails_adding_item_if_no_files_attached_but_upload_enabled() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), true));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, vec![], Option::None, false), Error::<Test>::MustUpload);
	});
}

#[test]
fn it_adds_item_with_one_file_attached() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), true));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let collection_item_files = vec![CollectionItemFile {
			name: "picture.png".as_bytes().to_vec(),
			content_type: "image/png".as_bytes().to_vec(),
			hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
			size: 123456,
		}];
		assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Option::None, false));
	});
}

#[test]
fn it_adds_item_with_token() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), true));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let collection_item_files = vec![CollectionItemFile {
			name: "picture.png".as_bytes().to_vec(),
			content_type: "image/png".as_bytes().to_vec(),
			hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
			size: 123456,
		}];
		let collection_item_token = CollectionItemToken {
			token_type: "ethereum_erc721".as_bytes().to_vec(),
			token_id: "{\"contract\":\"0x765df6da33c1ec1f83be42db171d7ee334a46df5\",\"token\":\"4391\"}".as_bytes().to_vec(),
		};
		assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Option::Some(collection_item_token), true));
	});
}

#[test]
fn it_fails_adding_item_with_too_large_token_type() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), true));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let collection_item_files = vec![CollectionItemFile {
			name: "picture.png".as_bytes().to_vec(),
			content_type: "image/png".as_bytes().to_vec(),
			hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
			size: 123456,
		}];
		let collection_item_token = CollectionItemToken {
			token_type: vec![0; 256],
			token_id: "{\"contract\":\"0x765df6da33c1ec1f83be42db171d7ee334a46df5\",\"token\":\"4391\"}".as_bytes().to_vec(),
		};
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Option::Some(collection_item_token), true), Error::<Test>::CollectionItemTooMuchData);
	});
}

#[test]
fn it_fails_adding_item_with_too_large_token_id() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), true));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let collection_item_files = vec![CollectionItemFile {
			name: "picture.png".as_bytes().to_vec(),
			content_type: "image/png".as_bytes().to_vec(),
			hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
			size: 123456,
		}];
		let collection_item_token = CollectionItemToken {
			token_type: "ethereum_erc721".as_bytes().to_vec(),
			token_id: vec![0; 256],
		};
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Option::Some(collection_item_token), true), Error::<Test>::CollectionItemTooMuchData);
	});
}

#[test]
fn it_fails_adding_item_with_missing_token() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), true));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let collection_item_files = vec![CollectionItemFile {
			name: "picture.png".as_bytes().to_vec(),
			content_type: "image/png".as_bytes().to_vec(),
			hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
			size: 123456,
		}];
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Option::None, true), Error::<Test>::MissingToken);
	});
}

#[test]
fn it_fails_adding_item_with_missing_files() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), true));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let collection_item_files = vec![];
		let collection_item_token = CollectionItemToken {
			token_type: "ethereum_erc721".as_bytes().to_vec(),
			token_id: "{\"contract\":\"0x765df6da33c1ec1f83be42db171d7ee334a46df5\",\"token\":\"4391\"}".as_bytes().to_vec(),
		};
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Option::Some(collection_item_token), true), Error::<Test>::MissingFiles);
	});
}

#[test]
fn it_adds_item_with_two_files_attached() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), true));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let collection_item_files = vec![
			CollectionItemFile {
				name: "picture.png".as_bytes().to_vec(),
				content_type: "image/png".as_bytes().to_vec(),
				hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
				size: 123456,
			},
			CollectionItemFile {
				name: "doc.pdf".as_bytes().to_vec(),
				content_type: "application/pdf".as_bytes().to_vec(),
				hash: BlakeTwo256::hash_of(&"some other content".as_bytes().to_vec()),
				size: 789,
			},
		];
		assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Option::None, false));
	});
}

#[test]
fn it_fails_to_add_item_with_duplicate_hash() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID, Option::None, Option::Some(1), true));
		assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

		let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
		let collection_item_description = "item-description".as_bytes().to_vec();
		let same_hash = BlakeTwo256::hash_of(&"file content".as_bytes().to_vec());
		let collection_item_files = vec![
			CollectionItemFile {
				name: "picture.png".as_bytes().to_vec(),
				content_type: "image/png".as_bytes().to_vec(),
				hash: same_hash,
				size: 123456,
			},
			CollectionItemFile {
				name: "doc.pdf".as_bytes().to_vec(),
				content_type: "application/pdf".as_bytes().to_vec(),
				hash: same_hash,
				size: 789,
			},
		];
		assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Option::None, false), Error::<Test>::DuplicateFile);
	});
}

#[test]
fn it_closes_and_seals_loc() {
	new_test_ext().execute_with(|| {
		assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOC_REQUESTER_ID));
		let seal = BlakeTwo256::hash_of(&"some external private data".as_bytes().to_vec());
		assert_ok!(LogionLoc::close_and_seal(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, seal));
		let loc = LogionLoc::loc(LOC_ID).unwrap();
		assert!(loc.closed);
		assert!(loc.seal.is_some());
		assert_eq!(loc.seal.unwrap(), seal);
	});
}
