use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rt::vec3::Vec3;

pub fn vec_bench(c: &mut Criterion) {
    // Benchmark for Vec3 addition
    c.bench_function("vec3_addition", |b| {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        b.iter(|| black_box(v1 + v2));
    });

    // Benchmark for Vec3 dot product
    c.bench_function("vec3_dot_product", |b| {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        b.iter(|| black_box(v1.dot(v2)));
    });

    // Benchmark for Vec3 cross product
    c.bench_function("vec3_cross_product", |b| {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        b.iter(|| black_box(v1.cross(v2)));
    });
}

criterion_group!(benches, vec_bench);
criterion_main!(benches);
