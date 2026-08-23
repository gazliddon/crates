use std::{env, fs, path::PathBuf};

use stargate_emu::{StargateInput, StargateMachine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let rom_dir = PathBuf::from(
        args.next()
            .ok_or("usage: stargate-frame <rom-directory> <output.ppm> [instructions]")?,
    );
    let output = PathBuf::from(
        args.next()
            .ok_or("usage: stargate-frame <rom-directory> <output.ppm> [instructions]")?,
    );
    let instructions = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(5_000_000usize);
    let pulse_advance = args.next().as_deref() == Some("--advance");
    let mut machine = StargateMachine::from_rom_dir(rom_dir)?;
    machine.reset()?;
    if pulse_advance {
        machine.run_instructions(instructions / 2)?;
        machine.set_input(StargateInput {
            advance: true,
            ..Default::default()
        });
        machine.run_instructions(2_000)?;
        machine.set_input(StargateInput::default());
        machine.run_instructions(instructions - instructions / 2 - 2_000)?;
    } else {
        machine.run_instructions(instructions)?;
    }
    let rgba = machine.video_rgba_visible();
    let mut ppm = format!("P6\n292 240\n255\n").into_bytes();
    for pixel in rgba.chunks_exact(4) {
        ppm.extend_from_slice(&pixel[..3]);
    }
    fs::write(output, ppm)?;
    eprintln!(
        "dumped {} instructions at PC=${:04x}",
        machine.instructions, machine.regs.pc
    );
    eprintln!("palette: {:02x?}", machine.palette());
    Ok(())
}
