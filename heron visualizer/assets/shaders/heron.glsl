
#ifdef GL_ES
precision mediump float;
#endif


// squared magnitude |p|^2
float mag2(in vec2 p) {
    return dot(p, p);
}
// magnitude |p|
float mag(in vec2 p) {
    return sqrt(mag2(p));
}
// HSV to RGB helper
vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0/3.0, 1.0/3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

// Map number to color
vec3 numberToColor(int x, int n) {
    float t = clamp(log(float(x)) / log(float(n)), 0.0, 1.0); // normalize
    float hue = t;                     // hue from 0 to 1
    float saturation = 1.0;
    float value = 1.0;
    return hsv2rgb(vec3(hue, saturation, value));
}


// Complex arithmetic helpers
vec2 cadd(vec2 a, vec2 b) { return vec2(a.x + b.x, a.y + b.y); }
vec2 csub(vec2 a, vec2 b) { return vec2(a.x - b.x, a.y - b.y); }
vec2 cmul(vec2 a, vec2 b) { return vec2(a.x*b.x - a.y*b.y, a.x*b.y + a.y*b.x); }
vec2 cdiv(vec2 a, vec2 b) { 
    float denom = b.x*b.x + b.y*b.y;
    return vec2((a.x*b.x + a.y*b.y)/denom, (a.y*b.x - a.x*b.y)/denom);
}
float cmag(vec2 z) { return sqrt(z.x*z.x + z.y*z.y); }
float cmag2(vec2 z) { return z.x*z.x + z.y*z.y; }

// Heron (Newton-Raphson) method for complex sqrt
int heronConvergesComplex(vec2 a, float prec, int iter) {

    // trivial case: a == 0 -> sqrt(0) = 0 converges immediately
    if (cmag2(a) == 0.0) {
        return 0;
    }

    // initial guess: use magnitude of a as real part
    float r = cmag(a);
    vec2 z = vec2(max(1.0, r), 0.0);

    // iteration loop
    for (int i = 0; i <= 30000; ++i) {
        // Newton update: z = (z + a/z) / 2
        vec2 a_over_z = cdiv(a, z);
        z = 0.5 * cadd(z, a_over_z);

        // residual = z^2 - a
        vec2 resid = csub(cmul(z, z), a);

        // convergence test
        if (cmag(resid)/r < prec) return i;
        if (i > iter) {
            break;
        }
    }

    // not converged
    return -1;
}

void main() {
    vec2 st = gl_FragCoord.xy/u_resolution.xy;
    st.x *= u_resolution.x/u_resolution.y;
    vec3 color = vec3(0.1);
    
    vec2 pos = vec2(-0.080,0.710);
    float zoom = 1.0;
    float prec = 2.0;
    int iter = 10000;
    int val = heronConvergesComplex((st)/zoom+pos, pow(10.0, -prec), iter);
    
    if (val != -1) {
        color = numberToColor(val, iter);
    } else {
        color = vec3(1.0);
    }

    gl_FragColor = vec4(color,1.0);
}