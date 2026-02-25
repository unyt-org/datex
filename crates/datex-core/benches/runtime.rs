use criterion::{Criterion, criterion_group};

fn bench_runtime(c: &mut Criterion) {
    c.bench_function("runtime init", |b| b.iter(|| 1 + 2));
}

criterion_group! {
    name = runtime_benches;
    config = Criterion::default().sample_size(10);
    targets = bench_runtime
}

// TODO #718: async initialization benchmark
// use datex_core::{
//     runtime::{Runtime, RuntimeConfig},
//     values::core_values::endpoint::Endpoint,
// };
// use log::info;
//
// // simple runtime initialization
// pub fn runtime_init() {
//     let runtime = Runtime::new(
//         RuntimeConfig::new_with_endpoint(Endpoint::new("@+bench")),
//     );
//     info!("Runtime version: {}", runtime.version);
// }
