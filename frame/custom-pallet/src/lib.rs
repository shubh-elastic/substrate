// All pallets must be configured for `no_std`.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
 use frame_support::pallet_prelude::*;
 use frame_system::pallet_prelude::*;
 use sp_core::H160;

 #[pallet::pallet]
 pub struct Pallet<T>(_);



//  #[pallet::config]  // <-- Step 2. code block will replace this.
/// Configure the pallet by specifying the parameters and types on which it depends.
#[pallet::config]
pub trait Config: frame_system::Config {
 /// Because this pallet emits events, it depends on the runtime's definition of an event.
 type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
}
//  #[pallet::event]   // <-- Step 3. code block will replace this.

// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
 /// Event emitted when a claim has been created.
 NFTAdded { who: H160, val: u32 },
 /// Event emitted when a claim is revoked by the owner.
 ClaimRevoked { who: T::AccountId, claim: T::Hash },
}


//  #[pallet::error]   // <-- Step 4. code block will replace this.

#[pallet::error]
pub enum Error<T> {
 /// The claim already exists.
 AlreadyClaimed,
 /// The claim does not exist, so it cannot be revoked.
 NoSuchClaim,
 /// The claim is owned by another account, so caller can't revoke it.
 NotClaimOwner,
}


//  #[pallet::storage] // <-- Step 5. code block will replace this.
#[pallet::storage]
#[pallet::getter(fn nfts)]
pub  type NFTs<T: Config> = StorageMap<_, Blake2_128Concat, H160, u32>;

// pub(super) type Claims<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, BlockNumberFor<T>)>;

// Dispatchable functions allow users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
#[pallet::call]
impl<T: Config> Pallet<T> {
 #[pallet::weight(Weight::default())]
 #[pallet::call_index(0)]
 pub fn add_nft(origin: OriginFor<T>, account: H160, val: u32) -> DispatchResult {
   // Check that the extrinsic was signed and get the signer.
   // This function will return an error if the extrclaiminsic is not signed.
   let _sender = ensure_root(origin)?;

   // Verify that the specified claim has not already been stored.
  //  ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);
  

   // Store the claim with the sender and block number.
   NFTs::<T>::insert(&account, val);

   // Emit an event that the claim was created.
   Self::deposit_event(Event:: NFTAdded { who: account, val: val });

   Ok(())
 }

//  #[pallet::weight(Weight::default())]
//  #[pallet::call_index(1)]
//  pub fn revoke_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
//    // Check that the extrinsic was signed and get the signer.
//    // This function will return an error if the extrinsic is not signed.
//    let sender = ensure_signed(origin)?;

//    // Get owner of the claim, if none return an error.
//    let (owner, _) = Claims::<T>::get(&claim).ok_or(Error::<T>::NoSuchClaim)?;

//    // Verify that sender of the current call is the claim owner.
//    ensure!(sender == owner, Error::<T>::NotClaimOwner);

//    // Remove claim from storage.
//    Claims::<T>::remove(&claim);

//    // Emit an event that the claim was erased.
//    Self::deposit_event(Event::ClaimRevoked { who: sender, claim });
//    Ok(())
//  }
}
}

pub mod weights {
  // Placeholder struct for the pallet weights
  pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
}
