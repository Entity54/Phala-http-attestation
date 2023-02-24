#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pink_extension as pink;

#[pink::contract(env=PinkEnvironment)]
mod phala_http_attestation_gist {
    // use core::fmt::Error;
    use super::pink;
    use ink_prelude::{
        format,
        string::{String, ToString},
        vec::Vec,
    };
    use pink::{http_get, PinkEnvironment};
    use scale::{Decode, Encode};

    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;
    use pink_utils::attestation;

    // use crate::alloc::string::ToString; //used at   account[1..last_elem_num].to_string()

    use serde::Deserialize;
    // you have to use crates with `no_std` support in contract.
    use serde_json_core;

    const CLAIM_PREFIX: &str = "This gist is owned by address: 0x";
    const ADDRESS_LEN: usize = 64;

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidEthAddress,
        InvalidPrefixEthAddress,
        InvalidLengthEthAddress,
        HttpRequestFailed,
        InvalidResponseBody,
        BadOrigin,
        BadgeContractNotSetUp,
        InvalidUrl,
        RequestFailed,
        NoClaimFound,
        InvalidAddressLength,
        InvalidAddress,
        NoPermission,
        InvalidSignature,
        UsernameAlreadyInUse,
        AccountAlreadyInUse,
        FailedToIssueBadge,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PhalaHttpAttestationGist {
        admin: AccountId,
        attestation_verifier: attestation::Verifier,
        attestation_generator: attestation::Generator,
        // linked_users: Mapping<String, ()>,
        linked_users: Mapping<String, AccountId>,
    }

    #[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
    pub struct EtherscanResponse<'a> {
        status: &'a str,
        message: &'a str,
        result: &'a str,
    }

    #[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct GistUrl {
        username: String,
        gist_id: String,
        filename: String,
    }

    #[derive(Clone, Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct GistQuote {
        username: String,
        account_id: AccountId,
    }

    impl PhalaHttpAttestationGist {
        #[ink(constructor)]
        pub fn new() -> Self {
            // Create the attestation helpers
            let (generator, verifier) = attestation::create(b"gist-attestation-key");
            // Save sender as the contract admin
            let admin = Self::env().caller();

            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.admin = admin;
                contract.attestation_generator = generator;
                contract.attestation_verifier = verifier;
                contract.linked_users = Default::default();
            })
        }

        /// The attestation generator
        #[ink(message)]
        pub fn get_attestation_generator(&self) -> attestation::Generator {
            self.attestation_generator.clone()
        }

        /// The attestation verifier
        #[ink(message)]
        pub fn get_attestation_verifier(&self) -> attestation::Verifier {
            self.attestation_verifier.clone()
        }

        #[ink(message)]
        pub fn get_admin(&self) -> AccountId {
            self.admin.clone()
        }

        /// Redeems a POAP with a signed `attestation`. (callable)
        /// The attestation must be created by [`attest_gist`] function. After the verification of
        /// the attestation, the the sender account will the linked to a Github username. Then a
        /// POAP redemption code will be allocated to the sender.
        /// Each blockchain account and github account can only be linked once.
        #[ink(message)]
        pub fn redeem(&mut self, attestation: attestation::Attestation) -> Result<()> {
            // Verify the attestation
            let data: GistQuote = self
                .attestation_verifier
                .verify_as(&attestation)
                .ok_or(Error::InvalidSignature)?;

            // The caller must be the attested account
            if data.account_id != self.env().caller() {
                pink::warn!("No permission.");
                return Err(Error::NoPermission);
            }
            // The github username can only link to one account
            if self.linked_users.contains(&data.username) {
                pink::warn!("Username alreay in use.");
                return Err(Error::UsernameAlreadyInUse);
            }
            // self.linked_users.insert(&data.username, &());
            self.linked_users.insert(&data.username, &data.account_id);
            Ok(())
        }

        /// Attests a Github Gist by the raw file url. (Query only)
        ///
        /// It sends a HTTPS request to the url and extract an address from the claim ("This gist
        /// is owned by address: 0x..."). Once the claim is verified, it returns a signed
        /// attestation with the data `(username, account_id)`.
        ///
        /// The `Err` variant of the result is an encoded `Error` to simplify cross-contract calls.
        /// Particularly, when another contract wants to call us, they may not want to depend on
        /// any special type defined by us (`Error` in this case). So we only return generic types.
        /// Example: https://gist.githubusercontent.com/Entity54/5476bcec0e9263266ad15ec3f9561411/raw/c51066450c5223878b6edfc2d4a3e15d835cd5e9/BE_003.txt
        ///
        #[ink(message)]
        pub fn attest(
            &self,
            url: String,
        ) -> core::result::Result<attestation::Attestation, Vec<u8>> {
            // pub fn attest(&self, url: String) -> core::result::Result<Vec<u8>, Vec<u8>> {

            // Verify the URL
            let gist_url = Self::parse_gist_url(&url).map_err(|e| e.encode())?;

            // Ok(String::from("All Good here"))

            // Fetch the gist content
            let resposne = http_get!(url);
            if resposne.status_code != 200 {
                return Err(Error::RequestFailed.encode());
            }
            let body = resposne.body;
            // Ok(body) //"Ok": "This gist is owned by address: 0xf0f4360fc5dbb8cd7107edf24fc3f3c9ef3914b32585062bfd7aa84e02f8b84e"

            // Verify the claim and extract the account id
            let account_id = Self::extract_claim(&body).map_err(|e| e.encode())?;
            let quote = GistQuote {
                username: gist_url.username,
                account_id,
            };
            let result = self.attestation_generator.sign(quote);
            Ok(result)
        }

        /// Parses a Github Gist url.
        ///
        /// - Returns a parsed [GistUrl] struct if the input is a valid url;
        /// - Otherwise returns an [Error].
        fn parse_gist_url(url: &str) -> Result<GistUrl> {
            let path = url
                .strip_prefix("https://gist.githubusercontent.com/")
                .ok_or(Error::InvalidUrl)?;
            let components: Vec<_> = path.split('/').collect();
            if components.len() < 5 {
                return Err(Error::InvalidUrl);
            }
            Ok(GistUrl {
                username: components[0].to_string(),
                gist_id: components[1].to_string(),
                filename: components[4].to_string(),
            })
        }

        /// Extracts the ownerhip of the gist from a claim in the gist body.
        ///
        /// A valid claim must have the statement "This gist is owned by address: 0x..." in `body`. The
        /// address must be the 256 bits public key of the Substrate account in hex.
        ///
        /// - Returns a 256-bit `AccountId` representing the owner account if the claim is valid;
        /// - otherwise returns an [Error].
        fn extract_claim(body: &[u8]) -> Result<AccountId> {
            let body = String::from_utf8_lossy(body);
            let pos = body.find(CLAIM_PREFIX).ok_or(Error::NoClaimFound)?;
            let addr: String = body
                .chars()
                .skip(pos)
                .skip(CLAIM_PREFIX.len())
                .take(ADDRESS_LEN)
                .collect();
            let addr = addr.as_bytes();
            let account_id = Self::decode_accountid_256(addr)?;
            Ok(account_id)
        }

        /// Decodes a hex string as an 256-bit AccountId32
        fn decode_accountid_256(addr: &[u8]) -> Result<AccountId> {
            use hex::FromHex;
            if addr.len() != ADDRESS_LEN {
                return Err(Error::InvalidAddressLength);
            }
            let bytes = <[u8; 32]>::from_hex(addr).or(Err(Error::InvalidAddress))?;
            Ok(AccountId::from(bytes))
        }

        /// Parses a Github Gist url.
        ///
        /// - Returns a parsed [GistUrl] struct if the input is a valid url;
        /// - Otherwise returns an [Error].
        /// Note: At https://github.com/Phala-Network/oracle-workshop this is just a function. Here we want to see
        /// Try:    https://gist.githubusercontent.com/Entity54/5476bcec0e9263266ad15ec3f9561411/raw/c51066450c5223878b6edfc2d4a3e15d835cd5e9/BE_003.txt
        #[ink(message)]
        pub fn ang_parse_gist_url(&self, url: String) -> Result<GistUrl> {
            let path = url
                .strip_prefix("https://gist.githubusercontent.com/")
                .ok_or(Error::InvalidUrl)?;
            let components: Vec<_> = path.split('/').collect();
            if components.len() < 5 {
                return Err(Error::InvalidUrl);
            }
            Ok(GistUrl {
                username: components[0].to_string(),
                gist_id: components[1].to_string(),
                filename: components[4].to_string(),
            })
        }
        // "Ok": {
        //     "username": "Entity54",
        //     "gistId": "5476bcec0e9263266ad15ec3f9561411",
        //     "filename": "BE_003.txt"
        //   }

        /// Extracts the ownerhip of the gist from a claim in the gist body.
        ///
        /// A valid claim must have the statement "This gist is owned by address: 0x..." in `body`. The
        /// address must be the 256 bits public key of the Substrate account in hex.
        ///
        /// - Returns a 256-bit `AccountId` representing the owner account if the claim is valid;
        /// - otherwise returns an [Error].
        // fn extract_claim(body: &[u8]) -> Result<AccountId> {
        // Provide as body
        // This gist is owned by address: 0xf0f4360fc5dbb8cd7107edf24fc3f3c9ef3914b32585062bfd7aa84e02f8b84e
        // Note: Instead of &[u8] I used Vec<u8>
        // In String::from_utf8_lossy(&body) used &body so it becomes &[u8] instead Vec<u8>
        //For account Mac2 5HWdttFeYE89GQDGNRYspsJouxZ56xwm6bzKxSPtbDjwpQbb with Hex 0xf0f4360fc5dbb8cd7107edf24fc3f3c9ef3914b32585062bfd7aa84e02f8b84e
        //we get "Ok": "464inykovjdRPhMhW2zbJ47iA8qYSmPWqKLkaEgH2xc6SQ4c"
        // pub fn extract_claim(&self, body: Vec<u8>) -> Result<Vec<u8>> {
        #[ink(message)]
        pub fn ang_extract_claim(&self, body: Vec<u8>) -> Result<AccountId> {
            let body = String::from_utf8_lossy(&body);
            let pos = body.find(CLAIM_PREFIX).ok_or(Error::NoClaimFound)?;
            let addr: String = body
                .chars()
                .skip(pos)
                .skip(CLAIM_PREFIX.len())
                .take(ADDRESS_LEN)
                .collect();
            let addr = addr.as_bytes();
            let account_id = Self::decode_accountid_256(addr)?;
            Ok(account_id)
            // Ok(addr.to_vec())
        }

        /// Decodes a hex string as an 256-bit AccountId32
        // If you pass f0f4360fc5dbb8cd7107edf24fc3f3c9ef3914b32585062bfd7aa84e02f8b84e you get "Ok": "464inykovjdRPhMhW2zbJ47iA8qYSmPWqKLkaEgH2xc6SQ4c"
        //For account Mac2 5HWdttFeYE89GQDGNRYspsJouxZ56xwm6bzKxSPtbDjwpQbb with Hex 0xf0f4360fc5dbb8cd7107edf24fc3f3c9ef3914b32585062bfd7aa84e02f8b84e
        #[ink(message)]
        pub fn ang_decode_accountid_256(&self, addr: Vec<u8>) -> Result<AccountId> {
            use hex::FromHex;
            if addr.len() != ADDRESS_LEN {
                return Err(Error::InvalidAddressLength);
            }
            let bytes = <[u8; 32]>::from_hex(addr).or(Err(Error::InvalidAddress))?;
            Ok(AccountId::from(bytes))
        }

        /// A function to handle direct off-chain Query from users.
        /// Such functions use the immutable reference `&self`
        /// so WILL NOT change the contract state.
        #[ink(message)]
        pub fn get_eth_balance(&self, _account: String) -> Result<String> {
            //Example _account "0xD0fE316B9f01A3b5fd6790F88C2D53739F80B464"
            // if !account.starts_with("0x") && account.len() != 42 {
            //     return Err(Error::InvalidEthAddress);
            // }

            //remove quotes
            let last_elem_num = _account.len() - 1;
            // let account = _account[1..last_elem_num].to_string();
            let account = String::from(&_account[1..last_elem_num]);

            if !account.starts_with("0x") {
                return Err(Error::InvalidPrefixEthAddress);
            }

            if (account.len() as u8) != 42 {
                return Err(Error::InvalidLengthEthAddress);
            }

            // get account ETH balance with HTTP requests to Etherscan
            // you can send any HTTP requests in Query handler
            let resp = http_get!(format!(
                "https://api.etherscan.io/api?module=account&action=balance&address={}",
                account
            ));
            if resp.status_code != 200 {
                return Err(Error::HttpRequestFailed);
            }

            let result: EtherscanResponse = serde_json_core::from_slice(&resp.body)
                .or(Err(Error::InvalidResponseBody))?
                .0;
            Ok(String::from(result.result))
        }

        #[ink(message)]
        pub fn get_positive_message(&self) -> String {
            let response = http_get!("https://example.com/");
            String::from("All Good")
        }
    }
}
