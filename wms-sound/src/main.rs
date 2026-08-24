use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};
use std::num::{NonZeroU16, NonZeroU32};

use emu6800::cpu::RegisterFileTrait;
use emucore::mem::MemoryIO;
use rodio::{buffer::SamplesBuffer, DeviceSinkBuilder, Player};
use wms_sound::{HarnessConfig, SoundHarness};

fn parse_byte(text: &str) -> Option<u8> {
    let text = text.trim();
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("$")) {
        u8::from_str_radix(hex, 16).ok()
    } else {
        text.parse().ok()
    }
}

fn parse_address(text: &str) -> Option<u16> {
    let text = text.trim();
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("$"))
        .unwrap_or(text);
    u16::from_str_radix(digits, 16).ok()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let fast = args.iter().any(|arg| arg == "--fast");
    let rom_path = args
        .iter()
        .skip(1)
        .find(|arg| arg.as_str() != "--fast")
        .ok_or("usage: wms-sound [--fast] <sound.bin>")?;
    let rom = std::fs::read(rom_path)?;
    let mut harness = if fast {
        SoundHarness::from_rom_fast(&rom, HarnessConfig::default())
    } else {
        SoundHarness::from_rom(&rom, HarnessConfig::default())
    };
    let mut breakpoints = BTreeSet::new();
    let mut recording: Option<String> = None;
    println!("WMS sound harness. Commands: sound <byte>, run <cycles>, replay <log> <cycles>, record <log>, check, break, clear, stack, trace, play, wav, status, quit");

    for line in io::stdin().lock().lines() {
        let line = line?;
        let mut words = line.split_whitespace();
        match words.next() {
            Some("sound") | Some("command") => match words.next().and_then(parse_byte) {
                Some(value) => {
                    harness.send_command(value);
                    println!("sent ${value:02x}");
                }
                None => println!("expected a byte, e.g. 0x19"),
            },
            Some("record") => match words.next() {
                Some(path) => { recording = Some(path.to_owned()); println!("recording PIA events to {path}"); }
                None => println!("usage: record output.log"),
            },
            Some("replay") => {
                let path = words.next();
                let cycles = words.next().and_then(|v| v.parse::<u64>().ok());
                match (path, cycles) {
                    (Some(path), Some(cycles)) => match SoundHarness::read_command_log(path) {
                        Ok(events) => match harness.replay_events(&events, cycles, false) {
                            Ok(now) => println!("replayed {} PIA events to cycle {now}", events.len()),
                            Err(error) => println!("CPU error: {error}"),
                        },
                        Err(error) => println!("replay file error: {error}"),
                    },
                    _ => println!("usage: replay input.log cycles"),
                }
            },
            Some("run") => match words.next().and_then(|v| v.parse::<u64>().ok()) {
                Some(cycles) => {
                    let target = harness.board.cycles + cycles;
                    let mut stopped = false;
                    while harness.board.cycles < target {
                        let pc = harness.board.cpu.regs.pc;
                        if breakpoints.contains(&pc) {
                            println!("breakpoint hit at ${pc:04x} (cycle {})", harness.board.cycles);
                            stopped = true;
                            break;
                        }
                        if let Err(error) = harness.board.step() {
                            println!("CPU error: {error}");
                            stopped = true;
                            break;
                        }
                    }
                    if !stopped { println!("ran to cycle {}", harness.board.cycles); }
                }
                None => println!("expected a cycle count"),
            },
            Some("break") => match words.next().and_then(parse_address) {
                Some(address) => { breakpoints.insert(address); println!("breakpoint set at ${address:04x}"); }
                None => println!("usage: break address"),
            },
            Some("clear") => match words.next() {
                Some("all") => { breakpoints.clear(); println!("breakpoints cleared"); }
                Some(address) => match parse_address(address) {
                    Some(address) => { breakpoints.remove(&address); println!("breakpoint cleared at ${address:04x}"); }
                    None => println!("usage: clear address|all"),
                },
                None => println!("breakpoints: {:?}", breakpoints),
            },
            Some("stack") => {
                let sp = harness.board.cpu.regs.sp;
                let start = sp.saturating_sub(8);
                for address in start..=sp.saturating_add(8) {
                    let value = harness.board.cpu.mem.inspect_byte(address as usize).unwrap_or(0);
                    println!("${address:04x}: ${value:02x}");
                }
            }
            Some("replay-wav") => {
                let log = words.next();
                let path = words.next();
                let cycles = words.next().and_then(|v| v.parse::<u64>().ok());
                match (log, path, cycles) {
                    (Some(log), Some(path), Some(cycles)) => match SoundHarness::read_command_log(log) {
                        Ok(events) => match harness.replay_events(&events, cycles, false) {
                            Ok(_) => match harness.write_wav(path, cycles) {
                                Ok(()) => println!("replayed {} events and wrote {path}", events.len()),
                                Err(error) => println!("WAV error: {error}"),
                            },
                            Err(error) => println!("CPU error: {error}"),
                        },
                        Err(error) => println!("replay file error: {error}"),
                    },
                    _ => println!("usage: replay-wav input.log output.wav cycles"),
                }
            },
            Some("peek") => match words.next().and_then(parse_address) {
                Some(address) => println!("${address:04x}: ${:02x}", harness.board.cpu.mem.inspect_byte(address as usize).unwrap_or(0)),
                None => println!("usage: peek address"),
            },
            Some("dump-ram") => match words.next() {
                Some(path) => {
                    let bytes: Vec<u8> = (0..0x100)
                        .map(|address| harness.board.cpu.mem.inspect_byte(address).unwrap_or(0))
                        .collect();
                    match std::fs::write(path, bytes) {
                        Ok(()) => println!("wrote RAM dump {path}"),
                        Err(error) => println!("RAM dump error: {error}"),
                    }
                }
                None => println!("usage: dump-ram output.bin"),
            },
            Some("dump-pia-trace") => match words.next() {
                Some(path) => match std::fs::File::create(path) {
                    Ok(mut file) => {
                        for access in harness.board.take_pia_accesses() {
                            let kind = if access.write { 'W' } else { 'R' };
                            writeln!(file, "{} {} {:04X} {:02X}", access.cycle, kind, access.address, access.value)?;
                        }
                        println!("wrote PIA trace {path}");
                    }
                    Err(error) => println!("PIA trace error: {error}"),
                },
                None => println!("usage: dump-pia-trace output.log"),
            },
            Some("check") => match words.next().and_then(|v| v.parse().ok()) {
                Some(cycles) => match harness.run_cycles_strict(cycles) {
                    Ok(now) => println!("strict check passed to cycle {now}"),
                    Err(error) => println!("strict check failed: {error}"),
                },
                None => println!("expected a cycle count"),
            },
            Some("trace") => {
                let cycles = words.next().and_then(|v| v.parse::<u64>().ok());
                let path = words.next();
                match (cycles, path) {
                    (Some(cycles), Some(path)) => match std::fs::File::create(path) {
                        Ok(file) => {
                            let mut out = file;
                            let target = harness.board.cycles + cycles;
                            let mut count = 0;
                            let result = (|| -> emu6800::cpu::CpuResult<()> {
                                while harness.board.cycles < target {
                                    let pc = harness.board.cpu.regs.pc();
                                    let opcode = harness.board.cpu.mem.inspect_byte(pc as usize).unwrap_or(0);
                                    let (a, b, x, sp, cc) = (harness.board.cpu.regs.a, harness.board.cpu.regs.b, harness.board.cpu.regs.x, harness.board.cpu.regs.sp, harness.board.cpu.regs.sr());
                                    writeln!(out, "{pc:04X}: opcode={opcode:02X} A={a:02X} B={b:02X} X={x:04X} SP={sp:04X} CC={cc:02X} cycle={}", harness.board.cycles).map_err(|error| emu6800::cpu::CpuErrKind::Effects(error.to_string()))?;
                                    harness.board.step()?;
                                    count += 1;
                                }
                                Ok(())
                            })();
                            match result {
                                Ok(()) => println!("wrote {path} ({count} instructions)"),
                                Err(error) => println!("trace CPU error: {error}"),
                            }
                        }
                        Err(error) => println!("trace file error: {error}"),
                    },
                    _ => println!("usage: trace cycles output.log"),
                }
            }
            Some("replay-trace") => {
                let log = words.next();
                let cycles = words.next().and_then(|v| v.parse::<u64>().ok());
                let path = words.next();
                match (log, cycles, path) {
                    (Some(log), Some(cycles), Some(path)) => match SoundHarness::read_command_log(log) {
                        Ok(events) => {
                            let result = (|| -> emu6800::cpu::CpuResult<usize> {
                                let end = harness.board.cycles + cycles;
                                let mut event_index = 0;
                                let mut count = 0;
                                let mut out = std::fs::File::create(path).map_err(|e| emu6800::cpu::CpuErrKind::Effects(e.to_string()))?;
                                while harness.board.cycles < end {
                                    while event_index < events.len() && events[event_index].cycle <= harness.board.cycles {
                                        harness.send_pia_value(events[event_index].pia_value);
                                        event_index += 1;
                                    }
                                    let pc = harness.board.cpu.regs.pc();
                                    let opcode = harness.board.cpu.mem.inspect_byte(pc as usize).unwrap_or(0);
                                    let cc = harness.board.cpu.regs.sr();
                                    let (a, b, x, sp) = (harness.board.cpu.regs.a, harness.board.cpu.regs.b, harness.board.cpu.regs.x, harness.board.cpu.regs.sp);
                                    harness.board.step()?;
                                    writeln!(out, "{pc:04X}: opcode={opcode:02X} A={a:02X} B={b:02X} X={x:04X} SP={sp:04X} CC={cc:02X} cycle={} hash={:016X}", harness.board.cycles, harness.board.state_hash()).map_err(|e| emu6800::cpu::CpuErrKind::Effects(e.to_string()))?;
                                    count += 1;
                                }
                                Ok(count)
                            })();
                            match result {
                                Ok(count) => println!("wrote {path} ({count} instructions)"),
                                Err(error) => println!("trace CPU error: {error}"),
                            }
                        }
                        Err(error) => println!("replay file error: {error}"),
                    },
                    _ => println!("usage: replay-trace input.log cycles output.log"),
                }
            }
            Some("wav") => {
                let path = words.next();
                let cycles = words.next().and_then(|v| v.parse().ok());
                match (path, cycles) {
                    (Some(path), Some(cycles)) => match harness.run_cycles(cycles) {
                        Ok(_) => match harness.write_wav(path, cycles) {
                            Ok(()) => println!("wrote {path}"),
                            Err(error) => println!("WAV error: {error}"),
                        },
                        Err(error) => println!("CPU error: {error}"),
                    },
                    _ => println!("usage: wav output.wav cycles"),
                }
            }
            Some("play") => match words.next().and_then(|v| v.parse().ok()) {
                Some(cycles) => match harness.run_cycles(cycles) {
                    Ok(_) => {
                        let samples = harness.render_audio(cycles);
                        let stream = DeviceSinkBuilder::open_default_sink()?;
                        let player = Player::connect_new(stream.mixer());
                        player.append(SamplesBuffer::new(
                            NonZeroU16::new(1).unwrap(),
                            NonZeroU32::new(harness.config.sample_rate).unwrap(),
                            samples,
                        ));
                        player.sleep_until_end();
                    }
                    Err(error) => println!("CPU error: {error}"),
                },
                None => println!("usage: play cycles"),
            },
            Some("status") => println!(
                "cycle={} pc=${:04x} x=${:04x} a=${:02x} b=${:02x} dac=${:02x} pia_pa=${:02x} pia_pb_in=${:02x} ddra=${:02x} cra=${:02x} pending_dac_events={}",
                harness.board.cycles,
                harness.board.cpu.regs.pc,
                harness.board.cpu.regs.x,
                harness.board.cpu.regs.a,
                harness.board.cpu.regs.b,
                harness.board.dac_value(),
                harness.board.cpu.mem.pia.port_a_output(),
                harness.board.cpu.mem.pia_input_b,
                harness.board.cpu.mem.pia.data_direction_a(),
                harness.board.cpu.mem.pia.control_a(),
                harness.board.pending_dac_events()
            ),
            Some("quit") | Some("exit") => {
                if let Some(path) = recording.take() {
                    match harness.write_command_log(&path) {
                        Ok(()) => println!("wrote PIA event log {path}"),
                        Err(error) => println!("record file error: {error}"),
                    }
                }
                break
            },
            Some("") | None => {}
            Some(_) => println!("commands: sound, run, replay, replay-wav, replay-trace, record, check, break, clear, trace, play, wav, status, quit"),
        }
        io::stdout().flush()?;
    }
    Ok(())
}
