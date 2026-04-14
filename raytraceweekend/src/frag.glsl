
struct Ray {
    vec3 orig;
    vec3 dir;
};

vec3 compute_color(in Ray r, in int depth) {
    if (depth == 0) {return vec3(0.);}

    
    if hit(r) {
        if let Some((ray, attenuation)) = rec.mat.scatter(r, &rec) {
            return attenuation * ray.compute_color(world, depth-1);
        } else {return Vec3::ZERO}
    }

    vec3 unit_dir = normalize(r.dir);
    float a = 0.5 * (unit_dir.y + 1.);
    return (1.0 - a) * vec3(1.) + a * vec3(0.5, 0.7, 1.0);
}


void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    float x = fragCoord.x;
    float y = fragCoord.y;
    int max_depth = 10;
    float focal_length = 1.;
    float viewport_height = 2.;
    vec3 cam_center = vec3(0.,0.,0.);
    float width = iResolution.x;
    float height = iResolution.y;

    float viewport_width = viewport_height * (width / height);
    vec3 viewport_u = vec3(viewport_width, 0., 0.);
    vec3 viewport_v = vec3(0., -viewport_height, 0.);

    vec3 pixel_delta_u = viewport_u / width;
    vec3 pixel_delta_v = viewport_v / height;

    vec3 viewport_upper_left = cam_center - vec3(0., 0., focal_length) - viewport_u / 2. - viewport_v / 2.;
    vec3 pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    vec3 pixel_center = pixel00_loc + (y * pixel_delta_u) + (x * pixel_delta_v);
    Ray r = Ray(pixel_center, pixel_center - cam_center);
    bool antialiasing = false;
    vec3 c;
    if (antialiasing) {
        // for i in 0..samples_per_pixel {
        //     let offset =
        //         vec3(rand::gen_range(-0.5, 0.5), rand::gen_range(-0.5, 0.5), 0.);

        //     let r = Ray {
        //         orig: pixel_center,
        //         dir: (pixel00_loc
        //             + ((y + offset.x) * pixel_delta_u)
        //             + ((x + offset.y) * pixel_delta_v))
        //             - cam_center,
        //     };

        //     c += r.compute_color(&world, max_depth) / samples_per_pixel;
        // }
    } else {
        c += compute_color(r, max_depth);
    }

    fragColor = vec4(sqrt(c.x),sqrt(c.y),sqrt(c.z), 1.);
}
