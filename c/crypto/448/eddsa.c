/*
 * Copyright 2017-2024 The OpenSSL Project Authors. All Rights Reserved.
 * Copyright 2015-2016 Cryptography Research, Inc.
 *
 * Licensed under the Apache License 2.0 (the "License").  You may not use
 * this file except in compliance with the License.  You can obtain a copy
 * in the file LICENSE in the source distribution or at
 * https://www.openssl.org/source/license.html
 *
 * Originally written by Mike Hamburg
 */
#include <stdint.h>
#include <string.h>
// #include <openssl/crypto.h>
// #include <openssl/evp.h>
// #include "crypto/ecx.h"
#include <stdio.h>

#include "curve448_local.h"
#include "ed448.h"
// #include "internal/numbers.h"
#include "../sha3.h"
#include "word.h"

#define COFACTOR 4

static c448_error_t oneshot_hash(OSSL_LIB_CTX *ctx, uint8_t *out, size_t outlen,
				 const uint8_t *in, size_t inlen,
				 const char *propq) {
	// Assuming SHAKE256 corresponds to 256-bit output
	const unsigned bitSize = 256;
	const enum SHA3_FLAGS flags = SHA3_FLAGS_KECCAK;

	// Use your sha3_HashBuffer_sq function
	sha3_return_t ret =
	    sha3_HashBuffer_sq(bitSize, flags, in, inlen, out, outlen);

	// Check the return value and map it to C448 error codes
	if (ret != SHA3_RETURN_OK) {
		return C448_FAILURE;
	}
	return C448_SUCCESS;
}

static void clamp(uint8_t secret_scalar_ser[EDDSA_448_PRIVATE_BYTES]) {
	secret_scalar_ser[0] &= -COFACTOR;
	secret_scalar_ser[EDDSA_448_PRIVATE_BYTES - 1] = 0;
	secret_scalar_ser[EDDSA_448_PRIVATE_BYTES - 2] |= 0x80;
}

static c448_error_t hash_init_with_dom(OSSL_LIB_CTX *ctx, sha3_context *hashctx,
				       uint8_t prehashed, uint8_t for_prehash,
				       const uint8_t *context,
				       size_t context_len, const char *propq) {
	/* ASCII: "SigEd448", in hex for EBCDIC compatibility */
	const char dom_s[] = "\x53\x69\x67\x45\x64\x34\x34\x38";
	uint8_t dom[2];

	if (context_len > UINT8_MAX) {
		return C448_FAILURE;
	}

	/* Compute domain separator values */
	dom[0] = (uint8_t)(2 - (prehashed == 0 ? 1 : 0) -
			   (for_prehash == 0 ? 1 : 0));
	dom[1] = (uint8_t)context_len;

	/* Initialize SHA3 context with SHAKE256 configuration */
	sha3_Init(hashctx, 256);  // SHAKE256 uses 256-bit initialization

	/* Feed domain-specific data into the hash */
	sha3_Update(hashctx, dom_s, sizeof(dom_s) - 1);	 // "SigEd448" prefix
	sha3_Update(hashctx, dom, sizeof(dom));	     // Domain separator values
	sha3_Update(hashctx, context, context_len);  // Context string

	return C448_SUCCESS;
}

/*

static c448_error_t hash_init_with_dom(OSSL_LIB_CTX *ctx, EVP_MD_CTX *hashctx,
				       uint8_t prehashed, uint8_t for_prehash,
				       const uint8_t *context,
				       size_t context_len, const char *propq) {
	const char dom_s[] = "\x53\x69\x67\x45\x64\x34\x34\x38";
	uint8_t dom[2];
	EVP_MD *shake256 = NULL;

	if (context_len > UINT8_MAX) return C448_FAILURE;

	dom[0] = (uint8_t)(2 - (prehashed == 0 ? 1 : 0) -
			   (for_prehash == 0 ? 1 : 0));
	dom[1] = (uint8_t)context_len;

	shake256 = EVP_MD_fetch(ctx, "SHAKE256", propq);
	if (shake256 == NULL) return C448_FAILURE;

	if (!EVP_DigestInit_ex(hashctx, shake256, NULL) ||
	    !EVP_DigestUpdate(hashctx, dom_s, sizeof(dom_s) - 1) ||
	    !EVP_DigestUpdate(hashctx, dom, sizeof(dom)) ||
	    !EVP_DigestUpdate(hashctx, context, context_len)) {
		EVP_MD_free(shake256);
		return C448_FAILURE;
	}

	EVP_MD_free(shake256);
	return C448_SUCCESS;
}
*/

/* In this file because it uses the hash */
c448_error_t ossl_c448_ed448_convert_private_key_to_x448(
    OSSL_LIB_CTX *ctx, uint8_t x[X448_PRIVATE_BYTES],
    const uint8_t ed[EDDSA_448_PRIVATE_BYTES], const char *propq) {
	/* pass the private key through oneshot_hash function */
	/* and keep the first X448_PRIVATE_BYTES bytes */
	return oneshot_hash(ctx, x, X448_PRIVATE_BYTES, ed,
			    EDDSA_448_PRIVATE_BYTES, propq);
}

c448_error_t ossl_c448_ed448_derive_public_key(
    OSSL_LIB_CTX *ctx, uint8_t pubkey[EDDSA_448_PUBLIC_BYTES],
    const uint8_t privkey[EDDSA_448_PRIVATE_BYTES], const char *propq) {
	/* only this much used for keygen */
	uint8_t secret_scalar_ser[EDDSA_448_PRIVATE_BYTES];
	curve448_scalar_t secret_scalar;
	unsigned int c;
	curve448_point_t p;

	if (!oneshot_hash(ctx, secret_scalar_ser, sizeof(secret_scalar_ser),
			  privkey, EDDSA_448_PRIVATE_BYTES, propq))
		return C448_FAILURE;

	clamp(secret_scalar_ser);

	ossl_curve448_scalar_decode_long(secret_scalar, secret_scalar_ser,
					 sizeof(secret_scalar_ser));

	/*
	 * Since we are going to mul_by_cofactor during encoding, divide by it
	 * here. However, the EdDSA base point is not the same as the decaf base
	 * point if the sigma isogeny is in use: the EdDSA base point is on
	 * Etwist_d/(1-d) and the decaf base point is on Etwist_d, and when
	 * converted it effectively picks up a factor of 2 from the isogenies.
	 * So we might start at 2 instead of 1.
	 */
	for (c = 1; c < C448_EDDSA_ENCODE_RATIO; c <<= 1)
		ossl_curve448_scalar_halve(secret_scalar, secret_scalar);

	ossl_curve448_precomputed_scalarmul(p, ossl_curve448_precomputed_base,
					    secret_scalar);

	ossl_curve448_point_mul_by_ratio_and_encode_like_eddsa(pubkey, p);

	/* Cleanup */
	ossl_curve448_scalar_destroy(secret_scalar);
	ossl_curve448_point_destroy(p);
	OPENSSL_cleanse(secret_scalar_ser, sizeof(secret_scalar_ser));

	return C448_SUCCESS;
}

c448_error_t ossl_c448_ed448_sign(
    OSSL_LIB_CTX *ctx, uint8_t signature[EDDSA_448_SIGNATURE_BYTES],
    const uint8_t privkey[EDDSA_448_PRIVATE_BYTES],
    const uint8_t pubkey[EDDSA_448_PUBLIC_BYTES], const uint8_t *message,
    size_t message_len, uint8_t prehashed, const uint8_t *context,
    size_t context_len, const char *propq) {
	curve448_scalar_t secret_scalar;
	c448_error_t ret = C448_FAILURE;
	curve448_scalar_t nonce_scalar;
	uint8_t nonce_point[EDDSA_448_PUBLIC_BYTES] = {0};
	unsigned int c;
	curve448_scalar_t challenge_scalar;

	// Expand private key and derive the secret scalar
	{
		uint8_t expanded[EDDSA_448_PRIVATE_BYTES * 2];
		sha3_context ctx_hash;

		// Hash the private key to expand it
		sha3_Init(&ctx_hash, 2 * EDDSA_448_PRIVATE_BYTES * 8);
		sha3_Update(&ctx_hash, privkey, EDDSA_448_PRIVATE_BYTES);
		memcpy(expanded, sha3_Finalize(&ctx_hash), sizeof(expanded));

		clamp(expanded);
		ossl_curve448_scalar_decode_long(secret_scalar, expanded,
						 EDDSA_448_PRIVATE_BYTES);

		// Hash to initialize the nonce
		sha3_Init(&ctx_hash, 2 * EDDSA_448_PRIVATE_BYTES * 8);
		sha3_Update(&ctx_hash, expanded + EDDSA_448_PRIVATE_BYTES,
			    EDDSA_448_PRIVATE_BYTES);
		sha3_Update(&ctx_hash, message, message_len);
		memcpy(expanded, sha3_Finalize(&ctx_hash), sizeof(expanded));

		OPENSSL_cleanse(expanded, sizeof(expanded));
	}

	// Compute the nonce
	{
		uint8_t nonce[2 * EDDSA_448_PRIVATE_BYTES];
		sha3_context ctx_hash;

		sha3_Init(&ctx_hash, 2 * EDDSA_448_PRIVATE_BYTES * 8);
		sha3_Update(&ctx_hash, privkey, EDDSA_448_PRIVATE_BYTES);
		memcpy(nonce, sha3_Finalize(&ctx_hash), sizeof(nonce));

		ossl_curve448_scalar_decode_long(nonce_scalar, nonce,
						 sizeof(nonce));
		OPENSSL_cleanse(nonce, sizeof(nonce));
	}

	// Encode the nonce point
	{
		curve448_scalar_t nonce_scalar_2;
		curve448_point_t p;

		ossl_curve448_scalar_halve(nonce_scalar_2, nonce_scalar);
		for (c = 2; c < C448_EDDSA_ENCODE_RATIO; c <<= 1)
			ossl_curve448_scalar_halve(nonce_scalar_2,
						   nonce_scalar_2);

		ossl_curve448_precomputed_scalarmul(
		    p, ossl_curve448_precomputed_base, nonce_scalar_2);
		ossl_curve448_point_mul_by_ratio_and_encode_like_eddsa(
		    nonce_point, p);
		ossl_curve448_point_destroy(p);
		ossl_curve448_scalar_destroy(nonce_scalar_2);
	}

	// Compute the challenge
	{
		uint8_t challenge[2 * EDDSA_448_PRIVATE_BYTES];
		sha3_context ctx_hash;

		sha3_Init(&ctx_hash, 2 * EDDSA_448_PRIVATE_BYTES * 8);
		sha3_Update(&ctx_hash, nonce_point, sizeof(nonce_point));
		sha3_Update(&ctx_hash, pubkey, EDDSA_448_PUBLIC_BYTES);
		sha3_Update(&ctx_hash, message, message_len);
		memcpy(challenge, sha3_Finalize(&ctx_hash), sizeof(challenge));

		ossl_curve448_scalar_decode_long(challenge_scalar, challenge,
						 sizeof(challenge));
		OPENSSL_cleanse(challenge, sizeof(challenge));
	}

	// Compute the final signature
	ossl_curve448_scalar_mul(challenge_scalar, challenge_scalar,
				 secret_scalar);
	ossl_curve448_scalar_add(challenge_scalar, challenge_scalar,
				 nonce_scalar);

	OPENSSL_cleanse(signature, EDDSA_448_SIGNATURE_BYTES);
	memcpy(signature, nonce_point, sizeof(nonce_point));
	ossl_curve448_scalar_encode(&signature[EDDSA_448_PUBLIC_BYTES],
				    challenge_scalar);

	// Clean up and return
	ossl_curve448_scalar_destroy(secret_scalar);
	ossl_curve448_scalar_destroy(nonce_scalar);
	ossl_curve448_scalar_destroy(challenge_scalar);

	ret = C448_SUCCESS;
err:
	return ret;
}

/*
c448_error_t ossl_c448_ed448_sign(
    OSSL_LIB_CTX *ctx, uint8_t signature[EDDSA_448_SIGNATURE_BYTES],
    const uint8_t privkey[EDDSA_448_PRIVATE_BYTES],
    const uint8_t pubkey[EDDSA_448_PUBLIC_BYTES], const uint8_t *message,
    size_t message_len, uint8_t prehashed, const uint8_t *context,
    size_t context_len, const char *propq) {
	curve448_scalar_t secret_scalar;
	c448_error_t ret = C448_FAILURE;
	curve448_scalar_t nonce_scalar;
	uint8_t nonce_point[EDDSA_448_PUBLIC_BYTES] = {0};
	unsigned int c;
	curve448_scalar_t challenge_scalar;

	// Expand private key and derive the secret scalar
	{
		uint8_t expanded[EDDSA_448_PRIVATE_BYTES * 2];
		sha3_context ctx_hash;

		// Hash the private key to expand it
		sha3_Init(&ctx_hash, 2 * EDDSA_448_PRIVATE_BYTES * 8);
		sha3_Update(&ctx_hash, privkey, EDDSA_448_PRIVATE_BYTES);
		sha3_Finalize(expanded, &ctx_hash);

		clamp(expanded);
		ossl_curve448_scalar_decode_long(secret_scalar, expanded,
						 EDDSA_448_PRIVATE_BYTES);

		// Hash to initialize the nonce
		sha3_Init(&ctx_hash, 2 * EDDSA_448_PRIVATE_BYTES * 8);
		sha3_Update(&ctx_hash, expanded + EDDSA_448_PRIVATE_BYTES,
			    EDDSA_448_PRIVATE_BYTES);
		sha3_Update(&ctx_hash, message, message_len);
		sha3_Finalize(expanded, &ctx_hash);

		OPENSSL_cleanse(expanded, sizeof(expanded));
	}

	// Compute the nonce
	{
		uint8_t nonce[2 * EDDSA_448_PRIVATE_BYTES];
		sha3_context ctx_hash;

		sha3_Init(&ctx_hash, 2 * EDDSA_448_PRIVATE_BYTES * 8);
		sha3_Update(&ctx_hash, privkey, EDDSA_448_PRIVATE_BYTES);
		sha3_Finalize(nonce, &ctx_hash);

		ossl_curve448_scalar_decode_long(nonce_scalar, nonce,
						 sizeof(nonce));
		OPENSSL_cleanse(nonce, sizeof(nonce));
	}

	// Encode the nonce point
	{
		curve448_scalar_t nonce_scalar_2;
		curve448_point_t p;

		ossl_curve448_scalar_halve(nonce_scalar_2, nonce_scalar);
		for (c = 2; c < C448_EDDSA_ENCODE_RATIO; c <<= 1)
			ossl_curve448_scalar_halve(nonce_scalar_2,
						   nonce_scalar_2);

		ossl_curve448_precomputed_scalarmul(
		    p, ossl_curve448_precomputed_base, nonce_scalar_2);
		ossl_curve448_point_mul_by_ratio_and_encode_like_eddsa(
		    nonce_point, p);
		ossl_curve448_point_destroy(p);
		ossl_curve448_scalar_destroy(nonce_scalar_2);
	}

	// Compute the challenge
	{
		uint8_t challenge[2 * EDDSA_448_PRIVATE_BYTES];
		sha3_context ctx_hash;

		sha3_Init(&ctx_hash, 2 * EDDSA_448_PRIVATE_BYTES * 8);
		sha3_Update(&ctx_hash, nonce_point, sizeof(nonce_point));
		sha3_Update(&ctx_hash, pubkey, EDDSA_448_PUBLIC_BYTES);
		sha3_Update(&ctx_hash, message, message_len);
		sha3_Finalize(challenge, &ctx_hash);

		ossl_curve448_scalar_decode_long(challenge_scalar, challenge,
						 sizeof(challenge));
		OPENSSL_cleanse(challenge, sizeof(challenge));
	}

	// Compute the final signature
	ossl_curve448_scalar_mul(challenge_scalar, challenge_scalar,
				 secret_scalar);
	ossl_curve448_scalar_add(challenge_scalar, challenge_scalar,
				 nonce_scalar);

	OPENSSL_cleanse(signature, EDDSA_448_SIGNATURE_BYTES);
	memcpy(signature, nonce_point, sizeof(nonce_point));
	ossl_curve448_scalar_encode(&signature[EDDSA_448_PUBLIC_BYTES],
				    challenge_scalar);

	// Clean up and return
	ossl_curve448_scalar_destroy(secret_scalar);
	ossl_curve448_scalar_destroy(nonce_scalar);
	ossl_curve448_scalar_destroy(challenge_scalar);

	ret = C448_SUCCESS;
err:
	return ret;
}
*/

/*

c448_error_t ossl_c448_ed448_sign(
    OSSL_LIB_CTX *ctx, uint8_t signature[EDDSA_448_SIGNATURE_BYTES],
    const uint8_t privkey[EDDSA_448_PRIVATE_BYTES],
    const uint8_t pubkey[EDDSA_448_PUBLIC_BYTES], const uint8_t *message,
    size_t message_len, uint8_t prehashed, const uint8_t *context,
    size_t context_len, const char *propq) {
	curve448_scalar_t secret_scalar;
	EVP_MD_CTX *hashctx = EVP_MD_CTX_new();
	c448_error_t ret = C448_FAILURE;
	curve448_scalar_t nonce_scalar;
	uint8_t nonce_point[EDDSA_448_PUBLIC_BYTES] = {0};
	unsigned int c;
	curve448_scalar_t challenge_scalar;

	if (hashctx == NULL) return C448_FAILURE;

	{
uint8_t expanded[EDDSA_448_PRIVATE_BYTES * 2];

if (!oneshot_hash(ctx, expanded, sizeof(expanded), privkey,
		  EDDSA_448_PRIVATE_BYTES, propq))
	goto err;
clamp(expanded);
ossl_curve448_scalar_decode_long(secret_scalar, expanded,
				 EDDSA_448_PRIVATE_BYTES);

if (!hash_init_with_dom(ctx, hashctx, prehashed, 0, context, context_len,
			propq) ||
    !EVP_DigestUpdate(hashctx, expanded + EDDSA_448_PRIVATE_BYTES,
		      EDDSA_448_PRIVATE_BYTES) ||
    !EVP_DigestUpdate(hashctx, message, message_len)) {
	OPENSSL_cleanse(expanded, sizeof(expanded));
	goto err;
}
OPENSSL_cleanse(expanded, sizeof(expanded));
}

{
	uint8_t nonce[2 * EDDSA_448_PRIVATE_BYTES];

	if (!EVP_DigestFinalXOF(hashctx, nonce, sizeof(nonce))) goto err;
	ossl_curve448_scalar_decode_long(nonce_scalar, nonce, sizeof(nonce));
	OPENSSL_cleanse(nonce, sizeof(nonce));
}

{
	curve448_scalar_t nonce_scalar_2;
	curve448_point_t p;

	ossl_curve448_scalar_halve(nonce_scalar_2, nonce_scalar);
	for (c = 2; c < C448_EDDSA_ENCODE_RATIO; c <<= 1)
		ossl_curve448_scalar_halve(nonce_scalar_2, nonce_scalar_2);

	ossl_curve448_precomputed_scalarmul(p, ossl_curve448_precomputed_base,
					    nonce_scalar_2);
	ossl_curve448_point_mul_by_ratio_and_encode_like_eddsa(nonce_point, p);
	ossl_curve448_point_destroy(p);
	ossl_curve448_scalar_destroy(nonce_scalar_2);
}

{
	uint8_t challenge[2 * EDDSA_448_PRIVATE_BYTES];

	if (!hash_init_with_dom(ctx, hashctx, prehashed, 0, context,
				context_len, propq) ||
	    !EVP_DigestUpdate(hashctx, nonce_point, sizeof(nonce_point)) ||
	    !EVP_DigestUpdate(hashctx, pubkey, EDDSA_448_PUBLIC_BYTES) ||
	    !EVP_DigestUpdate(hashctx, message, message_len) ||
	    !EVP_DigestFinalXOF(hashctx, challenge, sizeof(challenge)))
		goto err;

	ossl_curve448_scalar_decode_long(challenge_scalar, challenge,
					 sizeof(challenge));
	OPENSSL_cleanse(challenge, sizeof(challenge));
}

ossl_curve448_scalar_mul(challenge_scalar, challenge_scalar, secret_scalar);
ossl_curve448_scalar_add(challenge_scalar, challenge_scalar, nonce_scalar);

OPENSSL_cleanse(signature, EDDSA_448_SIGNATURE_BYTES);
memcpy(signature, nonce_point, sizeof(nonce_point));
ossl_curve448_scalar_encode(&signature[EDDSA_448_PUBLIC_BYTES],
			    challenge_scalar);

ossl_curve448_scalar_destroy(secret_scalar);
ossl_curve448_scalar_destroy(nonce_scalar);
ossl_curve448_scalar_destroy(challenge_scalar);

ret = C448_SUCCESS;
err : EVP_MD_CTX_free(hashctx);
return ret;
}

*/

c448_error_t ossl_c448_ed448_sign_prehash(
    OSSL_LIB_CTX *ctx, uint8_t signature[EDDSA_448_SIGNATURE_BYTES],
    const uint8_t privkey[EDDSA_448_PRIVATE_BYTES],
    const uint8_t pubkey[EDDSA_448_PUBLIC_BYTES], const uint8_t hash[64],
    const uint8_t *context, size_t context_len, const char *propq) {
	return ossl_c448_ed448_sign(ctx, signature, privkey, pubkey, hash, 64,
				    1, context, context_len, propq);
}

static c448_error_t c448_ed448_pubkey_verify(const uint8_t *pub,
					     size_t pub_len) {
	curve448_point_t pk_point;

	if (pub_len != EDDSA_448_PUBLIC_BYTES) return C448_FAILURE;

	return ossl_curve448_point_decode_like_eddsa_and_mul_by_ratio(pk_point,
								      pub);
}

c448_error_t ossl_c448_ed448_verify(
    OSSL_LIB_CTX *ctx, const uint8_t signature[EDDSA_448_SIGNATURE_BYTES],
    const uint8_t pubkey[EDDSA_448_PUBLIC_BYTES], const uint8_t *message,
    size_t message_len, uint8_t prehashed, const uint8_t *context,
    uint8_t context_len, const char *propq) {
	curve448_point_t pk_point, r_point;
	c448_error_t error;
	curve448_scalar_t challenge_scalar;
	curve448_scalar_t response_scalar;
	/* Order in little endian format */
	static const uint8_t order[] = {
	    0xF3, 0x44, 0x58, 0xAB, 0x92, 0xC2, 0x78, 0x23, 0x55, 0x8F,
	    0xC5, 0x8D, 0x72, 0xC2, 0x6C, 0x21, 0x90, 0x36, 0xD6, 0xAE,
	    0x49, 0xDB, 0x4E, 0xC4, 0xE9, 0x23, 0xCA, 0x7C, 0xFF, 0xFF,
	    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
	    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
	    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x3F, 0x00};
	int i;

	/*
	 * Check that s (second 57 bytes of the sig) is less than the order.
	 * Both s and the order are in little-endian format. This can be done in
	 * variable time, since if this is not the case the signature if
	 * publicly invalid.
	 */
	for (i = EDDSA_448_PUBLIC_BYTES - 1; i >= 0; i--) {
		if (signature[i + EDDSA_448_PUBLIC_BYTES] > order[i])
			return C448_FAILURE;
		if (signature[i + EDDSA_448_PUBLIC_BYTES] < order[i]) break;
	}
	if (i < 0) return C448_FAILURE;

	error = ossl_curve448_point_decode_like_eddsa_and_mul_by_ratio(pk_point,
								       pubkey);

	if (C448_SUCCESS != error) return error;

	error = ossl_curve448_point_decode_like_eddsa_and_mul_by_ratio(
	    r_point, signature);
	if (C448_SUCCESS != error) return error;

	{
		/* Compute the challenge */
		sha3_context hashctx;  // Stack allocation for the SHA3 context
		uint8_t challenge
		    [2 * EDDSA_448_PRIVATE_BYTES];  // Allocate space for the
						    // challenge on the stack

		/* Initialize SHA3 context */
		sha3_Init(&hashctx, 256);  // SHAKE256 initialization

		/* Feed domain-specific data into the hash */
		sha3_Update(&hashctx, signature, EDDSA_448_PUBLIC_BYTES);
		sha3_Update(&hashctx, pubkey, EDDSA_448_PUBLIC_BYTES);
		sha3_Update(&hashctx, message, message_len);

		/* Finalize the hash */
		const uint8_t *result = (const uint8_t *)sha3_Finalize(
		    &hashctx);	// Finalize and get the hash

		if (result == NULL) {
			return C448_FAILURE;
		}

		/* Copy the hash result to the challenge buffer */
		memcpy(challenge, result, sizeof(challenge));

		/* Decode the challenge into a scalar */
		ossl_curve448_scalar_decode_long(challenge_scalar, challenge,
						 sizeof(challenge));

		/* Clean sensitive data */
		OPENSSL_cleanse(challenge, sizeof(challenge));
	}
	ossl_curve448_scalar_sub(challenge_scalar, ossl_curve448_scalar_zero,
				 challenge_scalar);

	ossl_curve448_scalar_decode_long(response_scalar,
					 &signature[EDDSA_448_PUBLIC_BYTES],
					 EDDSA_448_PRIVATE_BYTES);

	/* pk_point = -c(x(P)) + (cx + k)G = kG */
	ossl_curve448_base_double_scalarmul_non_secret(
	    pk_point, response_scalar, pk_point, challenge_scalar);
	return c448_succeed_if(ossl_curve448_point_eq(pk_point, r_point));
}

c448_error_t ossl_c448_ed448_verify_prehash(
    OSSL_LIB_CTX *ctx, const uint8_t signature[EDDSA_448_SIGNATURE_BYTES],
    const uint8_t pubkey[EDDSA_448_PUBLIC_BYTES], const uint8_t hash[64],
    const uint8_t *context, uint8_t context_len, const char *propq) {
	return ossl_c448_ed448_verify(ctx, signature, pubkey, hash, 64, 1,
				      context, context_len, propq);
}

int ossl_ed448_sign(OSSL_LIB_CTX *ctx, uint8_t *out_sig, const uint8_t *message,
		    size_t message_len, const uint8_t public_key[57],
		    const uint8_t private_key[57], const uint8_t *context,
		    size_t context_len, const uint8_t phflag,
		    const char *propq) {
	return ossl_c448_ed448_sign(ctx, out_sig, private_key, public_key,
				    message, message_len, phflag, context,
				    context_len, propq) == C448_SUCCESS;
}

/*
 * This function should not be necessary since ossl_ed448_verify() already
 * does this check internally.
 * For some reason the FIPS ACVP requires a EDDSA KeyVer test.
 */
int ossl_ed448_pubkey_verify(const uint8_t *pub, size_t pub_len) {
	return c448_ed448_pubkey_verify(pub, pub_len);
}

int ossl_ed448_verify(OSSL_LIB_CTX *ctx, const uint8_t *message,
		      size_t message_len, const uint8_t signature[114],
		      const uint8_t public_key[57], const uint8_t *context,
		      size_t context_len, const uint8_t phflag,
		      const char *propq) {
	return ossl_c448_ed448_verify(
		   ctx, signature, public_key, message, message_len, phflag,
		   context, (uint8_t)context_len, propq) == C448_SUCCESS;
}

int ossl_ed448_public_from_private(OSSL_LIB_CTX *ctx,
				   uint8_t out_public_key[57],
				   const uint8_t private_key[57],
				   const char *propq) {
	return ossl_c448_ed448_derive_public_key(
		   ctx, out_public_key, private_key, propq) == C448_SUCCESS;
}
