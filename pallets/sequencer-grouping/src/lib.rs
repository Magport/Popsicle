#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
use frame_system::pallet_prelude::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use scale_info::prelude::vec::Vec;
use sp_runtime::traits::Zero;
#[frame_support::pallet]
pub mod pallet {

	use super::*;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	pub type RoundIndex = u32;
	#[derive(
		Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen, Default,
	)]
	pub struct NextRound<BlockNumber> {
		pub starting_block: BlockNumber,
		pub round_index: RoundIndex,
	}
	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::type_value]
	pub fn GroupIDOnEmpty<T: Config>() -> u32 {
		0
	}

	#[pallet::storage]
	#[pallet::getter(fn groupID)]
	pub type GroupID<T> = StorageValue<_, u32, ValueQuery, GroupIDOnEmpty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn next_round_t)]
	pub type NextRoundStorage<T: Config> =
		StorageValue<_, NextRound<BlockNumberFor<T>>, ValueQuery>;
	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: BlockNumberFor<T>) {
			// let old_group_id = GroupID::<T>::get();
			// let transaction_type = block_number % 4u32.into();
			// if transaction_type == Zero::zero() {
			// 	let mut new_group_id = old_group_id + 1;
			// 	if new_group_id == 5 {
			// 		new_group_id = 1;
			// 	}
			// 	GroupID::<T>::put(new_group_id);
			// }
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(().into())
				},
			}
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn set_group(origin: OriginFor<T>, groupID: u32) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			<GroupID<T>>::put(groupID);
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn set_next_round(
			origin: OriginFor<T>,
			round: NextRound<BlockNumberFor<T>>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			<NextRoundStorage<T>>::put(round);
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn account_in_group(account: T::AccountId) -> Result<u32, DispatchError> {
		Ok(GroupID::<T>::get())
	}

	pub fn all_group_ids() -> Vec<u32> {
		let group_id = GroupID::<T>::get();
		let mut groups = Vec::new();
		for i in 0..group_id {
			groups.push(i + 1);
		}
		groups
	}

	pub fn next_round() -> NextRound<BlockNumberFor<T>> {
		NextRoundStorage::<T>::get()
	}
}
