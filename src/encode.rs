
// Encoding alg for k/n < 0.5: message is a power of two
pub fn encode_low(data: &[Additive], k: usize, codeword: &mut [Additive], n: usize) {
	assert!(k + k <= n);
	assert_eq!(codeword.len(), n);
	assert_eq!(data.len(), n);

	assert!(is_nonzero_power_of_2(n));
	assert!(is_nonzero_power_of_2(k));

	// k | n is guaranteed
	assert_eq!((n / k) * k, n);

	// move the data to the codeword
    codeword.copy_from_slice(data);

	// split after the first k
	let (codeword_first_k, codeword_skip_first_k) = codeword.split_at_mut(k);

    inverse_afft(codeword_first_k, k, 0);

	// the first codeword is now the basis for the remaining transforms
	// denoted `M_topdash`

	for shift in (k..n).into_iter().step_by(k) {
		let codeword_at_shift = &mut codeword_skip_first_k[(shift - k)..shift];
		// copy `M_topdash` to the position we are currently at, the n transform
		codeword_at_shift.copy_from_slice(codeword_first_k);
		afft(codeword_at_shift, k, shift);
	}

	// restore `M` from the derived ones
	(&mut codeword[0..k]).copy_from_slice(&data[0..k]);
}


// TODO: Make encode_high actually work again!  Add tests!

//data: message array. parity: parity array. mem: buffer(size>= n-k)
//Encoding alg for k/n>0.5: parity is a power of two.
pub fn encode_high(data: &[Additive], k: usize, parity: &mut [Additive], mem: &mut [Additive], n: usize) {
	let t: usize = n - k;

	// mem_zero(&mut parity[0..t]);
	for i in 0..t {
		parity[i] = Additive(0);
	}

	let mut i = t;
	while i < n {
		(&mut mem[..t]).copy_from_slice(&data[(i - t)..t]);

		inverse_afft(mem, t, i);
		for j in 0..t {
			parity[j] ^= mem[j];
		}
		i += t;
	}
	afft(parity, t, 0);
}
