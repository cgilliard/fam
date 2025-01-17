/*
 * Copyright 2017-2023 The OpenSSL Project Authors. All Rights Reserved.
 * Copyright 2015-2016 Cryptography Research, Inc.
 *
 * Licensed under the Apache License 2.0 (the "License").  You may not use
 * this file except in compliance with the License.  You can obtain a copy
 * in the file LICENSE in the source distribution or at
 * https://www.openssl.org/source/license.html
 *
 * Originally written by Mike Hamburg
 */
#include "field.h"

static const gf MODULUS = {
    FIELD_LITERAL(0xffffffffffffffULL, 0xffffffffffffffULL, 0xffffffffffffffULL,
		  0xffffffffffffffULL, 0xfffffffffffffeULL, 0xffffffffffffffULL,
		  0xffffffffffffffULL, 0xffffffffffffffULL)};

/* Serialize to wire format. */
void gf_serialize(uint8_t serial[SER_BYTES], const gf x, int with_hibit) {
	unsigned int j = 0, fill = 0;
	dword_t buffer = 0;
	int i;
	gf red;

	gf_copy(red, x);
	gf_strong_reduce(red);
	if (!with_hibit) assert(gf_hibit(red) == 0);

	for (i = 0; i < (with_hibit ? X_SER_BYTES : SER_BYTES); i++) {
		if (fill < 8 && j < NLIMBS) {
			buffer |= ((dword_t)red->limb[LIMBPERM(j)]) << fill;
			fill += LIMB_PLACE_VALUE(LIMBPERM(j));
			j++;
		}
		serial[i] = (uint8_t)buffer;
		fill -= 8;
		buffer >>= 8;
	}
}

/* Return high bit of x = low bit of 2x mod p */
mask_t gf_hibit(const gf x) {
	gf y;

	gf_add(y, x, x);
	gf_strong_reduce(y);
	return 0 - (y->limb[0] & 1);
}

/* Return high bit of x = low bit of 2x mod p */
mask_t gf_lobit(const gf x) {
	gf y;

	gf_copy(y, x);
	gf_strong_reduce(y);
	return 0 - (y->limb[0] & 1);
}

/* Deserialize from wire format; return -1 on success and 0 on failure. */
mask_t gf_deserialize(gf x, const uint8_t serial[SER_BYTES], int with_hibit,
		      uint8_t hi_nmask) {
	unsigned int j = 0, fill = 0;
	dword_t buffer = 0;
	dsword_t scarry = 0;
	const unsigned nbytes = with_hibit ? X_SER_BYTES : SER_BYTES;
	unsigned int i;
	mask_t succ;

	for (i = 0; i < NLIMBS; i++) {
		while (fill < LIMB_PLACE_VALUE(LIMBPERM(i)) && j < nbytes) {
			uint8_t sj;

			sj = serial[j];
			if (j == nbytes - 1) sj &= ~hi_nmask;
			buffer |= ((dword_t)sj) << fill;
			fill += 8;
			j++;
		}
		x->limb[LIMBPERM(i)] =
		    (word_t)((i < NLIMBS - 1) ? buffer & LIMB_MASK(LIMBPERM(i))
					      : buffer);
		fill -= LIMB_PLACE_VALUE(LIMBPERM(i));
		buffer >>= LIMB_PLACE_VALUE(LIMBPERM(i));
		scarry = (scarry + x->limb[LIMBPERM(i)] -
			  MODULUS->limb[LIMBPERM(i)]) >>
			 (8 * sizeof(word_t));
	}
	succ = with_hibit ? 0 - (mask_t)1 : ~gf_hibit(x);
	return succ & word_is_zero((word_t)buffer) &
	       ~word_is_zero((word_t)scarry);
}

/* Reduce to canonical form. */
void gf_strong_reduce(gf a) {
	dsword_t scarry;
	word_t scarry_0;
	dword_t carry = 0;
	unsigned int i;

	/* first, clear high */
	gf_weak_reduce(a); /* Determined to have negligible perf impact. */

	/* now the total is less than 2p */

	/* compute total_value - p.  No need to reduce mod p. */
	scarry = 0;
	for (i = 0; i < NLIMBS; i++) {
		scarry =
		    scarry + a->limb[LIMBPERM(i)] - MODULUS->limb[LIMBPERM(i)];
		a->limb[LIMBPERM(i)] = scarry & LIMB_MASK(LIMBPERM(i));
		scarry >>= LIMB_PLACE_VALUE(LIMBPERM(i));
	}

	/*
	 * uncommon case: it was >= p, so now scarry = 0 and this = x common
	 * case: it was < p, so now scarry = -1 and this = x - p + 2^255 so
	 * let's add back in p.  will carry back off the top for 2^255.
	 */
	assert(scarry == 0 || scarry == -1);

	scarry_0 = (word_t)scarry;

	/* add it back */
	for (i = 0; i < NLIMBS; i++) {
		carry = carry + a->limb[LIMBPERM(i)] +
			(scarry_0 & MODULUS->limb[LIMBPERM(i)]);
		a->limb[LIMBPERM(i)] = carry & LIMB_MASK(LIMBPERM(i));
		carry >>= LIMB_PLACE_VALUE(LIMBPERM(i));
	}

	assert(carry < 2 && ((word_t)carry + scarry_0) == 0);
}

/* Subtract two gf elements d=a-b */
void gf_sub(gf d, const gf a, const gf b) {
	gf_sub_RAW(d, a, b);
	gf_bias(d, 2);
	gf_weak_reduce(d);
}

/* Add two field elements d = a+b */
void gf_add(gf d, const gf a, const gf b) {
	gf_add_RAW(d, a, b);
	gf_weak_reduce(d);
}

/* Compare a==b */
mask_t gf_eq(const gf a, const gf b) {
	gf c;
	mask_t ret = 0;
	unsigned int i;

	gf_sub(c, a, b);
	gf_strong_reduce(c);

	for (i = 0; i < NLIMBS; i++) ret |= c->limb[LIMBPERM(i)];

	return word_is_zero(ret);
}

uint128_t widemul(uint64_t a, uint64_t b) { return ((uint128_t)a) * b; }

void ossl_gf_mul(gf_s *RESTRICT cs, const gf as, const gf bs) {
	const uint64_t *a = as->limb, *b = bs->limb;
	uint64_t *c = cs->limb;
	uint128_t accum0 = 0, accum1 = 0, accum2;
	uint64_t mask = (1ULL << 56) - 1;
	uint64_t aa[4], bb[4], bbb[4];
	unsigned int i, j;

	for (i = 0; i < 4; i++) {
		aa[i] = a[i] + a[i + 4];
		bb[i] = b[i] + b[i + 4];
		bbb[i] = bb[i] + b[i + 4];
	}

	for (i = 0; i < 4; i++) {
		accum2 = 0;

		for (j = 0; j <= i; j++) {
			accum2 += widemul(a[j], b[i - j]);
			accum1 += widemul(aa[j], bb[i - j]);
			accum0 += widemul(a[j + 4], b[i - j + 4]);
		}
		for (; j < 4; j++) {
			accum2 += widemul(a[j], b[i + 8 - j]);
			accum1 += widemul(aa[j], bbb[i + 4 - j]);
			accum0 += widemul(a[j + 4], bb[i + 4 - j]);
		}

		accum1 -= accum2;
		accum0 += accum2;

		c[i] = ((uint64_t)(accum0)) & mask;
		c[i + 4] = ((uint64_t)(accum1)) & mask;

		accum0 >>= 56;
		accum1 >>= 56;
	}

	accum0 += accum1;
	accum0 += c[4];
	accum1 += c[0];
	c[4] = ((uint64_t)(accum0)) & mask;
	c[0] = ((uint64_t)(accum1)) & mask;

	accum0 >>= 56;
	accum1 >>= 56;

	c[5] += ((uint64_t)(accum0));
	c[1] += ((uint64_t)(accum1));
}

void ossl_gf_mulw_unsigned(gf_s *RESTRICT cs, const gf as, uint32_t b) {
	const uint64_t *a = as->limb;
	uint64_t *c = cs->limb;
	uint128_t accum0 = 0, accum4 = 0;
	uint64_t mask = (1ULL << 56) - 1;
	int i;

	for (i = 0; i < 4; i++) {
		accum0 += widemul(b, a[i]);
		accum4 += widemul(b, a[i + 4]);
		c[i] = accum0 & mask;
		accum0 >>= 56;
		c[i + 4] = accum4 & mask;
		accum4 >>= 56;
	}

	accum0 += accum4 + c[4];
	c[4] = accum0 & mask;
	c[5] += accum0 >> 56;

	accum4 += c[0];
	c[0] = accum4 & mask;
	c[1] += accum4 >> 56;
}

void ossl_gf_sqr(gf_s *RESTRICT cs, const gf as) {
	const uint64_t *a = as->limb;
	uint64_t *c = cs->limb;
	uint128_t accum0 = 0, accum1 = 0, accum2;
	uint64_t mask = (1ULL << 56) - 1;
	uint64_t aa[4];
	unsigned int i;

	/* For some reason clang doesn't vectorize this without prompting? */
	for (i = 0; i < 4; i++) aa[i] = a[i] + a[i + 4];

	accum2 = widemul(a[0], a[3]);
	accum0 = widemul(aa[0], aa[3]);
	accum1 = widemul(a[4], a[7]);

	accum2 += widemul(a[1], a[2]);
	accum0 += widemul(aa[1], aa[2]);
	accum1 += widemul(a[5], a[6]);

	accum0 -= accum2;
	accum1 += accum2;

	c[3] = ((uint64_t)(accum1)) << 1 & mask;
	c[7] = ((uint64_t)(accum0)) << 1 & mask;

	accum0 >>= 55;
	accum1 >>= 55;

	accum0 += widemul(2 * aa[1], aa[3]);
	accum1 += widemul(2 * a[5], a[7]);
	accum0 += widemul(aa[2], aa[2]);
	accum1 += accum0;

	accum0 -= widemul(2 * a[1], a[3]);
	accum1 += widemul(a[6], a[6]);

	accum2 = widemul(a[0], a[0]);
	accum1 -= accum2;
	accum0 += accum2;

	accum0 -= widemul(a[2], a[2]);
	accum1 += widemul(aa[0], aa[0]);
	accum0 += widemul(a[4], a[4]);

	c[0] = ((uint64_t)(accum0)) & mask;
	c[4] = ((uint64_t)(accum1)) & mask;

	accum0 >>= 56;
	accum1 >>= 56;

	accum2 = widemul(2 * aa[2], aa[3]);
	accum0 -= widemul(2 * a[2], a[3]);
	accum1 += widemul(2 * a[6], a[7]);

	accum1 += accum2;
	accum0 += accum2;

	accum2 = widemul(2 * a[0], a[1]);
	accum1 += widemul(2 * aa[0], aa[1]);
	accum0 += widemul(2 * a[4], a[5]);

	accum1 -= accum2;
	accum0 += accum2;

	c[1] = ((uint64_t)(accum0)) & mask;
	c[5] = ((uint64_t)(accum1)) & mask;

	accum0 >>= 56;
	accum1 >>= 56;

	accum2 = widemul(aa[3], aa[3]);
	accum0 -= widemul(a[3], a[3]);
	accum1 += widemul(a[7], a[7]);

	accum1 += accum2;
	accum0 += accum2;

	accum2 = widemul(2 * a[0], a[2]);
	accum1 += widemul(2 * aa[0], aa[2]);
	accum0 += widemul(2 * a[4], a[6]);

	accum2 += widemul(a[1], a[1]);
	accum1 += widemul(aa[1], aa[1]);
	accum0 += widemul(a[5], a[5]);

	accum1 -= accum2;
	accum0 += accum2;

	c[2] = ((uint64_t)(accum0)) & mask;
	c[6] = ((uint64_t)(accum1)) & mask;

	accum0 >>= 56;
	accum1 >>= 56;

	accum0 += c[3];
	accum1 += c[7];
	c[3] = ((uint64_t)(accum0)) & mask;
	c[7] = ((uint64_t)(accum1)) & mask;

	/* we could almost stop here, but it wouldn't be stable, so... */

	accum0 >>= 56;
	accum1 >>= 56;
	c[4] += ((uint64_t)(accum0)) + ((uint64_t)(accum1));
	c[0] += ((uint64_t)(accum1));
}

mask_t gf_isr(gf a, const gf x) {
	gf L0, L1, L2;

	ossl_gf_sqr(L1, x);
	ossl_gf_mul(L2, x, L1);
	ossl_gf_sqr(L1, L2);
	ossl_gf_mul(L2, x, L1);
	gf_sqrn(L1, L2, 3);
	ossl_gf_mul(L0, L2, L1);
	gf_sqrn(L1, L0, 3);
	ossl_gf_mul(L0, L2, L1);
	gf_sqrn(L2, L0, 9);
	ossl_gf_mul(L1, L0, L2);
	ossl_gf_sqr(L0, L1);
	ossl_gf_mul(L2, x, L0);
	gf_sqrn(L0, L2, 18);
	ossl_gf_mul(L2, L1, L0);
	gf_sqrn(L0, L2, 37);
	ossl_gf_mul(L1, L2, L0);
	gf_sqrn(L0, L1, 37);
	ossl_gf_mul(L1, L2, L0);
	gf_sqrn(L0, L1, 111);
	ossl_gf_mul(L2, L1, L0);
	ossl_gf_sqr(L0, L2);
	ossl_gf_mul(L1, x, L0);
	gf_sqrn(L0, L1, 223);
	ossl_gf_mul(L1, L2, L0);
	ossl_gf_sqr(L2, L1);
	ossl_gf_mul(L0, L2, x);
	gf_copy(a, L1);
	return gf_eq(L0, ONE);
}
