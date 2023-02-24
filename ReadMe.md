To compile

$cargo +nightly contract build --optimization-passes=0

The aim of this repo is to show how to do an http call and the attest it so it can be saved on chain

This is a Polakdot Decoded 2022 tutorial with a bit old code

Important source:

OFF LINE ATTESTATION
https://ethereum.org/en/decentralized-identity/#off-chain-attestations

# TODO FRONT END PHALA JS SDK SCAFFOLD WITH ALERTS

https://github.com/Phala-Network/js-sdk/tree/6c26c1aef0b6ea6eb85b5d75f1492f120233047c/packages/example

and

https://github.com/Phala-Network/js-sdk/blob/6c26c1aef0b6ea6eb85b5d75f1492f120233047c/packages/sdk/README.md

VIDEOS

https://www.youtube.com/watch?v=jJcHEFWSSEQ

https://www.youtube.com/watch?v=B7fUwRxelu4&t=1963s

Step 1

At constructor
let (generator, verifier) = attestation::create(b"gist-attestation-key");

we create the unique phat contract private key that generates generator and verifier
The generator will sign data and the verifier will enusre the data submitted is singed by the generator

In
pub fn get_attestation_generator(&self) -> attestation::Generator {
and
pub fn get_attestation_verifier(&self) -> attestation::Verifier {
we can see these keys

Step 2

Note: The aim is to create a public gist here https://gist.github.com/
and then ask Phat contract to http get the content extract the account mentioned, attest it and then store that on the chain

        fn parse_gist_url(url: &str) -> Result<GistUrl> {

parse the http link

Step 3

The above is used by

       #[ink(message)]
        pub fn attest(
            &self,
            url: String,
        ) -> core::result::Result<attestation::Attestation, Vec<u8>>

to ensure that this is a valid http address referrign to a gist

Step 4

        fn extract_claim(body: &[u8]) -> Result<AccountId>

Received the body of the http call and extracts the account details

Step 5

        fn decode_accountid_256(addr: &[u8]) -> Result<AccountId>

is used to decose a hex account to a proper AccountId address

Step 6

Attest function at

            let result = self.attestation_generator.sign(quote);

takes the quote that includes the username and AccountId and the attestation_generator signs it

Step 7

        pub fn redeem(&mut self, attestation: attestation::Attestation) -> Result<()>

Feeding the function with the attestation the attestation_verifier decomposes it and verifies this is valid signature

It then proceeds to store the data in the mapping

> NOTE: Although up to this point all works well, when I actually feed at https://phat-cb.phala.network/ or https://phat.phala.network/
> the function witht the attestation e.g.

Out of this

    {
    "Ok": {
        "data": "0x20456e746974793534f0f4360fc5dbb8cd7107edf24fc3f3c9ef3914b32585062bfd7aa84e02f8b84e",
        "signature": "0x7a4ac8075b8930527504b6a17eca0a8666e3544bba59a1d2cbbd7954ae08cb679c20d0149abef91eefd170eb35a92863b953c101eb0dde8522ca9f3fb88ce380"
    }
    }

I pass

    {
        "data": "0x20456e746974793534f0f4360fc5dbb8cd7107edf24fc3f3c9ef3914b32585062bfd7aa84e02f8b84e",
        "signature": "0x7a4ac8075b8930527504b6a17eca0a8666e3544bba59a1d2cbbd7954ae08cb679c20d0149abef91eefd170eb35a92863b953c101eb0dde8522ca9f3fb88ce380"
    }

to redeem function I get the error

<br>

![plot](./Printscreens/1.png)

<br>

Repo Paused as it achieved its target and mkving to more modern examples

> Note 1: A lot of functions were turned into queries so that we can actually see step by step what is going on

> Note 2: There are 2 steps. First http call brings the content which is attested. Then this attestation is submitted to the Phat contract so it is saved on Phala chain storage
