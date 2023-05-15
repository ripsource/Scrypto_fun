use scrypto::prelude::*;

#[blueprint]


mod barter {


    

    struct Barter {
        // User A vaults, keys and state
        a_nft_vaults: HashMap<ResourceAddress, Vault>,
        // a_token_vaults: HashMap<ResourceAddress, Vault>,
        a_vault_key: ResourceAddress,
        a_pending_time: bool,
        
        // User B vaults, keys and state
        b_nft_vaults: HashMap<ResourceAddress, Vault>,
        // b_token_vaults: HashMap<ResourceAddress, Vault>,
        b_vault_key: ResourceAddress,
        b_vault_key_hold: Vault,
        b_pending_time: bool,
     
       // final trade state
        a_has_accepted: bool,
        b_has_accepted: bool,
        // badge clean up burner
        badge_sweeper: Vault,
    }

    impl Barter {
     
        pub fn lets_barter(
            title_vault: String,) -> 
            (ComponentAddress, Bucket) {

            let badge_sweeper: Bucket = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .mint_initial_supply(1);
        
        // Create vault keys for each user
            let a_key = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", &title_vault)
            .metadata("symbol", "ESKRO")
            .metadata("description", "ESKRO Vault Key")
            .mintable(rule!(require(badge_sweeper.resource_address())), LOCKED)
            .burnable(rule!(require(badge_sweeper.resource_address())), LOCKED)
            .mint_initial_supply(1);
        
            let b_key = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", &title_vault)
            .metadata("description", &title_vault)
            .mintable(rule!(require(badge_sweeper.resource_address())), LOCKED)
            .burnable(rule!(require(badge_sweeper.resource_address())), LOCKED)
            .mint_initial_supply(1);

            let rules = AccessRulesConfig::new()
            // The third parameter here specifies the authority allowed to update the rule.
            .method(
                "deposit_vault_a",
                rule!(require(a_key.resource_address())),
                LOCKED,
            )
            .method(
                "deposit_vault_b",
                rule!(require(b_key.resource_address())),
                LOCKED,
            )
            .method(
                "withdraw_vault_a",
                rule!(require(a_key.resource_address())),
                LOCKED,
            )
            .method(
                "withdraw_vault_b",
                rule!(require(b_key.resource_address())),
                LOCKED,
            )
            .method(
                "take_all_vault_a",
                rule!(require(b_key.resource_address())),
                LOCKED,
            )
            .method(
                "take_all_vault_b",
                rule!(require(a_key.resource_address())),
                LOCKED,
            )
           
            // The second parameter here specifies the authority allowed to update the rule.
            .default(AccessRule::AllowAll, AccessRule::DenyAll);


            // Instantiate component
            let component = Self {
             

             
                a_vault_key: a_key.resource_address(),
                a_pending_time: false,
                a_nft_vaults: HashMap::new(),
              
                b_vault_key: b_key.resource_address(),
                b_vault_key_hold: Vault::with_bucket(b_key),
                b_pending_time: false,
                b_nft_vaults: HashMap::new(),
               // final trade state
                a_has_accepted: false,
                b_has_accepted: false,
                // badge clean up burner
                badge_sweeper: Vault::with_bucket(badge_sweeper),

            } 
                .instantiate();
                let component_address = component.globalize_with_access_rules(rules);
                

               (component_address, a_key)
        }


// Allow User B to claim a key to the B vault that they will control
// This means the user does not need to know what the account address of the other user is or send them the token
        pub fn claim_b_key(&mut self) -> Bucket {
            self.b_vault_key_hold.take(1)
        }


// Deposit tradeable assets in vaults AKA switch to pending trade/confirm assets for trade
        pub fn deposit_vault_a(&mut self, a_assets: Vec<Bucket>) {
            for bucket in a_assets.into_iter() {
                self.a_nft_vaults
                    .entry(bucket.resource_address())
                    .or_insert(Vault::new(bucket.resource_address()))
                    .put(bucket)
            }

            self.a_has_accepted = false;
            self.b_has_accepted = false;

        }

        pub fn deposit_vault_b(&mut self,  b_assets: Vec<Bucket>) {
      
            for bucket in b_assets.into_iter() {
                self.b_nft_vaults
                    .entry(bucket.resource_address())
                    .or_insert(Vault::new(bucket.resource_address()))
                    .put(bucket)
            }

            self.a_has_accepted = false;
            self.b_has_accepted = false;

        }

// Withdraw tradeable assets from vaults AKA Cancel trade
        pub fn withdraw_vault_a(&mut self) -> Vec<Bucket> {
           
            let a_assets: Vec<ResourceAddress> =
            self.a_nft_vaults.keys().cloned().collect();
            
            let mut buckets: Vec<Bucket> = vec![];

            for resource_address in a_assets.into_iter() {
                buckets.push(
                    self.a_nft_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }

            self.a_has_accepted = false;
            self.b_has_accepted = false;

            return buckets;

        }

        pub fn withdraw_vault_b(&mut self) -> Vec<Bucket> {

            let b_assets: Vec<ResourceAddress> =
            self.b_nft_vaults.keys().cloned().collect();
            
            let mut buckets: Vec<Bucket> = vec![];

            for resource_address in b_assets.into_iter() {
                buckets.push(
                    self.b_nft_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }

            self.a_has_accepted = false;
            self.b_has_accepted = false;

            return buckets;
    
        }


// in trade pending state : option to accept trade

        pub fn accept_trade(&mut self, auth: Proof) {
            let user_id = self.get_user_id(auth);

            assert!(!self.a_nft_vaults.is_empty() && !self.b_nft_vaults.is_empty(), "Vaults must contain assets before a trade can be accepted");

            if user_id == self.a_vault_key {
                assert!(!self.a_has_accepted, "You already accepted the offer!");
                self.a_has_accepted = true;
            } else {
                assert!(!self.b_has_accepted, "You already accepted the offer!");
                self.b_has_accepted = true;
            }

        }


// check proof of keys

        fn get_user_id(&self, badge: Proof) -> ResourceAddress {
            assert!(badge.amount() > Decimal::zero(), "Invalid user proof");
            
            let user_id = badge.resource_address();
            assert!(user_id == self.a_vault_key || user_id == self.b_vault_key, "Invalid user proof");

            user_id
        }



// after accepted

        pub fn take_all_vault_a(&mut self, b_key_return: Bucket) -> Vec<Bucket> {
            assert!(self.a_has_accepted, "Your trading partner hasn't accepted yet");
            assert!(self.b_has_accepted, "Your trading partner hasn't accepted yet");

            let a_assets: Vec<ResourceAddress> =
            self.a_nft_vaults.keys().cloned().collect();
            
          

            let mut buckets: Vec<Bucket> = vec![];

            for resource_address in a_assets.into_iter() {
                buckets.push(
                    self.a_nft_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }


            b_key_return.burn();
            return buckets;

        }

        pub fn take_all_vault_b(&mut self, a_key_return: Bucket ) -> Vec<Bucket> {
            assert!(self.a_has_accepted, "Your trading partner hasn't accepted yet");
            assert!(self.b_has_accepted, "Your trading partner hasn't accepted yet");

            let b_assets: Vec<ResourceAddress> =
            self.b_nft_vaults.keys().cloned().collect();
            
            let mut buckets: Vec<Bucket> = vec![];

            for resource_address in b_assets.into_iter() {
                buckets.push(
                    self.b_nft_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }
            a_key_return.burn();
            return buckets;

        }

       
    }
}