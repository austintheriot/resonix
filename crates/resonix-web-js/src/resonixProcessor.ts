class ResonixProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
  }

  process(
    _inputList: Array<Array<Float32Array>>,
    _outputList: Array<Array<Float32Array>>,
    _parameters: unknown,
  ) {
    // Using the inputs (or not, as needed),
    // write the output into each of the outputs
    // …
    return true;
  }
}

registerProcessor("resonix-processor", ResonixProcessor);
