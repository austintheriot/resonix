use criterion::{Criterion, criterion_group, criterion_main};
use resonix_graph::{
    implementations::{AudioBuffer, AudioBufferMut, ConstantNode, Graph, OutputNode},
    traits::Graph as _,
};

fn create_and_run_constant_to_external_graph() {
    let mut graph = Graph::with_block_size(1);

    let constant_node = ConstantNode::new(&mut graph);
    let constant_node_handle = graph.add_audio_node(constant_node).unwrap();

    let output_node = OutputNode::new(&mut graph);
    let output_node_handle = graph.add_audio_node(output_node).unwrap();

    graph
        .connect(
            constant_node_handle.output_port_address(),
            output_node_handle.input_port_address(),
        )
        .unwrap();

    graph
        .run::<AudioBuffer<'_>, AudioBufferMut<'_>>(&[], &mut [])
        .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("making & running Constant -> External nodes", |b| {
        b.iter(create_and_run_constant_to_external_graph)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
