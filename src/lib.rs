//#![deny(missing_docs)]
//#![deny(warnings)]

//! This library is usefull for developing C/C++ AWS Nitro Enclave applications 
//! with custom functionality like enclave-to-enclave 
//! secure communication and mutual attestation.
//! 
//! 

use webpki;
use aws_nitro_enclaves_cose as aws_cose;

use serde::Deserialize;
use serde_bytes::ByteBuf;
//use serde_cbor::Error as CborError;
//use serde_cbor::Value as CborValue;
//use serde_repr::{Deserialize_repr, Serialize_repr};
//use std::collections::BTreeMap;

//use chrono::serde::ts_seconds;
use chrono::serde::ts_milliseconds;

use chrono::{DateTime, Utc};

use std::collections::HashMap;
//use serde_cbor::from_slice;


#[derive(Debug, Deserialize)]
struct NitroAdDocPayload {
    module_id: String,
    digest: String,

    #[serde(with = "ts_milliseconds")]
    timestamp: DateTime<Utc>,

    pcrs: HashMap<u8, ByteBuf>,
    certificate: ByteBuf,
    cabundle: Vec<ByteBuf>,

    // optional
    public_key: Option<ByteBuf>,

    // optional
    user_data: Option<ByteBuf>,

    // optional 
    nonce: Option<ByteBuf>
}


#[cfg(test)]
mod tests {
    
    use super::*;
    use openssl::pkey::PKey;
    use openssl::pkey::{Private, Public};
    use openssl::ec::*;
    
    static ALL_SIGALGS: &[&webpki::SignatureAlgorithm] = &[
        &webpki::ECDSA_P256_SHA256,
        &webpki::ECDSA_P256_SHA384,
        &webpki::ECDSA_P384_SHA256,
        &webpki::ECDSA_P384_SHA384,
        &webpki::ED25519,
        #[cfg(feature = "alloc")]
        &webpki::RSA_PKCS1_2048_8192_SHA256,
        #[cfg(feature = "alloc")]
        &webpki::RSA_PKCS1_2048_8192_SHA384,
        #[cfg(feature = "alloc")]
        &webpki::RSA_PKCS1_2048_8192_SHA512,
        #[cfg(feature = "alloc")]
        &webpki::RSA_PKCS1_3072_8192_SHA384,
    ];

    // Public domain work: Pride and Prejudice by Jane Austen, taken from https://www.gutenberg.org/files/1342/1342.txt
    const TEXT: &[u8] = b"It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife.";

    #[test]
    fn aws_nitro_ad_validation_flow() {

        let ad_bytes = include_bytes!("../tests/data/nitro_ad_debug.bin");
        let ad_doc = aws_cose::COSESign1::from_bytes(ad_bytes);

        let ad_doc = ad_doc.unwrap();

        // for validation flow details see here:
        // https://github.com/aws/aws-nitro-enclaves-nsm-api/blob/main/docs/attestation_process.md 

        // !! no Signature checks for now - to do signature validation, specify pub key
        let ad_payload = ad_doc.get_payload(None).unwrap();

        let ad_parsed: NitroAdDocPayload = serde_cbor::from_slice(&ad_payload).unwrap();
        println!("{:?}", ad_parsed);

        // 

    }

    #[test]
    fn cose_sign1_ec384_validate() {
        let (_, ec_public) = get_ec384_test_key();

        // This output was validated against COSE-C implementation
        let cose_doc = aws_cose::COSESign1::from_bytes(&[
            0x84, /* Protected: {1: -35} */
            0x44, 0xA1, 0x01, 0x38, 0x22, /* Unprotected: {4: '11'} */
            0xA1, 0x04, 0x42, 0x31, 0x31, /* payload: */
            0x58, 0x75, 0x49, 0x74, 0x20, 0x69, 0x73, 0x20, 0x61, 0x20, 0x74, 0x72, 0x75, 0x74,
            0x68, 0x20, 0x75, 0x6E, 0x69, 0x76, 0x65, 0x72, 0x73, 0x61, 0x6C, 0x6C, 0x79, 0x20,
            0x61, 0x63, 0x6B, 0x6E, 0x6F, 0x77, 0x6C, 0x65, 0x64, 0x67, 0x65, 0x64, 0x2C, 0x20,
            0x74, 0x68, 0x61, 0x74, 0x20, 0x61, 0x20, 0x73, 0x69, 0x6E, 0x67, 0x6C, 0x65, 0x20,
            0x6D, 0x61, 0x6E, 0x20, 0x69, 0x6E, 0x20, 0x70, 0x6F, 0x73, 0x73, 0x65, 0x73, 0x73,
            0x69, 0x6F, 0x6E, 0x20, 0x6F, 0x66, 0x20, 0x61, 0x20, 0x67, 0x6F, 0x6F, 0x64, 0x20,
            0x66, 0x6F, 0x72, 0x74, 0x75, 0x6E, 0x65, 0x2C, 0x20, 0x6D, 0x75, 0x73, 0x74, 0x20,
            0x62, 0x65, 0x20, 0x69, 0x6E, 0x20, 0x77, 0x61, 0x6E, 0x74, 0x20, 0x6F, 0x66, 0x20,
            0x61, 0x20, 0x77, 0x69, 0x66, 0x65, 0x2E, /* signature - length 48 x 2 */
            0x58, 0x60, /* R: */
            0xCD, 0x42, 0xD2, 0x76, 0x32, 0xD5, 0x41, 0x4E, 0x4B, 0x54, 0x5C, 0x95, 0xFD, 0xE6,
            0xE3, 0x50, 0x5B, 0x93, 0x58, 0x0F, 0x4B, 0x77, 0x31, 0xD1, 0x4A, 0x86, 0x52, 0x31,
            0x75, 0x26, 0x6C, 0xDE, 0xB2, 0x4A, 0xFF, 0x2D, 0xE3, 0x36, 0x4E, 0x9C, 0xEE, 0xE9,
            0xF9, 0xF7, 0x95, 0xA0, 0x15, 0x15, /* S: */
            0x5B, 0xC7, 0x12, 0xAA, 0x28, 0x63, 0xE2, 0xAA, 0xF6, 0x07, 0x8A, 0x81, 0x90, 0x93,
            0xFD, 0xFC, 0x70, 0x59, 0xA3, 0xF1, 0x46, 0x7F, 0x64, 0xEC, 0x7E, 0x22, 0x1F, 0xD1,
            0x63, 0xD8, 0x0B, 0x3B, 0x55, 0x26, 0x25, 0xCF, 0x37, 0x9D, 0x1C, 0xBB, 0x9E, 0x51,
            0x38, 0xCC, 0xD0, 0x7A, 0x19, 0x31,
        ])
        .unwrap();

        assert_eq!(cose_doc.get_payload(Some(&ec_public)).unwrap(), TEXT);
    }

    #[test]
    fn aws_root_cert_used_as_end_entity_cert() {
        let ee: &[u8] = include_bytes!("../tests/data/aws_root.der");
        let ca = include_bytes!("../tests/data/aws_root.der");
    
        let anchors = vec![webpki::trust_anchor_util::cert_der_as_trust_anchor(ca).unwrap()];
        let anchors = webpki::TLSServerTrustAnchors(&anchors);
    
       //#[allow(clippy::unreadable_literal)] // TODO: Make this clear.
        let time = webpki::Time::from_seconds_since_unix_epoch(1616094379); // 18 March 2021
    
        let cert = webpki::EndEntityCert::from(ee).unwrap();
        assert_eq!(
            //Ok(()),
            Err(webpki::Error::CAUsedAsEndEntity),
            cert.verify_is_valid_tls_server_cert(ALL_SIGALGS, &anchors, &[], time)
        );
    }

    /// Static SECP384R1/P-384 key to be used when cross-validating the implementation
    fn get_ec384_test_key() -> (EcKey<Private>, EcKey<Public>) {
        let alg = openssl::ec::EcGroup::from_curve_name(openssl::nid::Nid::SECP384R1).unwrap();
        let x = openssl::bn::BigNum::from_hex_str(
            "5a829f62f2f4f095c0e922719285b4b981c677912870a413137a5d7319916fa8\
                584a6036951d06ffeae99ca73ab1a2dc",
        )
        .unwrap();
        let y = openssl::bn::BigNum::from_hex_str(
            "e1b76e08cb20d6afcea7423f8b49ec841dde6f210a6174750bf8136a31549422\
                4df153184557a6c29a1d7994804f604c",
        )
        .unwrap();
        let d = openssl::bn::BigNum::from_hex_str(
            "55c6aa815a31741bc37f0ffddea73af2397bad640816ef22bfb689efc1b6cc68\
                2a73f7e5a657248e3abad500e46d5afc",
        )
        .unwrap();
        let ec_public =
            openssl::ec::EcKey::from_public_key_affine_coordinates(&alg, &x, &y).unwrap();
        let ec_private =
            openssl::ec::EcKey::from_private_components(&alg, &d, &ec_public.public_key()).unwrap();
        (
            //PKey::from_ec_key(ec_private).unwrap(),
            //PKey::from_ec_key(ec_public).unwrap(),
            ec_private, 
            ec_public
        )
    }
}