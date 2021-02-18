#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error, 
	decl_event, 
	decl_module, 
	decl_storage, 
	dispatch::DispatchResult,
	ensure, 
	traits::{
		Currency, 
		ReservableCurrency, 
	},
};
use frame_system::{
	self as system, 
	ensure_signed,
	ensure_root,
};
use parity_scale_codec::{
	Decode, 
	Encode
};
use sp_std::prelude::*;

use pallet_token as Token;


#[cfg(test)]
mod tests;

pub trait Trait: system::Trait + pallet_token::Trait   {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: ReservableCurrency<Self::AccountId>;
}

pub type EtherIssueIndex = u128;
pub type EtherRedemptionIndex = u128;

type AccountIdOf<T> = <T as system::Trait>::AccountId;
type BalanceOf<T> = <<T as pallet_token::Trait>::Currency as Currency<AccountIdOf<T>>>::Balance;
type EtherIssueInfoOf<T> = EtherIssueInfo<AccountIdOf<T>, BalanceOf<T>, <T as system::Trait>::BlockNumber>;
type EtherRedemptionInfoOf<T> = EtherRedemptionInfo<AccountIdOf<T>, BalanceOf<T>, <T as system::Trait>::BlockNumber>;

#[derive(Encode, Decode, Default, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct EtherIssueInfo<AccountId, Balance, BlockNumber> {
	vault: Vec<u8>, 
	sender: Vec<u8>, 
	amount: Balance, 
	issuer: AccountId,
	issued: BlockNumber,
	vault_status: bool,
	transaction: Vec<u8>,
	minted_status: bool,
}

#[derive(Encode, Decode, Default, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct EtherRedemptionInfo<AccountId, Balance, BlockNumber> {
	vault: Vec<u8>, 
	receiver: Vec<u8>, 
	amount: Balance, 
	redeemer: AccountId,
	redeemed: BlockNumber,
	vault_status: bool,
	transaction: Vec<u8>,
	burnt_status: bool,
}


decl_storage! {
	trait Store for Module<T: Trait> as NestedStructs {

		pub Watchers get(fn watchers): Vec<AccountIdOf<T>>;

		pub EtherIssue get(fn ether_issue): 
			map hasher(blake2_128_concat) EtherIssueIndex => EtherIssueInfoOf<T>;
		pub EtherIssueByUser get(fn ether_issue_by_user): 
			map hasher(blake2_128_concat) AccountIdOf<T> => EtherIssueIndex;			
		pub EtherIssueCount get(fn ether_issue_count): EtherIssueIndex;
		pub EtherUnmintedIssues get(fn ether_unminted_issues): Vec<EtherIssueIndex>;

		pub EtherRedemption get(fn ether_redemption): 
			map hasher(blake2_128_concat) EtherRedemptionIndex => EtherRedemptionInfoOf<T>;
		pub EtherRedemptionByUser get(fn ether_redemption_by_user): 
			map hasher(blake2_128_concat) AccountIdOf<T> => EtherRedemptionIndex;	
		pub EtherRedemptionCount get(fn ether_redemption_count): EtherRedemptionIndex;		
		pub EtherUnburntRedemptions get(fn ether_unburnt_redemptions): Vec<EtherRedemptionIndex>;		

	}
}

decl_event! (
	pub enum Event<T>
	where
		Balance = BalanceOf<T>,
		<T as system::Trait>::AccountId,
	{
		// TODO
		WatcherAdded(AccountId), // STILL TODO		
		// TODO
		WatcherRemoved(AccountId), // STILL TODO
		
		// Ether issue created \[Issuer ETH Wallet, Issuer DCB Wallet, ETH Amount, Blocknumber\]
		EtherIssueCreated(u32, Balance, AccountId), // STILL TODO
		// Ether issue cancelled \[Ether Issue ID\]
		EtherIssueCancelled(u32, Balance, AccountId), // STILL TODO
		// Ether issue created \[Issuer ETH Wallet, Issuer DCB Wallet, ETH Amount, Blocknumber\]
		EtherIssueExecuted(u32, Balance, AccountId), // STILL TODO

		// Ether issue created \[Issuer ETH Wallet, Issuer DCB Wallet, ETH Amount, Blocknumber\]
		EtherRedemptionCreated(u32, Balance, AccountId), // STILL TODO
		// Ether issue cancelled \[Ether Redemption ID\]
		EtherRedemptionCancelled(u32, Balance, AccountId), // STILL TODO
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Sample Error
		WhatTypeOfError,

		IssueNotFound,
		IssueAlreadyCreated,
		IssueAlreadyMinted,
		ExceedIssue,
		VaultHasReceivedEther,
		VaultHasNotReceivedEther,

		RedemptionNotFound,
		RedemptionAlreadyCreated,
		RedemptionAlreadyBurnt,
		ExceedRedemption,
		InsuffientEther,
		BelowMinimumEtherWithdrawal,
		VaultHasSentEther,
		VaultHasNotSentEther,		

		AlreadyWatcher,
		NotWatcher
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		type Error = Error<T>;

		#[weight = 10_000]
		fn add_watcher(
			origin,
			watcher: AccountIdOf<T>
		) -> DispatchResult {
            ensure_root(origin)?;

			let mut watchers = Watchers::<T>::get();

			match watchers.binary_search(&watcher) {

				Ok(_) => Err(Error::<T>::AlreadyWatcher.into()),
				Err(index) => {
					watchers.insert(index, watcher.clone());
					Watchers::<T>::put(watchers);
					Self::deposit_event(RawEvent::WatcherAdded(watcher));		
					Ok(())
				}
			}			
		}	
		
		#[weight = 10_000]
		fn remove_watcher(
			origin,
			watcher: AccountIdOf<T>
		) -> DispatchResult {
            ensure_root(origin)?;

			let mut watchers = Watchers::<T>::get();
			match watchers.binary_search(&watcher) {
				Ok(index) => {
					watchers.remove(index);
					Watchers::<T>::put(watchers);
					Self::deposit_event(RawEvent::WatcherRemoved(watcher));
					Ok(())
				},
				Err(_) => Err(Error::<T>::NotWatcher.into()),
			}			
		}		

		#[weight = 10_000]
		fn create_issue(
			origin, 
			eth_sender: Vec<u8>, 
			eth_vault: Vec<u8>, 
			eth_amount: BalanceOf<T>
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;
			let created = <system::Module<T>>::block_number();
			let salah: bool = false;
			let user_caller = _caller.clone();
			let caller_number_of_issues = <EtherIssueByUser<T>>::get(&_caller);
			let maximum_issues:u128 = 5;
			ensure!(caller_number_of_issues < maximum_issues, Error::<T>::ExceedIssue);

			let mut ether_unminted_issues = EtherUnmintedIssues::get();			
			let index = EtherIssueCount::get();
			let index2 = index.clone();
			let index3 = index.clone();
			let empty_vector: Vec<u8> = Vec::new();

			match ether_unminted_issues.binary_search(&index) {
				Ok(_) => {
					Err(Error::<T>::IssueAlreadyCreated.into())
				},
				Err(index) => {											
					ether_unminted_issues.insert(index, index2);
					<EtherUnmintedIssues>::put(ether_unminted_issues);
					<EtherIssue<T>>::insert(index2, EtherIssueInfo {
						vault: eth_vault,
						sender: eth_sender,
						amount: eth_amount,
						issuer: _caller,
						issued: created,
						vault_status: salah,
						transaction: empty_vector,
						minted_status: salah
		
					});
					EtherIssueCount::put(index3 + 1);	
					let new_number: u128 = caller_number_of_issues + 1;
					<EtherIssueByUser<T>>::insert(user_caller, new_number);
					Ok(())
				}
			}			
		}	

		#[weight = 10_000]
		fn update_issue(
			origin, 
			transaction: Vec<u8>,
			issue_id: EtherIssueIndex
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;
			let betul: bool = true;

			let watchers = Watchers::<T>::get();
			match watchers.binary_search(&_caller) {
				Ok(_index) => {

					let mut ether_unminted_issues = EtherUnmintedIssues::get();	
					
					match ether_unminted_issues.binary_search(&issue_id) {
						Ok(_index2) => {
							ether_unminted_issues.remove(_index2);
							EtherUnmintedIssues::put(ether_unminted_issues);							
							let ether_issue = <EtherIssue<T>>::get(issue_id);
							<EtherIssue<T>>::mutate(issue_id, |v| *v = EtherIssueInfo {
								vault: ether_issue.vault,
								sender: ether_issue.sender,
								amount: ether_issue.amount,
								issuer: ether_issue.issuer,
								issued: ether_issue.issued,
								vault_status: betul,
								transaction: transaction,
								minted_status: ether_issue.minted_status
							});					
							Ok(())
						},
						Err(_) => {										
							Err(Error::<T>::IssueNotFound.into())	
						}
					}						
				},
				Err(_) => Err(Error::<T>::NotWatcher.into()),
			}		
		}		
		
		#[weight = 10_000]
		fn execute_issue(
			origin,
			issue_id: EtherIssueIndex
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;	
			let ether_issue = <EtherIssue<T>>::get(issue_id);
			let issuer = ether_issue.issuer.clone();
			let amount = ether_issue.amount.clone();
			ensure!(ether_issue.vault_status == true, Error::<T>::VaultHasNotReceivedEther);
			ensure!(ether_issue.minted_status == false, Error::<T>::IssueAlreadyMinted);
			let betul: bool = true;
			EtherIssue::<T>::mutate(issue_id, |v| *v = EtherIssueInfo {
				vault: ether_issue.vault,
				sender: ether_issue.sender,
				amount: ether_issue.amount,
				issuer: ether_issue.issuer,
				issued: ether_issue.issued,
				vault_status: ether_issue.vault_status,
				transaction: ether_issue.transaction,
				minted_status: betul
			});		
			Self::mint_(5, issuer, amount);			
			Ok(())
			

		}		
		
		#[weight = 10_000]
		fn cancel_issue(
			origin,
			issue_id: EtherIssueIndex
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;
			let issue = <EtherIssue<T>>::get(issue_id);
			ensure!(issue.vault_status == false, Error::<T>::VaultHasReceivedEther);
			ensure!(issue.minted_status == false, Error::<T>::IssueAlreadyMinted);

			let mut ether_unminted_issues = EtherUnmintedIssues::get();

			match ether_unminted_issues.binary_search(&issue_id) {
				Ok(index) => {
					ether_unminted_issues.remove(index);
					EtherUnmintedIssues::put(ether_unminted_issues);
					EtherIssueByUser::<T>::mutate(_caller, |v| *v -= 1);
					Ok(())
				},
				Err(_) => Err(Error::<T>::IssueNotFound.into()),
			}	
		}	
		
		#[weight = 10_000]
		fn create_redeem(origin,
			eth_receiver: Vec<u8>, 
			eth_vault: Vec<u8>, 
			eth_amount: BalanceOf<T>
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;
			let _caller2 = _caller.clone();
			let created = <system::Module<T>>::block_number();
			let betul: bool = true;
			let salah: bool = false;
			let user_caller = _caller.clone();
			let caller_number_of_redemptions = <EtherRedemptionByUser<T>>::get(&_caller);
			let mut ether_unburnt_redemptions = EtherUnburntRedemptions::get();			
			let index = EtherRedemptionCount::get();
			let index2 = index.clone();
			let index3 = index.clone();
			let empty_vector: Vec<u8> = Vec::new();
			let caller_eth_balance = Self::get_balance(5, _caller2);
			//let minimum_eth_withdrawal = Self::u64_to_balance_option(10000000000000) ; // minimum 10 ETH for withdrawal

			ensure!(caller_eth_balance > eth_amount, Error::<T>::InsuffientEther);
			//ensure!(eth_amount > minimum_eth_withdrawal.into(), Error::<T>::BelowMinimumEtherWithdrawal);

			match ether_unburnt_redemptions.binary_search(&index) {
				Ok(_) => {
					Err(Error::<T>::RedemptionAlreadyCreated.into())
				},
				Err(index) => {	
					let caller_cloned = _caller.clone();
					let eth_amount_cloned = eth_amount.clone();		
					Self::burn_(5, caller_cloned, eth_amount_cloned);										
					ether_unburnt_redemptions.insert(index, index2);
					<EtherUnburntRedemptions>::put(ether_unburnt_redemptions);
					<EtherRedemption<T>>::insert(index2, EtherRedemptionInfo {
						vault: eth_vault,
						receiver: eth_receiver,
						amount: eth_amount,
						redeemer: _caller,
						redeemed: created,
						vault_status: salah,
						transaction: empty_vector,
						burnt_status: betul
					});
					EtherRedemptionCount::put(index3 + 1);	
					let new_number: u128 = caller_number_of_redemptions + 1;
					<EtherRedemptionByUser<T>>::insert(user_caller, new_number);
					Ok(())
				}
			}
		}	

		#[weight = 10_000]
		fn update_redeem(
			origin,
			transaction: Vec<u8>,
			redemption_id: EtherRedemptionIndex
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;
			let betul: bool = true;

			let watchers = Watchers::<T>::get();

			match watchers.binary_search(&_caller) {
				Ok(_index) => {

					let mut ether_unburnt_redemptions = EtherUnburntRedemptions::get();	
					
					match ether_unburnt_redemptions.binary_search(&redemption_id) {
						Ok(_index2) => {
							ether_unburnt_redemptions.remove(_index2);
							EtherUnburntRedemptions::put(ether_unburnt_redemptions);							
							let ether_redemption = <EtherRedemption<T>>::get(redemption_id);
							<EtherRedemption<T>>::mutate(redemption_id, |v| *v = EtherRedemptionInfo {
								vault: ether_redemption.vault,
								receiver: ether_redemption.receiver,
								amount: ether_redemption.amount,
								redeemer: ether_redemption.redeemer,
								redeemed: ether_redemption.redeemed,
								vault_status: betul,
								transaction: transaction,
								burnt_status: ether_redemption.burnt_status,
							});					
							Ok(())
						},
						Err(_) => {										
							Err(Error::<T>::RedemptionNotFound.into())	
						}
					}						
				},
				Err(_) => Err(Error::<T>::NotWatcher.into()),
			}		


		}	

		#[weight = 10_000]
		fn cancel_redeem(
			origin,
			redemption_id: EtherRedemptionIndex
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;
			let redemption = <EtherRedemption<T>>::get(redemption_id);
			ensure!(redemption.vault_status == false, Error::<T>::VaultHasSentEther);

			let mut ether_unburnt_redemptions = EtherUnburntRedemptions::get();

			match ether_unburnt_redemptions.binary_search(&redemption_id) {
				Ok(index) => {
					ether_unburnt_redemptions.remove(index);
					EtherUnburntRedemptions::put(ether_unburnt_redemptions);
					EtherRedemptionByUser::<T>::mutate(_caller, |v| *v -= 1);
					Ok(())
				},
				Err(_) => Err(Error::<T>::RedemptionNotFound.into()),
			}	
		}			
				
	}
}

impl<T: Trait> Module<T> {

/*
|	List of tokens with ID
|	004 - Bitcoin
|	005 - Ether
|	006 - Tether USD
|
*/

	pub fn get_balance(
		token: u32,
		account: AccountIdOf<T>, 
	) -> BalanceOf<T> {
		return <Token::Module<T>>::get_balance(token, account);
	}

	pub fn burn_(
		token: u32,
		burner: AccountIdOf<T>, 
		amount: BalanceOf<T>
	) -> () {
		<Token::Module<T>>::burn_(burner, token, amount);
	}

	pub fn mint_(
		token: u32,
		minter: AccountIdOf<T>, 
		amount: BalanceOf<T>
	) -> () {
		<Token::Module<T>>::mint_(minter, token, amount);
	}	
}