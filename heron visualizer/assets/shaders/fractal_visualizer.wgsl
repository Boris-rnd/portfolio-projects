struct FragmentInput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(2) @binding(0) var<uniform> cam_center_pos: vec2<f32>;
@group(2) @binding(1) var<uniform> zoom: vec2<f32>;
@group(2) @binding(2) var<storage> ref_orbit: array<vec2<f32>>;


@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    // Transform UV coordinates based on camera position and zoom
    // let uv = (input.uv - 0.5) * zoom + center_pos;
    let center_pos = C128(f32_to_f128(cam_center_pos.x), f32_to_f128(-cam_center_pos.y));
    var uvc = C128(f32_to_f128(input.uv.x), f32_to_f128(input.uv.y));
    let scaled = csub128(uvc, C128(f32_to_f128(0.5), f32_to_f128(0.5)));
    uvc = C128(
        f128_add(f128_mul(scaled.real, f32_to_f128(zoom.x)), center_pos.real),
        f128_add(f128_mul(scaled.imag, f32_to_f128(zoom.y)), center_pos.imag)
    );


    var color = vec3(1.0);
    let val = mandelbrotConverges(uvc, 500, f32_to_f128(5.0));
    let prec = 4.0;
    // let val = heronConverges(uvc, f32_to_f128(pow(10.0, -prec)));
    if (val != -1) {
        color = colormap(val, 4);
    }
    return vec4<f32>(color, 1.0);
}

fn hsv2rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let h1 = h * 6.0;
    let x = c * (1.0 - abs(fract(h1 / 2.0) * 2.0 - 1.0));
    let m = v - c;
    
    var rgb: vec3<f32>;
    if (h1 < 1.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h1 < 2.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h1 < 3.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h1 < 4.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h1 < 5.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    return rgb + m;
}

fn colormap(val: i32, maxIter: i32) -> vec3<f32> {
    if (val == 0) {
        return vec3<f32>(0.0); // Black for points in set
    }
    
    // Convert iteration count to smooth color
    let t = f32(val) / f32(maxIter);
    
    // Create different color bands
    let h = fract(0.5 + t * 0.5); // Hue cycles through colors
    let s = 0.8 + 0.2 * sin(t * 6.28318); // Saturation oscillation
    let v = 0.7 + 0.3 * cos(t * 4.28318); // Value/brightness oscillation
    
    return hsv2rgb(h, s, v);
}


// Complex helpers

// complex multiply
fn cmul(p: C64, q: C64) -> C64 {
    let real = f64_sub(f64_mul(p.real, q.real), f64_mul(p.imag, q.imag));
    let imag = f64_add(f64_mul(p.real, q.imag), f64_mul(p.imag, q.real));
    return C64(real, imag);
}
// complex divide p / q, safe if q != 0
fn cdiv(p: C64, q: C64) -> C64 {
    var d = mag_squared(q);
    // avoid division by exact zero; if d is too small, fallback to small real
    if (f64_to_f32(d) < 1e-30) {
        // if denominator is (almost) zero, return a large value to avoid NaNs
        // using p * conj(q) / tiny -> large magnitude
        d = f32_to_f64(1e-30);
    }
    let real = f64_add(f64_mul(p.real, q.real), f64_mul(p.imag, q.imag));
    let imag = f64_sub(f64_mul(p.imag, q.real), f64_mul(p.real, q.imag));
    return C64(f64_div(real, d), f64_div(imag, d));
}
// complex subtraction p - q
fn csub(p: C64, q: C64) -> C64 {
    return C64(f64_sub(p.real, q.real), f64_sub(p.imag, q.imag));
}
fn cadd(p: C64, q: C64) -> C64 {
    return C64(f64_add(p.real, q.real), f64_add(p.imag, q.imag));
}
// squared magnitude |p|^2
fn mag_squared(p: C64) -> F64 {
    return dot_c64(p, p);
}
// magnitude |p|
fn mag(p: C64) -> F64 {
    return f64_sqrt(mag_squared(p));
}

fn dot_c64(p: C64, q: C64) -> F64 {
    return f64_add(f64_mul(p.real, q.real), f64_mul(p.imag, q.imag));
}




struct F64 {
    low: f32,
    high: f32,
}
struct C64 {
    real: F64,
    imag: F64,
}

fn f32_to_f64(a: f32) -> F64 {
    // Use a scaling factor to split the number
    let factor = 4097.0; // 2^12 + 1
    let temp = a * factor;
    let high = temp - (temp - a);
    let low = a - high;
    return F64(high, low);
}

fn f64_mul(a: F64, b: F64) -> F64 {
    let high1 = a.high * b.high;
    let low1 = a.high * b.low;
    let low2 = a.low * b.high;
    let low3 = a.low * b.low;
    
    let sum = high1 + (low1 + low2 + low3);
    let high = sum;
    let low = (high1 - high) + (low1 + low2 + low3);
    
    return F64(high, low);
}


fn f64_to_f32(a: F64) -> f32 {
    return a.low + a.high;
}

// Improved double-precision addition
fn f64_add(a: F64, b: F64) -> F64 {
    let s = a.high + b.high;
    let v = s - a.high;
    let t1 = (a.high - (s - v)) + (b.high - v);
    let t2 = (a.low + b.low) + t1;
    return F64(s + t2, t2 - ((s + t2) - s));
}


// Double-precision subtraction
fn f64_sub(a: F64, b: F64) -> F64 {
    let t = a.low - b.low;
    let e = t - a.low;
    let low = (a.low - (t - e)) - (b.low + e) + (a.high - b.high);
    return F64(t + low, low - (t + low - t));
}

// Double-precision multiplication
// fn f64_mul(a: F64, b: F64) -> F64 {
//     let t = a.low * b.low;
//     let e = a.low * b.low - t;
//     let low = a.low*b.high +a.high*b.low+e;
//     return F64(t + low, low - (t + low - t));
// }

fn f64_div(a: F64, b: F64) -> F64 {
    let approx = 1.0 / b.low;
    let t = a.low * approx;
    // refine using Newton-Raphson
    let r = f64_sub(f32_to_f64(1.0), f64_mul(b, f32_to_f64(approx)));
    let correction = f64_mul(f32_to_f64(t), r);
    let result = f64_add(f32_to_f64(t), correction);
    return result;
}

fn f64_lt(a: F64, b: F64) -> bool {
    if (a.low < b.low) {
        return true;
    } else if (a.low > b.low) {
        return false;
    } else {
        return a.high < b.high;
    }
}

fn f64_sqrt(a: F64) -> F64 {
    let approx = sqrt(a.low);
    // refine using Newton-Raphson
    let half_approx = approx * 0.5;
    let approx_f64 = f32_to_f64(approx);
    let r = f64_sub(a, f64_mul(approx_f64, approx_f64));
    let correction = f64_div(r, f32_to_f64(2.0 * approx));
    let result = f64_add(approx_f64, correction);
    return result;
}


// Complex Heron/ Newton sqrt convergence test for GLSL
// a: complex number as vec2 (a.x = real, a.y = imag)
// maxSteps: maximum number of iterations allowed
// prec: target precision on residual |z*z - a| (use float, e.g. 1e-6)
fn heronConverges(a: C128, prec: F128) -> i32 {
    let prec_squared = f128_mul(prec, prec);
    // let a = C128(b.real, f32_toè_f);
    // Choose an initial guess z0.
    // A simple safe real positive initial guess: use |a| (magnitude) as real approximation.
    // This avoids introducing an initial zero in many cases.
    var r = mag128(a);
    // let max_r = f32_to_f128(1.0); 
    // if (f128_lt(r, max_r)) {
    //     r = max_r;
    // }
    var z = C128(r, f32_to_f128(0.0));
    // Iteration loop
    for (var i = 0; i < 500; i+=1) {
        // compute a / z (safe division)
        let a_over_z = cdiv128(a, z);

        // Newton (Heron) update: z = (z + a/z) / 2
        let z2 = cadd128(z, a_over_z);
        z = C128(f128_mul(z2.real, f32_to_f128(0.5)), f128_mul(z2.imag, f32_to_f128(0.5)));

        // residual = z*z - a
        let zz = cmul128(z, z);
        let resid = csub128(zz, a);

        // convergence test: |resid| < prec
        if (f128_lt(f128_div(mag_squared128(resid), r), prec)) {
            return i;
        }
    }

    // not converged within maxSteps
    return -1;
}




fn mandelbrotConverges(c: C128, maxSteps: i32, escapeRadius: F128) -> i32 {
    var z = c;
    let escapeRadiusSquared = f128_mul(escapeRadius, escapeRadius);

    for (var i = 0; i < maxSteps; i += 1) {
        // z = z*z + c
        z = cadd128(cmul128(z, z), c);

        // Check for escape
        if (f128_lt(escapeRadiusSquared, mag_squared128(z))) {
            return i;
        }
    }

    return 0; // did not escape
}

fn mandelbrotConverges32(c64: C64, maxSteps: i32, escapeRadius64: F64) -> i32 {
    let escapeRadius = f64_to_f32(escapeRadius64);
    let c = vec2<f32>(f64_to_f32(c64.real), f64_to_f32(c64.imag));
    var z = vec2(0.0);
    let escapeRadiusSquared = escapeRadius*escapeRadius;

    for (var i = 0; i < maxSteps; i += 1) {
        z = cmul32(z, z) + c;
        // z = cadd(cmul(z, z), c);

        // Check for escape
        if (escapeRadiusSquared < dot(z, z)) {
            return i;
        }
    }

    return 0; // did not escape
}

fn cmul32(p: vec2<f32>, q: vec2<f32>) -> vec2<f32> {
    let real = p.x * q.x - p.y * q.y;
    let imag = p.x * q.y + p.y * q.x;
    return vec2<f32>(real, imag);
}



struct F128 {
    x0: f32, // Most significant part
    x1: f32, // Second most significant
    x2: f32, // Third most significant
    x3: f32, // Least significant part
}

fn f32_to_f128(a: f32) -> F128 {
    return F128(a, 0.0, 0.0, 0.0);
}

fn f128_to_f32(a: F128) -> f32 {
    return a.x0 + a.x1 + a.x2 + a.x3;
}

fn quick_two_sum(a: f32, b: f32) -> vec2<f32> {
    let s = a + b;
    let e = b - (s - a);
    return vec2<f32>(s, e);
}

fn two_sum(a: f32, b: f32) -> vec2<f32> {
    let s = a + b;
    let v = s - a;
    let e = (a - (s - v)) + (b - v);
    return vec2<f32>(s, e);
}

fn two_prod(a: f32, b: f32) -> vec2<f32> {
    let p = a * b;
    let e = fma(a, b, -p);
    return vec2<f32>(p, e);
}

fn f128_add(a: F128, b: F128) -> F128 {
    var s0: f32; var s1: f32; var s2: f32; var s3: f32;
    var t0: f32; var t1: f32; var t2: f32; var t3: f32;

    let st0 = two_sum(a.x0, b.x0);
    let st1 = two_sum(a.x1, b.x1);
    let st2 = two_sum(a.x2, b.x2);
    let st3 = two_sum(a.x3, b.x3);

    let v0 = quick_two_sum(st0.x, st1.x);
    let v1 = quick_two_sum(v0.y + st2.x, st3.x);
    let v2 = quick_two_sum(v0.x, v1.x);
    let v3 = quick_two_sum(v2.y + v1.y, st0.y + st1.y + st2.y + st3.y);

    return F128(v2.x, v3.x, v3.y, v1.y);
}

fn f128_mul(a: F128, b: F128) -> F128 {
    let p0 = two_prod(a.x0, b.x0);
    let p1 = two_prod(a.x0, b.x1);
    let p2 = two_prod(a.x1, b.x0);
    let p3 = two_prod(a.x1, b.x1);
    
    let s0 = p0.x;
    let s1 = p0.y + p1.x + p2.x;
    let s2 = p1.y + p2.y + p3.x;
    let s3 = p3.y;
    
    let v0 = quick_two_sum(s0, s1);
    let v1 = quick_two_sum(v0.y + s2, s3);
    let v2 = quick_two_sum(v0.x, v1.x);
    
    return F128(v2.x, v2.y, v1.y, v1.x);
}

fn f128_sub(a: F128, b: F128) -> F128 {
    return f128_add(a, F128(-b.x0, -b.x1, -b.x2, -b.x3));
}

fn f128_lt(a: F128, b: F128) -> bool {
    if (a.x0 != b.x0) { return a.x0 < b.x0; }
    if (a.x1 != b.x1) { return a.x1 < b.x1; }
    if (a.x2 != b.x2) { return a.x2 < b.x2; }
    return a.x3 < b.x3;
}

fn f128_abs(a: F128) -> F128 {
    if (a.x0 < 0.0) {
        return F128(-a.x0, -a.x1, -a.x2, -a.x3);
    }
    return a;
}

// Helper function for division and square root
fn f128_div(a: F128, b: F128) -> F128 {
    let q0 = a.x0 / b.x0;
    let r = f128_sub(a, f128_mul(f32_to_f128(q0), b));
    let q1 = f128_to_f32(r) / b.x0;
    return f128_add(f32_to_f128(q0), f32_to_f128(q1));
}

fn f128_sqrt(a: F128) -> F128 {
    if (a.x0 <= 0.0) { return F128(0.0, 0.0, 0.0, 0.0); }
    
    let x0 = sqrt(a.x0);
    var r = f128_sub(a, f128_mul(f32_to_f128(x0), f32_to_f128(x0)));
    let x1 = f128_to_f32(r) / (2.0 * x0);
    
    return f128_add(f32_to_f128(x0), f32_to_f128(x1));
}
struct C128 {
    real: F128,
    imag: F128,
}

fn c128_mul(a: C128, b: C128) -> C128 {
    let real = f128_sub(
        f128_mul(a.real, b.real),
        f128_mul(a.imag, b.imag)
    );
    let imag = f128_add(
        f128_mul(a.real, b.imag),
        f128_mul(a.imag, b.real)
    );
    return C128(real, imag);
}

fn c128_add(a: C128, b: C128) -> C128 {
    return C128(
        f128_add(a.real, b.real),
        f128_add(a.imag, b.imag)
    );
}

fn mandelbrotConverges128(c: C128, maxSteps: i32, escapeRadius: F128) -> i32 {
    var z = C128(F128(0.0, 0.0, 0.0, 0.0), F128(0.0, 0.0, 0.0, 0.0));
    let escapeRadiusSquared = f128_mul(escapeRadius, escapeRadius);

    for (var i = 0; i < maxSteps; i += 1) {
        z = c128_add(c128_mul(z, z), c);
        
        let mag = f128_add(
            f128_mul(z.real, z.real),
            f128_mul(z.imag, z.imag)
        );
        
        if (f128_lt(escapeRadiusSquared, mag)) {
            return i;
        }
    }
    return 0;
}


fn csub128(p: C128, q: C128) -> C128 {
    return C128(f128_sub(p.real, q.real), f128_sub(p.imag, q.imag));
}
fn cadd128(p: C128, q: C128) -> C128 {
    return C128(f128_add(p.real, q.real), f128_add(p.imag, q.imag));
}
// squared magnitude |p|^2
fn mag_squared128(p: C128) -> F128 {
    return dot_c128(p, p);
}
// magnitude |p|
fn mag128(p: C128) -> F128 {
    return f128_sqrt(mag_squared128(p));
}

fn dot_c128(p: C128, q: C128) -> F128 {
    return f128_add(f128_mul(p.real, q.real), f128_mul(p.imag, q.imag));
}

fn cmul128(p: C128, q: C128) -> C128 {
    let real = f128_sub(f128_mul(p.real, q.real), f128_mul(p.imag, q.imag));
    let imag = f128_add(f128_mul(p.real, q.imag), f128_mul(p.imag, q.real));
    return C128(real, imag);
}

// complex divide p / q, safe if q != 0
fn cdiv128(p: C128, q: C128) -> C128 {
    var d = mag_squared128(q);
    // avoid division by exact zero; if d is too small, fallback to small real
    if (f128_to_f32(d) < 1e-30) {
        // if denominator is (almost) zero, return a large value to avoid NaNs
        // using p * conj(q) / tiny -> large magnitude
        d = f32_to_f128(1e-30);
    }
    let real = f128_add(f128_mul(p.real, q.real), f128_mul(p.imag, q.imag));
    let imag = f128_sub(f128_mul(p.imag, q.real), f128_mul(p.real, q.imag));
    return C128(f128_div(real, d), f128_div(imag, d));
}


fn mandelbrot_perturbation(delta: vec2<f32>) -> f32 {
    var dz = delta;
    var i = 0u;
    
    let len = arrayLength(&ref_orbit);
    while (i < len) {
        let ref_z = ref_orbit[i];
        
        // Perturbation formula: δz' = 2*Z*δz + δz² + δc
        let new_dz = vec2<f32>(
            2.0 * (ref_z.x * dz.x - ref_z.y * dz.y) + dz.x * dz.x - dz.y * dz.y + delta.x,
            2.0 * (ref_z.x * dz.y + ref_z.y * dz.x) + 2.0 * dz.x * dz.y + delta.y
        );
        dz = new_dz;
        
        // Check escape with reference + delta
        let z_total = ref_z + dz;
        if (dot(z_total, z_total) > 4.0) {
            return f32(i) / f32(len);
        }
        i = i + 1u;
    }
    return 1.0;
}

