#[cfg(test)]
mod tests {
    use mollusk_svm::{Mollusk, result::Check};
    use solana_sdk::instruction::Instruction;
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_hello_world() {
        let program_id = Pubkey::new_from_array([
            0x1e, 0x3c, 0xd6, 0x28, 0x43, 0x80, 0x94, 0x0e, 0x08, 0x62, 0x4c, 0xb8, 0x33, 0x8b,
            0x77, 0xdc, 0x33, 0x25, 0x75, 0xd1, 0x5f, 0xa3, 0x9a, 0x0f, 0x1d, 0xf1, 0x5e, 0xe0,
            0x8f, 0xb8, 0x23, 0xee,
        ]);

        let instruction = Instruction::new_with_bytes(program_id, &[], vec![]);

        let mollusk = Mollusk::new(&program_id, "abort");

        mollusk.process_and_validate_instruction(
            &instruction,
            &[],
            // Expect the program to fail with error 0x1.
            &[Check::err(ProgramError::Custom(1))],
        );
    }
}
