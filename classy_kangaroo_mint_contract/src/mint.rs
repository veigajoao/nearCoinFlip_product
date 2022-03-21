use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: AccountId,
        //we add an optional parameter for perpetual royalties
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) {
        assert! {
            self.activate == U128(0),
            "Contract not activated!"
        }

        let currentNFTs = u128::from(self.currentNFTs);
        let maxNFTs = u128::from(self.maxNFTs);

        assert!(
            currentNFTs < maxNFTs,
            "All {} / {} NFTs have been minted",
            currentNFTs,
            maxNFTs
        );

        //measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        // create a royalty map to store in the token
        let mut royalty = HashMap::new();

        //iterate through the perpetual royalties and insert the account and amount in the royalty map
        let account: AccountId = "classykangaroos1.near".parse().unwrap();
            royalty.insert(account, 5);

        //specify the token struct that contains the owner ID 
        let token = Token {
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            //we set the approved account IDs to the default value (an empty map)
            approved_account_ids: Default::default(),
            //the next approval ID is set to 0
            next_approval_id: 0,
            //the map of perpetual royalties for the token (The owner will get 100% - total perpetual royalties)
            royalty,
        };

        let god: AccountId = "gaius1337.near".parse().unwrap();
        let dev: AccountId = "obedear.near".parse().unwrap();
        let dev2: AccountId = "dleer.near".parse().unwrap();

        if u128::from(self.whitelist) == 1 {
            assert!(self.whitelist_list.contains(&token.owner_id), "You are not whitelisted {}", &token.owner_id);

            if &token.owner_id == &god || &token.owner_id ==  &dev || &token.owner_id ==  &dev2 {

            } else {
                let from: Option<U128> = Some(U128(0));
                let limit: Option<u64> = Some(5);
                let tokens = self.nft_tokens_for_owner_internal(&token.owner_id, from, limit);
                assert!(tokens.len() < 1 , "Whitelist mode, you cannot mint more than one! {}", &token.owner_id);
            }
        }

        let token_id_u = u128::from(self.currentNFTs) + 1;
        let token_id = token_id_u.to_string();

        let metadatas = self.tokens_to_mint.get(&token_id);

        if let Some(metadata) = metadatas {

            //insert the token ID and token struct and make sure that the token doesn't exist
            assert!(
                self.tokens_by_id.insert(&token_id, &token).is_none(),
                "Token already exists {}",
                &token_id
            );
            let img: &Option<String> = &metadata.media;
            let img2: &String = &img.as_ref().unwrap();
            //insert the token ID and token struct and make sure that the token doesn't exist
            assert!(
                self.tokens_by_img_url.insert(img2, &token).is_none(),
                "Cheeky"
            );

            let required_cost = 10000000000000000000000000;
            let attached_deposit = env::attached_deposit();
            assert!(
                required_cost <= attached_deposit,
                "Must attach {} yoctoNEAR to cover costs",
                required_cost,
            );

            let owner: AccountId = "classykangaroos1.near".parse().unwrap();

            Promise::new(owner).transfer(required_cost);

            //insert the token ID and metadata
            self.token_metadata_by_id.insert(&token_id, &metadata);

            //call the internal method for adding the token to the owner
            self.internal_add_token_to_owner(&token.owner_id, &token_id);

            self.currentNFTs = U128(u128::from(self.currentNFTs) + 1);

            // Construct the mint log as per the events standard.
            let nft_mint_log: EventLog = EventLog {
                // Standard name ("nep171").
                standard: NFT_STANDARD_NAME.to_string(),
                // Version of the standard ("nft-1.0.0").
                version: NFT_METADATA_SPEC.to_string(),
                // The data related with the event stored in a vector.
                event: EventLogVariant::NftMint(vec![NftMintLog {
                    // Owner of the token.
                    owner_id: token.owner_id.to_string(),
                    // Vector of token IDs that were minted.
                    token_ids: vec![token_id.to_string()],
                    // An optional memo to include.
                    memo: None,
                }]),
         };

            // Log the serialized json.
            env::log_str(&nft_mint_log.to_string());

            //calculate the required storage which was the used - initial
            let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

            //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
        // refund_deposit(required_storage_in_bytes);
    } else {
        assert!(true == false, "No metadata");
    }
    }

    #[payable]
    pub fn whitelist_off(
        &mut self,
        owner_id: AccountId,
    ) {
        let owner: AccountId = "classykangaroos1.near".parse().unwrap();
        assert!(env::signer_account_id() == owner, "You are not the Owner");
        self.whitelist = U128(0);
    }

    #[payable]
    pub fn activate_minting(
        &mut self,
        owner_id: AccountId,
    ) {
        let owner: AccountId = "classykangaroos1.near".parse().unwrap();
        assert!(env::signer_account_id() == owner, "You are not the Owner");
        self.activate = U128(0);
    }

    #[payable]
    pub fn metadata_upload(
        &mut self,
        token_id: String,
        metadata: TokenMetadata,
    ) {
        let owner: AccountId = "classykangaroos1.near".parse().unwrap();
        assert!(env::signer_account_id() == owner, "You are not the Owner");
        self.tokens_to_mint.insert(&token_id, &metadata);
    }

    #[payable]
    pub fn royalty_update(
        &mut self,
        token_id: TokenId
    ) {
        let owner: AccountId = "classykangaroos1.near".parse().unwrap();
        assert!(env::signer_account_id() == owner, "You are not the Owner");
        let mut token = self.tokens_by_id.get(&token_id).expect("No token");
        let account: AccountId = "classykangarooroyalty.near".parse().unwrap();
        // create a royalty map to store in the token
        let mut royalty = HashMap::new();
        royalty.insert(account, 500);
        token.royalty = royalty;

        self.tokens_by_id.insert(&token_id, &token);
    }


    #[payable]
    pub fn whitelist_upload(
        &mut self,
        account_id: AccountId,
    ) {
        let owner: AccountId = "classykangaroos1.near".parse().unwrap();
        assert!(env::signer_account_id() == owner, "You are not the Owner");
        self.whitelist_list.push(account_id);
    }
}