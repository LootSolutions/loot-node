use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_nft_class() {
    new_test_ext().execute_with(|| {
        assert_ok!(LootNft::create_nft_class(Origin::signed(100), vec![0], (), 10, 10));
        assert_eq!(NFT::next_class_id(), 1);
        assert_ok!(LootNft::create_nft_class(Origin::signed(101), vec![0], (), 10, 10));
        assert_eq!(NFT::next_class_id(), 2);
    });
}

#[test]
fn mint_nft_token() {
    new_test_ext().execute_with(|| {
        // Can mint token
        assert_ok!(LootNft::create_nft_class(Origin::signed(100), vec![0], (), 10, 10));
        assert_ok!(LootNft::mint_nft_token(Origin::signed(200), 0, vec![0], ()));

        // Can't mint token that is too expensive
        assert_ok!(LootNft::create_nft_class(Origin::signed(100), vec![0], (), 1000, 10));
        assert_noop!(LootNft::mint_nft_token(Origin::signed(200), 1, vec![0], ()), Error::<Test>::NotEnoughFunds);
    });
}

#[test]
fn transfer_token() {
    new_test_ext().execute_with(|| {
        assert_ok!(LootNft::create_nft_class(Origin::signed(100), vec![0], (), 10, 10));
        assert_ok!(LootNft::mint_nft_token(Origin::signed(200), 0, vec![0], ()));

        assert_eq!(NFT::tokens(0, 0).unwrap().owner, 200);

        // Can transfer NFT that they own
        assert_ok!(LootNft::nft_transfer(Origin::signed(200), 100, 0, 0));
        assert_eq!(NFT::tokens(0, 0).unwrap().owner, 100);

        // Can't transfer NFT they do not own
        assert_noop!(LootNft::nft_transfer(Origin::signed(200), 101, 0, 0), orml_nft::Error::<Test>::NoPermission);
        assert_eq!(NFT::tokens(0, 0).unwrap().owner, 100);

        // Can't transfer NFT that does not exist
        assert_noop!(LootNft::nft_transfer(Origin::signed(200), 101, 0, 1), orml_nft::Error::<Test>::TokenNotFound);
        assert_eq!(NFT::tokens(0, 1), None);

        // Can't transfer NFT of class that does not exist
        assert_noop!(LootNft::nft_transfer(Origin::signed(200), 101, 1, 0), orml_nft::Error::<Test>::TokenNotFound);
        assert_eq!(NFT::tokens(1, 0), None);
    });
}

#[test]
fn set_info() {
    new_test_ext().execute_with(|| {
        // Info set properly on class creation
        assert_ok!(LootNft::create_nft_class(Origin::signed(100), vec![0], (), 10, 10));
        assert_eq!(LootNft::info(0), Some((true, 10, 10)));

        // Can set mintable
        assert_ok!(LootNft::set_mintable(Origin::signed(100), 0, false));
        assert_eq!(LootNft::info(0), Some((false, 10, 10)));

        // Can set price
        assert_ok!(LootNft::set_price(Origin::signed(100), 0, 20));
        assert_eq!(LootNft::info(0), Some((false, 20, 10)));

        // Can set royalty
        assert_ok!(LootNft::set_royalty(Origin::signed(100), 0, 20));
        assert_eq!(LootNft::info(0), Some((false, 20, 20)));

        // Non class owner can't set mintable
        assert_noop!(LootNft::set_mintable(Origin::signed(200), 0, true), Error::<Test>::InvalidPermission);
        assert_eq!(LootNft::info(0), Some((false, 20, 20)));

        // Non class owner can't set price
        assert_noop!(LootNft::set_price(Origin::signed(200), 0, 10), Error::<Test>::InvalidPermission);
        assert_eq!(LootNft::info(0), Some((false, 20, 20)));        

        // Non class owner can't set royalty
        assert_noop!(LootNft::set_royalty(Origin::signed(200), 0, 10), Error::<Test>::InvalidPermission);
        assert_eq!(LootNft::info(0), Some((false, 20, 20)));

        // Can't set royalty passed 100
        assert_noop!(LootNft::set_royalty(Origin::signed(100), 0, 101), Error::<Test>::InvalidRoyalty);
        assert_eq!(LootNft::info(0), Some((false, 20, 20)));
    });
}

#[test]
fn buy_nft() {
    new_test_ext().execute_with(|| {
        let seller_balance = Balances::free_balance(&100);
        let buyer_balance = Balances::free_balance(&200);
        let class_creator_balance = Balances::free_balance(&300);
        let minting_price = 10;
        let sales_price = 20;
        let royalty_amount = 2; // 10% of price

        assert_ok!(LootNft::create_nft_class(Origin::signed(300), vec![0], (), minting_price, 10));

        assert_ok!(LootNft::mint_nft_token(Origin::signed(100), 0, vec![0], ()));
        assert_ok!(LootNft::create_sale(Origin::signed(100), 0, 0, sales_price));

        // Can buy
        assert_ok!(LootNft::buy(Origin::signed(200), 0, 0));

        // Balances transfered
        assert_eq!(Balances::free_balance(200), buyer_balance - sales_price);
        assert_eq!(Balances::free_balance(100), seller_balance - minting_price + sales_price - royalty_amount);
        assert_eq!(Balances::free_balance(300), class_creator_balance + minting_price + royalty_amount);

        // Token transfered to buyer
        assert_eq!(NFT::tokens(0, 0).unwrap().owner, 200);

        // Token no longer for sale
        assert_noop!(LootNft::buy(Origin::signed(100), 0, 0), Error::<Test>::TokenNotForSale);

        // Can't buy token not for sale
        assert_ok!(LootNft::mint_nft_token(Origin::signed(100), 0, vec![0], ()));
        assert_noop!(LootNft::buy(Origin::signed(200), 0, 1), Error::<Test>::TokenNotForSale);

        // Can't buy your own token
        assert_ok!(LootNft::create_sale(Origin::signed(100), 0, 1, sales_price));
        assert_noop!(LootNft::buy(Origin::signed(100), 0, 1), Error::<Test>::BuyerSellerSame);
    });
}

#[test]
fn delete_sale() {
    new_test_ext().execute_with(|| {
        assert_ok!(LootNft::create_nft_class(Origin::signed(100), vec![0], (), 10, 10));

        assert_ok!(LootNft::mint_nft_token(Origin::signed(100), 0, vec![0], ()));
        assert_ok!(LootNft::create_sale(Origin::signed(100), 0, 0, 20));

        // Can't remove sale on token you don't own
        assert_noop!(LootNft::delete_sale(Origin::signed(200), 0, 0), Error::<Test>::TokenNotOwned);

        // Can remove sale
        assert_ok!(LootNft::delete_sale(Origin::signed(100), 0, 0));

        // Can't buy after sale removed
        assert_noop!(LootNft::buy(Origin::signed(200), 0, 0), Error::<Test>::TokenNotForSale);

        // Removing sale that doesn't exist is noop
        assert_noop!(LootNft::delete_sale(Origin::signed(100), 0, 0), Error::<Test>::TokenNotForSale);
    });
}