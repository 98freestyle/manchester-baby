// Programs that can be loaded into the Baby's RAM.

use crate::memory::Memory;

// The 1948 highest factor program, in the amended version Geoff Tootill wrote in July 1948
// (Tom Kilburn's original ran on 21 June 1948). It finds the biggest number that divides 262,144 exactly.
// It tries every number from 262,143 downwards. For each one, it subtracts it over and over
// to see if it lands exactly on 0. The first one that does is the answer.
pub fn load_1948(memory: &mut Memory) {
    // Setup.
    memory[1] = 16408; // load row 24
    memory[2] = 24602; // store row 26
    memory[3] = 16410; // load row 26
    memory[4] = 24603; // store row 27
    memory[5] = 16407; // load row 23

    // Keep subtracting the number being tried until the accumulator goes negative.
    memory[6] = 32795; // subtract row 27
    memory[7] = 49152; // skip if negative
    memory[8] = 8212; // jump forward row 20

    // Work out the remainder. If it's 0, the number divides exactly, so stop.
    memory[9] = 32794; // subtract row 26
    memory[10] = 24601; // store row 25
    memory[11] = 16409; // load row 25
    memory[12] = 49152; // skip if negative
    memory[13] = 57344; // stop

    // Not a divisor. Try the next number down and start again.
    memory[14] = 16410; // load row 26
    memory[15] = 32789; // subtract row 21
    memory[16] = 24603; // store row 27
    memory[17] = 16411; // load row 27
    memory[18] = 24602; // store row 26
    memory[19] = 22; // jump row 22

    // Numbers the program uses.
    memory[20] = -3; // how far jump forward moves the counter
    memory[21] = 1; // used to step down to the next number
    memory[22] = 4; // where jump sends the counter
    memory[23] = -262_144; // the number to factor, stored negative
    memory[24] = 262_143; // the first number to try
    // Row 25 holds the remainder while running.
    // Row 26 holds the number being tried, negative.
    // Row 27 holds the number being tried. When the program stops, this is the answer.
}

#[cfg(test)]
mod tests {
    use crate::cpu::Cpu;
    use crate::memory;

    // Runs the 1948 program start to finish and checks it gets the same answer the real Baby did.
    #[test]
    fn program_1948_finds_131072() {
        let mut memory = memory::empty();
        super::load_1948(&mut memory);
        let mut cpu = Cpu::new();
        let mut instructions_run = 0;

        while !cpu.stopped {
            cpu.step(&mut memory);
            instructions_run = instructions_run + 1;
        }

        assert_eq!(memory[27], 131_072);
        assert_eq!(instructions_run, 2_097_154);
    }
}
