use crate::traits::ModifyGraph;

pub trait RetainedGraph: ModifyGraph {
    type Buffer: crate::traits::AudioBuffer;

    fn get_inputs_mut(&mut self) -> &mut [Option<Self::Buffer>];

    fn get_outputs_mut(&mut self) -> &mut [Option<Self::Buffer>];

    // `run` function is supplied by caller, since it
    // typically requires copying input/output data in/out
}
