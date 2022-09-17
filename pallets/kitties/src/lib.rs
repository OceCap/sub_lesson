/*
 * @Author: Gxp-Ning 77679755+Gxp-Ning@users.noreply.github.com
 * @Date: 2022-09-13 21:15:57
 * @LastEditors: Gxp-Ning 77679755+Gxp-Ning@users.noreply.github.com
 * @LastEditTime: 2022-09-18 00:31:53
 * @FilePath: \substrate-node-template\pallets\kitties\src\lib.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_system::{pallet_prelude::*};
    use frame_support::{pallet_prelude::*, traits::Randomness, traits::{Currency,ExistenceRequirement}};
    use sp_io::hashing::blake2_128;
    use sp_runtime::traits::{AtLeast32Bit, Bounded, CheckedAdd};
  //  type KittyIndex = u32;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    #[pallet::type_value]
    pub fn GetDefaultValue<T: Config>() -> T::KittyIndex {
        0_u32.into()
    }
    
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct Kitty(pub [u8; 16]);

    #[pallet::config]
    pub trait Config: frame_system::Config{
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type KittyIndex: AtLeast32Bit + Copy + Parameter + Default + Bounded + MaxEncodedLen;
        #[pallet::constant]
        type MAXKittyIndex: Get<u32>;
        type KittyPrice: Get<BalanceOf<Self>>;
        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_kitty_id)]
    pub type NextKittyId<T: Config> = StorageValue<
    _,
    T::KittyIndex,
    ValueQuery,
    GetDefaultValue<T>
    >;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::KittyIndex,
    Kitty
    >;

    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type KittyOwner<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::KittyIndex,
    T::AccountId
    >;

    #[pallet::storage]
    #[pallet::getter(fn all_kitties)]
    pub type AllKitties<T: Config> = StorageMap<
    _,
    Blake2_128,
    Kitty,
    T::AccountId,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreated(T::AccountId, T::KittyIndex, Kitty),
        Transfersucceed(T::AccountId, T::AccountId, T::KittyIndex),
        KittyBreed(T::AccountId, T::KittyIndex, T::KittyIndex),
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidKittyId,
        MaxKittyCount,
        NotKittyOwner,
        TransferToSelf,
        SameKittyId,
        KittyCountOverflow,
        NotEnoughBalance,
    }
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult{
            let who = ensure_signed(origin)?;
            let kitty_id = Self::get_next_id().map_err(|_| {Error::<T>::InvalidKittyId})?;
            let dna = Self::random_value(&who);
            let kitty = Kitty(dna);
            ensure!(T::Currency::can_slash(&who, T::KittyPrice::get()), Error::<T>::NotEnoughBalance);
            Kitties::<T>::insert(kitty_id, &kitty);
            KittyOwner::<T>::insert(kitty_id, &who);
            AllKitties::<T>::insert(&kitty, &who);
            T::Currency::slash(&who, T::KittyPrice::get());
            let next_kitty_id = kitty_id.checked_add(&(T::KittyIndex::from(1_u8)))
                .ok_or(Error::<T>::KittyCountOverflow)?;
            NextKittyId::<T>::set(next_kitty_id);
            //emit event
            Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn breed_kiity(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult{
            let who = ensure_signed(origin)?;
            //make sure two kitties are different
            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
            let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
            let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;
            //get next id
            let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::MaxKittyCount)?;
            //random kitty dna
            let selector = Self::random_value(&who);
            let mut data = [0u8; 16];
            //breed new kitty dna
            for i in 0..kitty_1.0.len() {
                data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & selector[i]);
            }
            let new_kitty = Kitty(data);
            //ensure this accountid have enough money
            ensure!(T::Currency::can_slash(&who, T::KittyPrice::get()), Error::<T>::NotEnoughBalance);
            //kitty + 1
            Kitties::<T>::insert(kitty_id, &new_kitty);
            KittyOwner::<T>::insert(kitty_id, &who);
            AllKitties::<T>::insert(&new_kitty, &who);
            //reserve some token
            T::Currency::slash(&who, T::KittyPrice::get());
            //next id + 1
            let next_kitty_id = kitty_id.checked_add(&(T::KittyIndex::from(1_u8)))
                .ok_or(Error::<T>::KittyCountOverflow)?;
            NextKittyId::<T>::set(next_kitty_id);
            //emit event
            Self::deposit_event(Event::KittyBreed(who, kitty_id_1, kitty_id_2));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn transfer_kitty(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;
            //cannot transfer to self
            ensure!(who != new_owner, Error::<T>::TransferToSelf);
            //this kitty must be exist
            let kitty = Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;
            //cannot transfer others kitty
            ensure!(Self::kitty_owner(&kitty_id) == Some(who.clone()), Error::<T>::NotKittyOwner);
            ensure!(T::Currency::can_slash(&new_owner, T::KittyPrice::get()), Error::<T>::NotEnoughBalance);
            KittyOwner::<T>::insert(kitty_id, &who);
            AllKitties::<T>::insert(kitty, &who);
            T::Currency::transfer(&new_owner, &who, T::KittyPrice::get(), ExistenceRequirement::KeepAlive)?;
            Self::deposit_event(Event::Transfersucceed(who, new_owner, kitty_id));
            Ok(())
        }

    }

    impl<T: Config> Pallet<T> {
        //get a random 256
        fn random_value(sender: &T::AccountId) -> [u8; 16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet::<T>>::extrinsic_index(),
            );
            payload.using_encoded(blake2_128)
        }
        //get next id
        fn get_next_id() -> Result<T::KittyIndex, ()> {
            let next_kitty_id = Self::next_kitty_id() ;
            match next_kitty_id {
                _ if T::KittyIndex::max_value() <= next_kitty_id  => Err(()),
                val => Ok(val),
                
            }
        }
        //get kitty via id
        fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
            match Self::kitties(kitty_id) {
                Some(kitty) => Ok(kitty),
                None => Err(()),
            }
        }
    }
}