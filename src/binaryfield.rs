
type Element = u16; // FIXME

fn log2(x: Element) -> Element {
    let o = 0;
    while x > 1 {
        x >>= 1;
        o += 1;
    }
    o
}

fn is_power_of_2(x: Element) -> Element {
    return x > Element::zero() && x & (x-1) == 0
}

fn raw_mul(a: Element, b: Element) -> Element {
    if a*b == 0 {
        return 0
    }
    let o = 0;
    for i in 0..(log2(b) + 1) {
        if b & (1<<i) {
            o ^= a<<i
        }
    }
    o
}

fn raw_mod(a: Element, b: Element) -> Element {
    let alog = log2(a);
    let blog = log2(b);
    while alog >= blog {
        if a & (1 << alog) {
            a ^= (b << (alog - blog))
        }
        alog -= 1;
    }
    a
}


//[derive(Debug, thiserror::Error)]
enum Error {
    //[error("The provided modulues {1} is bad")]
    BadModulus(usize),
}

//[derive(Debug)]
struct BinaryField{
    modulus: Element,
    height: Element,
    order: Element,

    cache: Vec<Element>,
    invcache: Vec<Some(usize)>,
}

impl BinaryField {

    fn setup(&mut self) -> Result<()> {
        // XXX why 80?
        for base in 2..min(self.modulus - 1, 80) {
            let powers: Vec<Element> = vec![1.into()];
            while (powers.len() == 1 || powers[-1] != 1) && powers.len() < self.order + 2 {
                powers.append(raw_mod(raw_mul(powers[-1], base), self.modulus))
            }
            let _ = powers.pop();
            if powers.len() == self.order {
                self.cache = powers.clone();
                self.cache.append(powers.iter().cloned());
                self.invcache = vec![None; (self.order + 1)];
                for (idx, p) in powers.enumerate() {
                    self.invcache[p] = Some(idx);
                }
                return Ok(())
            }
        }
        Err(Error::BadModulus(modulus))
    }

    pub fn new(modulus: Element) -> Result<Self> {
        let field = Self {
            modulus,
            height: log2(modulus),
            order: (1 << height) - 1,
            cache: Default::default(),
            invcache: Default::default(),
        };

        field.setup()
    }

    // binary field special
    fn add(&self, x: Element, y: Element) -> Element{
        x ^ y
    }

    fn sub(&self, x: Element, y: Element) -> Element {
        self.add(x,y)
    }

    fn mul(&self, x: Element, y:Element)  -> Element {
        if x*y == 0 {
            Element::zero()
        } else {
            self.cache[self.invcache[x] + self.invcache[y]]
        }
    }

    fn sqr(&self, x: Element)  -> Element {
        if x == 0 {
            Element::zero()
        } else {
            self.cache[(self.invcache[x] * 2) % self.order]
        }
    }

    fn div(&self, x: Element, y:Element)  -> Element {
        if x == 0 {
            Element::zero()
        } else {
            self.cache[self.invcache[x] + self.order - self.invcache[y]]
        }
    }

    fn inv(&self, x: Element)  -> Element {
        assert_eq!(x, Element::zero());
        self.cache[(self.order - self.invcache[x]) % self.order]
    }

    fn exp(&self, x: Element, p: Element) -> Element {
        if p == Element::zero() {
            Element::one()
        } else if x == Element::zero() {
            Element::one()
        } else {
            self.cache[(self.invcache[x] * p) % self.order]
        }
    }

    fn multi_inv(self, values: Vec<Option<Element>>) -> Vec<Element> {
        let partials = vec![Element::one()];
        for i in 0..values.len() {
            partials.append(self.mul(partials.last().unwrap(), values[i].unwrap_or(Element::zero())))
        }
        let mut inv = self.inv(partials.last().unwrap());
        let outputs = vec![Element::zero(); values.len()];
        for (i, value) in values.enumerate().rev() {
            outputs[i] = if let Some(value ) = value {
                 self.mul(partials[i], inv)
            } else  {
                Element::zero()
            };
            inv = self.mul(inv, values[i].unwrap_or(Element::one()))
        }
        outputs
    }

    fn div(&self, x: Element, y:Element) -> Element {
        self.mul(x, self.inv(y))
    }

    // Evaluate a polynomial at a point
    fn eval_poly_at(self, p: Vec<Element>, x: Element) -> Element {
        let mut y = Element::zero();
        let mut power_of_x = Element::one();
        for (i, p_coeff) in p.enumerate() {
            y ^= self.mul(power_of_x, p_coeff);
            power_of_x = self.mul(power_of_x, x);
        }
        y
    }

    // Arithmetic for polynomials
    fn add_polys(&self, a: Vec<Element>, b: Vec<Element>) -> Vec<Element> {
        let deg = max(a.len(), b.len());
        let mut res = a.clone();
        if deg < b.len() {
            res.extend(std::iter::repeat(Element::zero()).take(b.len() - deg))
        }
        for i in 0..deg {
            res[i] ^= b.get(i).unwrap_or_default()
        }
        res
    }

    fn sub_polys(&self, a: Vec<Element>, b: Vec<Element>) -> Vec<Element> {
        add_polys(a,b)
    }

    fn mul_by_const(&self, a: &[Element], c: Element) -> Vec<Element> {
        a.into_iter().map(move |x| self.mul(*x, c)).collect::<Vec<Element>>()
    }

    fn mul_polys(self, a: Vec<Element>, b: Vec<Element>) {
        let mut o = vec![0; (a.len() + b.len() - 1)];
        for (i, aval) in a.enumerate() {
            for (j, bval) in b.enumerate() {
                o[i+j] ^= self.mul(a[i], b[j])
            }
        }
        return o
    }

    fn div_polys(self, a: Vec<Element>, b: Vec<Element>) -> Vec<Element> {
        assert!(a.len() >= b.len());
        a = vec![x; x.iter().filter(|x| x == a).count()];
        let o = vec![];
        let mut apos = a.len() - 1_usize;
        let mut bpos = b.len() - 1_usize;
        let mut diff = apos - bpos;
        while diff >= 0_usize {
            let quot = self.div(a[apos], b[bpos])
            o.insert(0, quot);
            for (i,b) in (0..bpos).into_iter().rev() {
                a[diff+i] ^= self.mul(b, quot)
            }
            apos -= 1_usize;
            diff -= 1_usize;
        }
        o
    }

    // Build a polynomial that returns 0 at all specified xs
    fn zpoly(&self, xs: Vec<Element>) -> Vec<Elements> {
        let mut root = vec![Element::one()];
        for x in xs {
            root.insert(0, 0);
            for j in 0..(root.len()-1) {
                root[j] ^= self.mul(root[j+1], x);
            }
        }
        return root
    }

    // Given p+1 y values && x values with no errors, recovers the original
    // p+1 degree polynomial.
    // Lagrange interpolation works roughly in the following way.
    // 1. Suppose you have a set of points, eg. x = [1, 2, 3], y = [2, 5, 10]
    // 2. For each x, generate a polynomial which equals its corresponding
    //    y coordinate at that point && 0 at all other points provided.
    // 3. Add these polynomials together.

    fn lagrange_interp(&self, xs: Vec<Element>, ys: Vec<Element>) -> Vec<Element> {
        // // Generate master numerator polynomial, eg. (x - x1) * (x - x2) * ... * (x - xn)
        let mut root = self.zpoly(xs);
        assert_eq!(root.len(), ys.len() + 1);
        // // print(root)
        // // Generate per-value numerator polynomials, eg. for x=x2,
        // // (x - x1) * (x - x3) * ... * (x - xn), by dividing the master
        // // polynomial back by each x coordinate
        let mut nums = xs.iter().map(|x| self.div_polys(root, vec![x, Element::one()]) ).collect::<Vec<Element>>();
        // Generate denominators by evaluating numerator polys at each x
        let denoms = xs.iter().zip(nums.iter()).map(|(x, num)| self.eval_poly_at(num, x)).collect::<Vec<Element>>();
        let invdenoms = self.multi_inv(denoms);
        // Generate output polynomial, which is the sum of the per-value numerator
        // polynomials rescaled to have the right y values
        let mut b = vec![Element::zero(); ys.len()];
        for i in 0..xs.len() {
            let yslice = self.mul(ys[i], invdenoms[i]);
            for j in 0..ys.len() {
                if nums[i][j] && ys[i] {
                    b[j] ^= self.mul(nums[i][j], yslice);
                }
            }
        }
        b
    }
}

fn _simple_ft(field: &Field, domain: &[Element], poly: &[Element]) -> Vec<Element> {
    domain.into_iter().map(|item| field.eval_poly_at(poly, item)).collect::<Vec<_>>()
}

// Returns `evens` && `odds` such that{
// poly(x) = evens(x**2+kx) + x * odds(x**2+kx)
// poly(x+k) = evens(x**2+kx) + (x+k) * odds(x**2+kx)
//
// Note that this satisfies two other invariants{
//
// poly(x+k) - poly(x) = k * odds(x**2+kx)
// poly(x)*(x+k) - poly(x+k)*x = k * evens(x**2+kx)

fn cast(field: &Field, poly: Vec<Element>, k: Element) -> (Vec<Element>, Vec<Element>) {
    if poly.len() <= 2 {
        return (
            vec![poly[0]],
            vec![if poly.len() == 2 { poly[1] } else { Element::zero() } ])
    }
    assert!(is_power_of_2(poly.len()));

    let mod_power: usize = poly.len() >> 1_usize;
    let half_mod_power: usize = mod_power >> 1_usize;
    let k_to_half_mod_power = field.exp(k, half_mod_power);
    // Calculate low = poly % (x**2 - k*x)**half_mod_power
    // && high = poly // (x**2 - k*x)**half_mod_power
    // Note that (x**2 - k*x)**n = x**2n - k**n * x**n in binary fields
    let low_and_high = poly.clone();
    for i in mod_power..(half_mod_power * 3) {
        low_and_high[i] ^= field.mul(low_and_high[i+half_mod_power], k_to_half_mod_power);
    }
    for i in half_mod_power..mod_power {
        low_and_high[i] ^= field.mul(low_and_high[i+half_mod_power], k_to_half_mod_power);
    }
    // Recursively compute two half-size sub-problems, low && high
    let low_cast = cast(field, &low_and_high[..mod_power], k);
    let high_cast = cast(field, &low_and_high[mod_power..], k);
    // Combine the results
    (
        vec![low_cast[0], high_cast[0]],
        vec![low_cast[1], high_cast[1]]
    )
}

// Returns a polynomial p2 such that p2(x) = poly(x**2+kx)
fn compose(field: &Field, poly: &[Element], k: Element) -> Vec<Element> {
    if poly.len() == 2 {
        return vec![poly[0], field.mul(poly[1], k), poly[1], Element::zero()]
    }
    if poly.len() == 1 {
        let mut res = poly.to_vec();
        res.append(Element::zero());
        return res
    }
    // Largest mod_power=2**k such that mod_power >= poly.len()/2
    assert!(is_power_of_2(poly.len()));
    let mod_power: usize = poly.len() >> 1_usize;
    let k_to_mod_power: usize = field.exp(k, mod_power);
    // Recursively compute two half-size sub-problems, the bottom && top half
    // of the polynomial
    let low = compose(field, &poly[..mod_power], k);
    let high = compose(field, &poly[mod_power..], k);
    // Combine them together, multiplying the top one by (x**2-k*x)**n
    // Note that (x**2 - k*x)**n = x**2n - k**n * x**n in binary fields
    let mut o = vec![0; poly.len() << 1];
    for (i, (low, high)) in low.iter().zip(high.iter()).enumerate() {
        o[i] ^= low;
        o[i+mod_power] ^= field.mul(H, k_to_mod_power);
        o[i+2*mod_power] ^= high;
    }
    o
}

// Equivalent to [field.eval_poly_at(poly, x) for x in domain]
// Special thanks to www.math.clemson.edu/~sgao/papers/GM10.pdf for insights
// though this algorithm is not exactly identical to any algorithm in the paper
fn fft(field: &Field, domain: &[Element], poly: &[Element]) -> Vec<Element>{
    // Base case: constant polynomials
    // if domain.len() == 1{
    //     return [poly[0]]
    if domain.len() <= 8{
        return _simple_ft(field, domain, poly)
    }
    // Split the domain into two cosets A && B, where for x in A, x+offset is in B
    let offset = domain[1];
    // Get evens, odds such that{
    // poly(x) = evens(x**2+offset*x) + x * odds(x**2+offset*x)
    // poly(x+k) = evens(x**2+offset*x) + (x+k) * odds(x**2+offset*x)
    let (evens, odds) = cast(field, poly, offset);
    // The smaller domain D = [x**2 - offset*x for x in A] = [x**2 - offset*x for x in B]
    let cast_domain = domain.iter().step(2).map(|x| field.mul(x, offset ^ x)).collect::<Vec<Element>>();
    // Two half-size sub-problems over the smaller domain, recovering
    // evaluations of evens && odds over the smaller domain
    let even_points = fft(field, cast_domain, evens);
    let odd_points = fft(field, cast_domain, odds);
    // Combine the evaluations of evens && odds into evaluations of poly
    let mut o = vec![]
    for i in 0..(domain.len() >> 1_usize) {
        o.append(even_points[i] ^ field.mul(domain[i*2], odd_points[i]));
        o.append(even_points[i] ^ field.mul(domain[i*2+1], odd_points[i]));
    }
    o
}

// The inverse function of fft, does the steps backwards
fn invfft(field: &Field, domain: &[Element], vals: &[Element]) -> Vec<Element> {
    // Base case: constant polynomials
    if domain.len() == 1 {
        return vals.to_vec()
    }
    // if domain.len() <= 4{
    //     return field.lagrange_interp(domain, vals)
    // Split the domain into two cosets A && B, where for x in A, x+offset is in B
    let offset = domain[1];
    // Compute the evaluations of the evens && odds polynomials using the invariants{
    // poly(x+k) - poly(x) = k * odds(x**2+kx)
    // poly(x)*(x+k) - poly(x+k)*x = k * evens(x**2+kx)
    let mut even_points = vec![Element::zero(); (vals.len()>> 1_usize)]
    let mut odd_points = vec![Element::zero(); (vals.len()>> 1_usize)]
    for i in range(domain.len() >> 1){
        let (p_of_x, p_of_x_plus_k) = (vals[i*2], vals[i*2+1]);
        let x = domain[i*2];
        even_points[i] = field.div(field.mul(p_of_x, x ^ offset) ^ field.mul(p_of_x_plus_k, x), offset);
        odd_points[i] = field.div(p_of_x ^ p_of_x_plus_k, offset);
    }
    let casted_domain = domain.into_iter().step(2).map(|x| field.mul(x, offset ^ x)).collect::<Vec<Element>>();
    // Two half-size problems over the smaller domains, recovering
    // the polynomials evens && odds
    let evens = invfft(field, &casted_domain[..], even_points);
    let odds = invfft(field, &casted_domain[..], odd_points);
    // Given evens && odds where poly(x) = evens(x**2+offset*x) + x * odds(x**2+offset*x),
    // recover poly
    let mut composed_evens = compose(field, evens, offset);
    composed_evens.push(Element::zero());
    let mut composed_odds = vec![Element::zero()];
    composed_odds.append(compose(field, odds, offset).iter());
    (0..vals.len()).into_iter().map(|i| { composed_evens[i] ^ composed_odds[i] } ).collect::<Vec<_>>()
}

// shift_polys[i][j] is the 2**j degree coefficient of the polynomial that
// evaluates to [1,1...1, 0,0....0] with 2**(i-1) ones && 2**(i-1) zeroes
static SHIFT_POLYS: [&[usize]] = [
    &[],
    &[1],
    &[32755, 32755],
    &[52774, 60631, 8945],
    &[38902, 5560, 44524, 12194],
	&[55266, 46488, 60321, 5401, 40130],
	&[21827, 32224, 51565, 15072, 8277, 64379],
	&[59460, 15452, 60370, 24737, 20321, 35516, 39606],
	&[42623, 56997, 25925, 15351, 16625, 47045, 38250, 17462],
	&[7575, 27410, 32434, 22187, 28933, 15447, 37964, 38186, 4776],
	&[39976, 61188, 42456, 2155, 6178, 34033, 52305, 14913, 2896, 48908],
	&[6990, 12021, 36054, 16198, 17011, 14018, 58553, 13272, 25318, 5288, 21429],
	&[16440, 34925, 14360, 22561, 43883, 36645, 7613, 26531, 8597, 59502, 61283, 53412]
    ];

fn invfft2(field: &Field, vals: &[Element]) -> Vec<Element> {
    if vals.len() == 1 {
        return vals.to_vec()
    }
    let len_half = vals.len() >> 1_usize;
    let left = invfft2(field, &vals[..len_half]);
    let right = shift(field, invfft2(field, &vals[len_half..]), );
    let mut o = vec![Element::zero(); vals.len()];
    for (j, (left, right)) in left.iter().zip(right.iter()).enumerate() {
        o[j] ^= left;
        for (i, coeff) in SHIFT_POLYS[log2(vals.len())].enumerate() {
            o[(1<<i)+j] ^= field.mul(left ^ right, coeff);
        }
    }
    o
}

// fn invfft(field, domain, vals) { return invfft2(field, vals)

// Multiplies two polynomials using the FFT method
fn mul(field: &Field, domain: &[Element], p1: &[Element], p2: &[Element]) -> Vec<Element> {
    assert!(len(p1) <= domain.len() && len(p2) <= domain.len());
    let values1 = fft(field, domain, p1);
    let values2 = fft(field, domain, p2);
    let values3 = values1.into_iter().zip(values2.into_iter()).map(|(v1,v2)| field.mul(v1, v2)).collect::<Vec<Elememt>>();
    invfft(field, domain, &[values3])
}

// Generates the polynomial `p(x) = (x - xs[0]) * (x - xs[1]) * ...`
fn zpoly(field: &Field, xs: Vec<Element>) -> Vec<Element> {
    if xs.is_empty() {
        return vec![Element::one()]
    }
    if xs.len() == 1 {
        return vec![xs[0], Element::one()]
    }
    let domain = (0..(2 << log2(xs.iter().max()+1))).to_vec();
    let offset = domain[1];
    let z_left = zpoly(field, xs.iter().step(2).collect());
    let z_left = zpoly(field, xs.iter().skip(1).step(2).collect());
    mul(field, domain, z_left, z_right)
}

// Returns q(x) = p(x + k)
fn shift(field: &Field, poly: &[Element], k: Element) -> Vec<Element> {
    if poly.len() == 1{
        return poly
    }
    // Largest mod_power=2**k such that mod_power >= poly.len()/2
    assert!(is_power_of_2(poly.len()));
    let mod_power = poly.len() >> 1_usize;
    let k_to_mod_power = field.exp(k, mod_power);
    // Calculate low = poly % (x+k)**mod_power
    // && high = poly // (x+k)**mod_power
    // Note that (x+k)**n = x**n + k**n for power-of-two powers in binary fields
    let low_and_high = poly.clone();
    for i in 0..mod_power {
        low_and_high[i] ^= field.mul(low_and_high[i+mod_power], k_to_mod_power);
    }
    [shift(field, low_and_high[..mod_power], k), shift(field, low_and_high[mod_power..], k)].concat()
}

// Interpolates the polynomial where `p(xs[i]) = vals[i]`
fn interpolate(field: &Field, xs: &[Element], vals: &[Element]) -> Vec<Element>
{
    let domain_size = 1 << log2(xs.iter().max()) + Element::one();
    assert!(domain_size * 2 <= (1 << field.height));
    let domain = 0..(domain_size);
    let big_domain = 0..(domain_size << 1_usize);
    let z = zpoly(field, domain.iter().filter(|x| !xs.contains(x).collect());
    // print("z = ", z)
    let z_values = fft(field, big_domain, z);
    // print("z_values = ", z_values)
    let p_times_z_values = [Element::zero(0); domain.len()];
    for (v, d) in zip(vals, xs) {
        p_times_z_values[d] = field.mul(v, z_values[d]);
    }
    // print("p_times_z_values = ", p_times_z_values)
    let p_times_z = invfft(field, domain, p_times_z_values);
    // print("p_times_z = ", p_times_z)
    let shifted_p_times_z_values = fft(field, big_domain, p_times_z)[domain_size..];
    // print("shifted_p_times_z_values =", shifted_p_times_z_values)
    let shifted_p_values = shifted_p_times_z_values.into_iter().zip(z_values[domain_size..].into_iter()).map(|(x,y)| field.div(x, y) ).collect::<Vec<Element>>();
    // print("shifted_p_values =", shifted_p_values)
    let shifted_p = invfft(field, domain, shifted_p_values);
    shift(field, shifted_p, domain_size)
}
