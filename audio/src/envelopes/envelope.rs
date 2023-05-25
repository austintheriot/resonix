use std::ops::Deref;

use crate::SINE_ENVELOPE;

pub type EnvelopeBuffer = [f32; 2048];

pub enum EnvelopeType {
    Sine,
    All0,
    All1,
    Custom(EnvelopeBuffer),
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Envelope {
    Sine(EnvelopeBuffer),
    All0(EnvelopeBuffer),
    All1(EnvelopeBuffer),
    Custom(EnvelopeBuffer),
}

impl Envelope {
    pub fn new_sine() -> Self {
        Self::Sine(SINE_ENVELOPE)
    }

    pub fn new_all_0() -> Self {
        Self::All0([0.0; 2048])
    }

    pub fn new_all_1() -> Self {
        Self::All1([1.0; 2048])
    }

    pub fn new_custom(envelope_buffer: EnvelopeBuffer) -> Self {
        Self::Custom(envelope_buffer)
    }
}

impl Deref for Envelope {
    type Target = EnvelopeBuffer;

    fn deref(&self) -> &Self::Target {
        match self {
            Envelope::Sine(buffer) => buffer,
            Envelope::All0(buffer) => buffer,
            Envelope::All1(buffer) => buffer,
            Envelope::Custom(buffer) => buffer,
        }
    }
}

impl From<EnvelopeType> for Envelope {
    fn from(value: EnvelopeType) -> Self {
        match value {
            EnvelopeType::Sine => Envelope::new_sine(),
            EnvelopeType::All0 => Envelope::new_all_0(),
            EnvelopeType::All1 => Envelope::new_all_1(),
            EnvelopeType::Custom(buffer) => Envelope::new_custom(buffer),
        }
    }
}

impl From<Envelope> for EnvelopeType {
    fn from(value: Envelope) -> Self {
        match value {
            Envelope::Sine(_) => EnvelopeType::Sine,
            Envelope::All0(_) => EnvelopeType::All0,
            Envelope::All1(_) => EnvelopeType::All1,
            Envelope::Custom(buffer) => EnvelopeType::Custom(buffer),
        }
    }
}
