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
