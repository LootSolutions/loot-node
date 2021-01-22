#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
use frame_support::traits::ExistenceRequirement;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get};
use frame_system::ensure_signed;

use sp_runtime::{traits::StaticLookup, DispatchError, DispatchResult};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait + orml_nft::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type RoyaltyFee: Get<BalanceOf<Self>>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as TemplateModule {
        pub Info get(fn info): map hasher(blake2_128_concat) T::ClassId => Option<(bool, BalanceOf<T>, u64)>;
        pub Sales get(fn sales): double_map hasher(twox_64_concat) T::ClassId, hasher(twox_64_concat) T::TokenId => Option<BalanceOf<T>>;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        ClassId = <T as orml_nft::Trait>::ClassId,
        Balance = BalanceOf<T>,
        TokenId = <T as orml_nft::Trait>::TokenId,
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        OrmlNftClassCreated(AccountId, ClassId),
        OrmlNftTokenMinted(AccountId, TokenId),
        OrmlNftTokenTransferred(AccountId, AccountId, ClassId, TokenId),
        RoyaltySent(AccountId, Balance),
        TokenSaleCreated(ClassId, TokenId),
        TokenSaleDeleted(ClassId, TokenId),
        TokenSaleCompleted(AccountId, ClassId, TokenId),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        InvalidClassId,
        CantMint,
        InvalidPermission,
        TokenNotFound,
        TokenNotOwned,
        TokenNotForSale,
        BuyerSellerSame,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        //
        // "CID": "Vec<u8>"
        // https://github.com/open-web3-stack/open-runtime-module-library/blob/f278c766d8bcc36b94c0e0c63d1205a4e5351841/nft/src/lib.rs#L34
        //
        // "ClassData": "u32"
        // "TokenData": "u32"
        // https://github.com/open-web3-stack/open-runtime-module-library/blob/f278c766d8bcc36b94c0e0c63d1205a4e5351841/nft/src/lib.rs#L66
        //
        // "ClassId": "u64"
        // "TokenId": "u64"
        // https://github.com/open-web3-stack/open-runtime-module-library/blob/f278c766d8bcc36b94c0e0c63d1205a4e5351841/nft/src/lib.rs#L62
        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn create_nft_class(origin, class_metadata: orml_nft::CID, class_data : <T as orml_nft::Trait>::ClassData, price: BalanceOf<T>, royalty: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let class_id = orml_nft::Module::<T>::next_class_id();
            let token_id = orml_nft::Module::<T>::create_class(&who, class_metadata, class_data)?;

            Info::<T>::insert(class_id, (true, price, royalty));

            Self::deposit_event(RawEvent::OrmlNftClassCreated(who, token_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn set_mintable(origin, class_id: T::ClassId, can_mint: bool) -> DispatchResult {
            Self::ensure_class_owner(origin, class_id)?;

            Info::<T>::try_mutate(class_id, |info| -> DispatchResult {
                let (_, price, royalty) = info.ok_or(Error::<T>::InvalidClassId)?;
                *info = Some((can_mint, price, royalty));

                Ok(())
            })?;

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn set_price(origin, class_id: T::ClassId, price: BalanceOf<T>) -> DispatchResult {
            Self::ensure_class_owner(origin, class_id)?;

            Info::<T>::try_mutate(class_id, |info| -> DispatchResult {
                let (can_mint, _, royalty) = info.ok_or(Error::<T>::InvalidClassId)?;
                *info = Some((can_mint, price, royalty));

                 Ok(())
            })?;

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn set_royalty(origin, class_id: T::ClassId, royalty: u64) -> DispatchResult {
            Self::ensure_class_owner(origin, class_id)?;

            Info::<T>::try_mutate(class_id, |info| -> DispatchResult {
                let (can_mint, price, _) = info.ok_or(Error::<T>::InvalidClassId)?;
                *info = Some((can_mint, price, royalty));

                Ok(())
            })?;

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn mint_nft_token(origin, class_id: T::ClassId, metadata: orml_nft::CID, data: <T as orml_nft::Trait>::TokenData) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (can_mint, price, _) = Self::info(class_id).ok_or(Error::<T>::InvalidClassId)?;

            ensure!(can_mint, Error::<T>::CantMint);

            let token_id = orml_nft::Module::<T>::mint(&who, class_id, metadata, data)?;

            let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::InvalidClassId)?;
            T::Currency::transfer(&who, &class_info.owner, price, ExistenceRequirement::KeepAlive)?;

            Self::deposit_event(RawEvent::OrmlNftTokenMinted(who, token_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn nft_transfer(origin, dest: <T::Lookup as StaticLookup>::Source, token_class_id: T::ClassId, token_id: T::TokenId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let to: T::AccountId = T::Lookup::lookup(dest)?;
            Self::send_royalties(&who, token_class_id)?;
            orml_nft::Module::<T>::transfer(&who, &to, (token_class_id, token_id))?;
            Self::deposit_event(RawEvent::OrmlNftTokenTransferred(who, to, token_class_id, token_id));
            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn create_sale(origin, class_id: T::ClassId, token_id: T::TokenId, price: BalanceOf<T>) -> DispatchResult {
            Self::ensure_token_owner(origin, (class_id, token_id))?;
            Sales::<T>::insert(class_id, token_id, price);
            Self::deposit_event(RawEvent::TokenSaleCreated(class_id, token_id));
            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn delete_sale(origin, class_id: T::ClassId, token_id: T::TokenId) -> DispatchResult {
            ensure!(Sales::<T>::contains_key(class_id, token_id), Error::<T>::TokenNotForSale);
            Self::ensure_token_owner(origin, (class_id, token_id))?;
            Sales::<T>::remove(class_id, token_id);
            Self::deposit_event(RawEvent::TokenSaleDeleted(class_id, token_id));
            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn buy(origin, class_id: T::ClassId, token_id: T::TokenId) -> DispatchResult {
            ensure!(Sales::<T>::contains_key(class_id, token_id), Error::<T>::TokenNotForSale);
            let buyer = ensure_signed(origin)?;
            let token_info = orml_nft::Module::<T>::tokens(class_id, token_id).ok_or(Error::<T>::TokenNotFound)?;
            let token_owner = token_info.owner;

            // can't buy your own sale
            ensure!(buyer != token_owner, Error::<T>::BuyerSellerSame);

            //transfer the nft
            orml_nft::Module::<T>::transfer(&token_owner, &buyer, (class_id, token_id))?;

            //send over funds to seller for purchase
            let price = Sales::<T>::take(class_id, token_id).ok_or(Error::<T>::TokenNotForSale)?;
            T::Currency::transfer(
                &buyer,
                &token_owner,
                price,
                ExistenceRequirement::KeepAlive,
            )?;

            //send royalties to class owner from the token owner who sold it
            Self::send_royalties(&token_owner, class_id)?;
            Self::deposit_event(RawEvent::TokenSaleCompleted(buyer, class_id, token_id));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn send_royalties(who: &T::AccountId, class_id: T::ClassId) -> DispatchResult {
        let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::InvalidClassId)?;

        let royalty = T::RoyaltyFee::get();

        T::Currency::transfer(
            who,
            &class_info.owner,
            royalty,
            ExistenceRequirement::KeepAlive,
        )?;

        Self::deposit_event(RawEvent::RoyaltySent(who.clone(), royalty));

        Ok(())
    }

    fn ensure_class_owner(
        origin: T::Origin,
        class_id: T::ClassId,
    ) -> Result<T::AccountId, DispatchError> {
        let who = ensure_signed(origin)?;
        let class_info =
            orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::InvalidClassId)?;

        ensure!(who == class_info.owner, Error::<T>::InvalidPermission);

        Ok(who)
    }

    fn ensure_token_owner(
        origin: T::Origin,
        token: (T::ClassId, T::TokenId),
    ) -> Result<T::AccountId, DispatchError> {
        let who = ensure_signed(origin)?;
        ensure!(orml_nft::Module::<T>::tokens_by_owner(who.clone(), token).is_some(), Error::<T>::TokenNotOwned);
        Ok(who)
    }
}
