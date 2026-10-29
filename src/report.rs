use std::error::Error;
use crate::{alloc, Run};
use alloc::Event;
use argh::FromArgs;
use bytesize::ByteSize;
use std::path::PathBuf;
use textplots::{Chart, Plot, Shape};

#[derive(FromArgs)]
/// Analyze report
#[argh(subcommand, name = "report")]
pub struct Report {
    #[argh(positional)]
    path: PathBuf,
}

trait Delta {
    fn delta(self) -> isize;
}

impl Delta for Event {
    fn delta(self) -> isize {
        match self {
            //the enum value should match the record key
            Event::Alloc { size, .. } => size as isize,
            Event::Freed { size, .. } => -(size as isize),
        }
    }
}

impl Run for Report {
     fn run(self) -> Result<(), Box<dyn Error>> {
        let bytes = std::fs::read(&self.path)?;
        let content = if bytes.starts_with(&[0xFF, 0xFE]) {
            let u16s: Vec<u16> = bytes[2..]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            char::decode_utf16(u16s)
                .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect::<String>()
        } else if bytes.starts_with(&[0xFE, 0xFF]) {
            let u16s: Vec<u16> = bytes[2..]
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            char::decode_utf16(u16s)
                .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect::<String>()
        } else if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            String::from_utf8_lossy(&bytes[3..]).into_owned()
        } else {
            match std::str::from_utf8(&bytes) {
                Ok(s) => s.to_string(),
                Err(_) if bytes.len() >= 2 && bytes[1] == 0 => {
                    let u16s: Vec<u16> = bytes
                        .chunks_exact(2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .collect();
                    char::decode_utf16(u16s)
                        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                        .collect::<String>()
                }
                Err(_) => String::from_utf8_lossy(&bytes).into_owned(),
            }
        };

        let mut events: Vec<Event> = Default::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let ev: Event = serde_json::from_str(line)?;
            events.push(ev);
        }
        println!("found {} events", events.len());

        let mut points = vec![];
        let mut curr_bytes = 0;
        let mut peak_bytes = 0;
        let mut alloc_events = 0;
        let mut alloc_bytes = 0;
        let mut freed_events = 0;
        let mut freed_bytes = 0;
        for (i, ev) in events.iter().copied().enumerate() {
            curr_bytes += ev.delta();
            points.push((i as f32, curr_bytes as f32));

            if peak_bytes < curr_bytes {
                peak_bytes = curr_bytes;
            }
            match ev {
                Event::Alloc { size, .. } => {
                    alloc_events += 1;
                    alloc_bytes += size;
                }
                Event::Freed { size, .. } => {
                    freed_events += 1;
                    freed_bytes += size;
                }
            }
        }
        Chart::new(120, 80, 0.0, points.len() as f32)
            .lineplot(&Shape::Steps(&points[..]))
            .nice();

        println!("     total events | {}", events.len());
        println!("      peak bytes  | {}", ByteSize::kib(peak_bytes as _));
        println!("     ----------------------------");
        println!("     alloc events | {}", alloc_events);
        println!("     alloc bytes  | {}", ByteSize::kib(alloc_bytes as _));
        println!("     ----------------------------");
        println!("     freed events | {}", freed_events);
        println!("     freed bytes  | {}", ByteSize(freed_bytes as _));
        Ok(())
     }
}