### Observations

#### 1 - 2 waves with opposite sign, overlapping -> wave interference (construction, destruction)
Start conditions
```rust
for i in start..(start+size) {
    // Has  increase in a sin way
    let x = (i-start) as f32;
    blocks[i].h[0] = (5.0 * (x/(size) as f32 * PI).sin()).powi(2);
}
```

#### 2 - Standing waves, oscillating without moving
Start conditions
```rust
let standing_wave_count = 4.0;
for i in 0..blocks.len() {
    blocks[i].h[0] = ((i as f32/blocks.len() as f32)*PI*standing_wave_count).sin()*100.0;
}
```
With standing_wave_count not an integer, we get multiple smaller waves that travel back and forth (like first example), with still the standing wave pattern (as bigger waves).
