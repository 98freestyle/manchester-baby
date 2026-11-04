// The Baby's CPU: the counter (CI), the present instruction (PI), the accumulator,
// and the logic that carries out instructions.

use crate::memory::{Memory, Row};

// Instruction numbers, as the Baby's designers assigned them.
const JUMP: Row = 0;
const JUMP_FORWARD: Row = 1;
const LOAD: Row = 2;
const STORE: Row = 3;
const SUBTRACT: Row = 4;
// The Baby had a second number for subtract that did exactly the same thing.
const SUBTRACT_ALSO: Row = 5;
const SKIP_IF_NEGATIVE: Row = 6;
const STOP: Row = 7;

pub struct Cpu {
    // Which row the CPU is up to. The Baby called this CI, the control instruction.
    // It holds the row of the instruction that just ran.
    pub counter: Row,
    // The instruction currently being carried out. The Baby called this PI, the present instruction.
    pub present_instruction: Row,
    // Holds the result of the last calculation.
    pub accumulator: Row,
    // Set to true once a stop instruction runs. On the real Baby, this lit the stop lamp.
    pub stopped: bool,
}

impl Cpu {
    // Creates a CPU with everything at 0, not stopped.
    pub fn new() -> Cpu {
        Cpu {
            counter: 0,
            present_instruction: 0,
            accumulator: 0,
            stopped: false,
        }
    }

    // Runs one instruction from memory: move to the next row, read it, carry it out.
    // Returns the instruction that was run, so it can be displayed.
    pub fn step(&mut self, memory: &mut Memory) -> Row {
        self.counter = self.counter.wrapping_add(1);
        // Only the bottom 5 bits of the counter pick the row, so it wraps around within rows 0 to 31.
        let instruction = memory[(self.counter & 0b11111) as usize];
        self.execute(instruction, memory);
        instruction
    }

    // Carries out one instruction.
    // Normally the instruction comes from memory (see step). In manual mode it comes
    // straight from the switches on the control panel instead.
    pub fn execute(&mut self, instruction: Row, memory: &mut Memory) {
        self.present_instruction = instruction;

        // All maths wraps around at 32 bits, like the real hardware. Going past the biggest
        // number rolls over to the most negative one, instead of crashing the emulator.

        // Split the instruction into its two parts.
        let which_row = (instruction & 0b11111) as usize;
        let which_instruction = (instruction >> 13) & 0b111;

        match which_instruction {
            // Jump: set the counter to the row's number.
            // The next row run is the one after that, since the counter moves forward 1 first.
            JUMP => self.counter = memory[which_row],
            // Jump forward: add the row's number to the counter.
            // A negative number moves the counter backwards, which is how programs loop.
            JUMP_FORWARD => self.counter = self.counter.wrapping_add(memory[which_row]),
            // Load: copy the row into the accumulator, flipped negative.
            LOAD => self.accumulator = memory[which_row].wrapping_neg(),
            // Store: copy the accumulator into the row.
            STORE => memory[which_row] = self.accumulator,
            // Subtract: take the row's number away from the accumulator.
            SUBTRACT | SUBTRACT_ALSO => {
                self.accumulator = self.accumulator.wrapping_sub(memory[which_row])
            }
            // Skip if negative: if the accumulator is below 0, move the counter forward
            // one extra row, so the next row gets jumped over.
            SKIP_IF_NEGATIVE => {
                if self.accumulator < 0 {
                    self.counter = self.counter.wrapping_add(1);
                }
            }
            // Stop: halt the machine.
            STOP => self.stopped = true,
            // Only 3 bits are used for the instruction number, so it can never be above 7.
            _ => unreachable!(),
        }
    }
}
