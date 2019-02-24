extern crate socool_ast;
use instrument::loudness::loudness_normalization;
use socool_ast::ast::OscType;

#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub index: usize,
    pub past: VoiceState,
    pub current: VoiceState,
    pub phase: f64,
    pub osc_type: OscType,
    pub attack: usize,
    pub decay: usize,
    pub asr: ASR,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ASR {
    ASR,
    AS,
    S,
    R,
    Silence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleInfo {
    pub index: usize,
    pub p_delta: f64,
    pub gain: f64,
    pub portamento_length: usize,
    pub factor: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoiceState {
    pub frequency: f64,
    pub gain: f64,
}

impl VoiceState {
    fn init() -> VoiceState {
        VoiceState {
            frequency: 0.0,
            gain: 0.0,
        }
    }
}

impl Voice {
    pub fn init(index: usize) -> Voice {
        Voice {
            index,
            past: VoiceState::init(),
            current: VoiceState::init(),
            phase: 0.0,
            osc_type: OscType::Sine,
            attack: 10000,
            decay: 10000,
            asr: ASR::Silence,
        }
    }
    pub fn generate_waveform(
        &mut self,
        buffer: &mut Vec<f64>,
        portamento_length: usize,
        factor: f64,
    ) {
//        println!("{:?}", self);
        let p_delta = self.calculate_portamento_delta(portamento_length);

        let buffer_len = buffer.len();

        for (index, sample) in buffer.iter_mut().enumerate() {
            let info = SampleInfo {
                index,
                p_delta,
                gain: self.calculate_gain(buffer_len, index),
                portamento_length,
                factor,
            };
            let new_sample = match self.osc_type {
                OscType::Sine => self.generate_sine_sample(info),
                OscType::Square => self.generate_square_sample(info),
                OscType::Noise => self.generate_random_sample(info),
            };

            *sample += new_sample
        }
    }

    pub fn update(&mut self, mut frequency: f64, gain: f64, osc_type: OscType, silence_next: bool) {
        if frequency < 20.0 {
            frequency = 0.0;
        }

        let mut gain = if frequency != 0.0 { gain } else { 0.0 };
//        if osc_type != OscType::Sine {
//            gain /= 3.0
//        }
        let loudness = loudness_normalization(frequency);
        gain *= loudness;

        if self.osc_type == OscType::Sine && osc_type == OscType::Noise {
            self.past.gain = self.current.gain / 3.0;
        } else {
            self.past.gain = self.current.gain;
        }

        self.osc_type = osc_type;
        self.past.frequency = self.current.frequency;
        self.current.frequency = frequency;
        self.current.gain = gain;

        self.set_asr(silence_next);
//        println!("{:?}", self.asr);
    }

    fn set_asr(&mut self, silence_next: bool) {
        if self.silent() {
            self.asr = ASR::Silence;
        } else {
            match self.asr {
                ASR::Silence | ASR::ASR | ASR::R => {
                    if silence_next {
                        self.asr = ASR::ASR;
                    } else {
                        self.asr = ASR::AS;
                    }
                }
                _ => {
                    if silence_next {
                        self.asr = ASR::R;
                    } else {
                        self.asr = ASR::S;
                    }
                }
            }
        }
    }

    fn silent(&self) -> bool {
        self.current.frequency == 0.0 || self.current.gain == 0.0
    }

    pub fn silence_to_sound(&self) -> bool {
        self.past.frequency == 0.0 && self.current.frequency != 0.0
    }

    pub fn sound_to_silence(&self) -> bool {
        self.past.frequency != 0.0 && self.current.frequency == 0.0
    }

    pub fn calculate_portamento_delta(&self, portamento_length: usize) -> f64 {
        (self.current.frequency - self.past.frequency) / (portamento_length as f64)
    }

    fn is_short(&self, buffer_len: usize) -> bool {
        buffer_len <= self.attack + self.decay
    }

    fn calculate_attack(&self, distance: f64, attack_index: usize, attack_length: usize) -> f64 {
        self.past.gain + (distance * attack_index as f64 / attack_length as f64)
    }

    fn calculate_decay(&self, distance: f64, decay_index: usize, decay_length: usize) -> f64 {
        distance * decay_index as f64 / decay_length as f64
    }

    fn calculate_gain(&mut self, buffer_len: usize, index: usize) -> f64 {
        let short = self.is_short(buffer_len);

        match self.asr {
            ASR::Silence => {
                return 0.0;
            }
            ASR::AS => {
                let len = if short { buffer_len } else { self.attack };
                let distance = self.current.gain - self.past.gain;
                if index <= len {
                    return self.calculate_attack(distance, index, len)
                } else {
                    return self.current.gain;
                }
            }
            ASR::S => {
                return self.past.gain
                    + ((self.current.gain - self.past.gain) * index as f64 / buffer_len as f64);
            }
            ASR::ASR => {
                let mut attack_length = self.attack;
                let mut decay_length = self.decay;

                if short {
                    attack_length = buffer_len / 2;
                    decay_length = buffer_len / 2;
                }
                if index <= attack_length {
                    let distance = self.current.gain - self.past.gain;
                    return self.calculate_attack(distance, index, attack_length)
                } else if index > buffer_len - decay_length {
                    let decay_index = buffer_len - (index + 1);
                    self.calculate_decay(self.current.gain, decay_index, decay_length)
                } else {
                    return self.current.gain;
                }
            }
            ASR::R => {
                if short {
                    let decay_index = buffer_len - (index + 1);
                    return self.calculate_decay(self.past.gain, decay_index, buffer_len)
                };

                let len = buffer_len - self.decay;
                if index < len {
                    let distance = self.current.gain - self.past.gain;
                    let result = self.past.gain + (distance * (index as f64 / len as f64));
                    return result;
                } else {
                    let decay_index = buffer_len - (index + 1);
                    return self.calculate_decay(self.current.gain, decay_index, self.decay)
                };
            }
        }
    }
}
