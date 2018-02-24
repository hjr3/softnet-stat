/*  Parser for /proc/softnet_stats file
 *  Copyright (C) 2016  Herman J. Radtke III <herman@hermanradtke.com>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#[macro_use]
extern crate nom;
extern crate getopts;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use nom::{IResult, Err, Context, space, hex_u32, line_ending};
use nom::types::CompleteByteSlice;

use std::io;
use std::fs::File;

use getopts::Options;
use std::env;

/// Network data processing statistics
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct SoftnetStat {
    /// The number of network frames processed.
    ///
    /// This can be more than the total number of network frames received if
    /// you are using ethernet bonding. There are cases where the ethernet
    /// bonding driver will trigger network data to be re-processed, which
    /// would increment the processed count more than once for the same packet.
    pub processed: u32,

    /// The number of network frames dropped because there was no room on the processing queue.
    pub dropped: u32,

    /// The number of times the `net_rx_action` loop terminated because the budget was consumed or
    /// the time limit was reached, but more work could have been.
    pub time_squeeze: u32,

    /// The number of times a collision occurred when trying to obtain a device lock
    /// when transmitting packets.
    pub cpu_collision: u32,

    /// The number of times this CPU has been woken up to process packets via an Inter-processor Interrupt.
    ///
    /// Support was added in kernel v2.6.36
    pub received_rps: Option<u32>,

    /// The number of times the flow limit has been reached.
    ///
    /// Flow limiting is an optional Receive Packet Steering feature.
    /// Support was added in kernel v3.11
    pub flow_limit_count: Option<u32>,
}

//named!(parse_softnet_stats<CompleteByteSlice, Vec<SoftnetStat>>,
fn parse_softnet_stats(input: CompleteByteSlice) -> IResult<CompleteByteSlice, Vec<SoftnetStat>> {
    many1!(input, parse_softnet_line)
}

//named!(parse_softnet_line<CompleteByteSlice, SoftnetStat>,
fn parse_softnet_line(input: CompleteByteSlice) -> IResult<CompleteByteSlice, SoftnetStat> {
    let i = input.0.clone();
    match do_parse!(
        i,
        processed: hex_u32 >>
        space >>
        dropped: hex_u32 >>
        space >>
        time_squeeze: hex_u32 >>
        space >>
        hex_u32 >>
        space >>
        hex_u32 >>
        space >>
        hex_u32 >>
        space >>
        hex_u32 >>
        space >>
        hex_u32 >>
        space >>
        cpu_collision: hex_u32 >>
        received_rps: opt!(
            do_parse!(
                opt!(space) >>
                v: hex_u32 >>

                (v)
            )
        ) >>
        flow_limit_count: opt!(
            do_parse!(
                opt!(space) >>
                v: hex_u32 >>

                (v)
            )
        ) >>
        line_ending >>

        (SoftnetStat {
            processed: processed,
            dropped: dropped,
            time_squeeze: time_squeeze,
            cpu_collision: cpu_collision,
            received_rps: received_rps,
            flow_limit_count: flow_limit_count,
        })
    ) {
        Ok((remaining, value)) => Ok((CompleteByteSlice(remaining), value)),
        Err(Err::Incomplete(needed)) => Err(Err::Incomplete(needed)),
        Err(Err::Error(Context::Code(input, code))) => {
            Err(Err::Error(Context::Code(CompleteByteSlice(input), code)))
        }
        Err(Err::Failure(Context::Code(input, code))) => {
            Err(Err::Failure(Context::Code(CompleteByteSlice(input), code)))
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("j", "json", "use json output");
    opts.optflag("p", "prometheus", "use prometheus output");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("s", "stdin", "read from stdin");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!("Failed to parse options - {}", e),
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let file = "/proc/net/softnet_stat";

    let raw = if matches.opt_present("s") {
        let handle = io::stdin();
        read_proc_file(handle).expect("Failed to read proc from stdin")
    } else {
        let handle = File::open(file).expect("Failed to open file");
        read_proc_file(handle).expect("Failed to read proc from file")
    };

    let stats = match parse_softnet_stats(CompleteByteSlice(&raw)) {
        Ok((_, value)) => value,
        Err(Err::Incomplete(needed)) => panic!("{} is in an unsupported format. Needed: {:?}", file, needed),
        Err(Err::Error(e)) | Err(Err::Failure(e)) => panic!("Error while parsing {}: {:?}", file, e),
    };

    if matches.opt_present("j") {
        json(&stats);
    } else if matches.opt_present("p") {
        prometheus(&stats);
    } else {
        print(&stats, 15);
    }

}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn read_proc_file<R>(mut handle: R) -> io::Result<Vec<u8>> where R: io::Read {
    let mut buf = vec![];
    try!(handle.read_to_end(&mut buf));

    Ok(buf)
}

fn print(stats: &Vec<SoftnetStat>, spacer: usize) {
    println!("{:<spacer$}{:<spacer$}{:<spacer$}{:<spacer$}{:<spacer$}{:<spacer$}{:<spacer$}",
             "Cpu",
             "Processed",
             "Dropped",
             "Time Squeezed",
             "Cpu Collision",
             "Received RPS",
             "Flow Limit Count",
             spacer = spacer);

    for (i, stat) in stats.iter().enumerate() {
        println!("{:<spacer$}{:<spacer$}{:<spacer$}{:<spacer$}{:<spacer$}{:<spacer$}{:<spacer$}",
                 i,
                 stat.processed,
                 stat.dropped,
                 stat.time_squeeze,
                 stat.cpu_collision,
                 stat.received_rps.unwrap_or_default(),
                 stat.flow_limit_count.unwrap_or_default(),
                 spacer = spacer);
    }
}

fn json(stats: &Vec<SoftnetStat>) {
    let data = serde_json::to_string(&stats).expect("Failed to encode stats into json format");
    println!("{}", data);
}

fn prometheus(stats: &Vec<SoftnetStat>) {
    for (i, stat) in stats.iter().enumerate() {
        println!("softnet_frames_processed{{cpu=\"cpu{}\"}} {}", i, stat.processed);
        println!("softnet_frames_dropped{{cpu=\"cpu{}\"}} {}", i, stat.dropped);
        println!("softnet_time_squeeze{{cpu=\"cpu{}\"}} {}", i, stat.time_squeeze);
        println!("softnet_cpu_collisions{{cpu=\"cpu{}\"}} {}", i, stat.cpu_collision);
        println!("softnet_received_rps{{cpu=\"cpu{}\"}} {}", i, stat.received_rps.unwrap_or_default());
        println!("softnet_flow_limit_count{{cpu=\"cpu{}\"}} {}", i, stat.flow_limit_count.unwrap_or_default());

    }
}

#[test]
fn test_parse_softnet_line() {
    use nom::AsBytes;

    let pwd = env!("CARGO_MANIFEST_DIR");
    let file = format!("{}/tests/wtf", pwd);
    let handle = File::open(file).unwrap();
    let raw = read_proc_file(handle).unwrap();

    match parse_softnet_line(CompleteByteSlice(&raw)) {
        Ok((remaining, value)) => {
            assert_eq!(0, remaining.as_bytes().len());
            assert_eq!(SoftnetStat {
                processed: 1842008611,
                dropped: 0,
                time_squeeze: 1,
                cpu_collision: 0,
                received_rps: None,
                flow_limit_count: None,
            }, value);
        },
        Err(Err::Incomplete(needed)) => panic!("Unsupported format. Needed: {:?}", needed),
        Err(Err::Error(e)) | Err(Err::Failure(e)) => panic!("Error while parsing: {:?}", e),
    }
}

#[test]
fn test_parse_softnet_stats() {
    let pwd = env!("CARGO_MANIFEST_DIR");
    let files = vec![
        format!("{}/tests/wtf", pwd),
        //format!("{}/tests/proc-net-softnet_stat-2_6_32", pwd),
        //format!("{}/tests/proc-net-softnet_stat-2_6_36", pwd),
        //format!("{}/tests/proc-net-softnet_stat-3_11", pwd),
    ];

    for file in files.iter() {
        let handle = File::open(file).unwrap();
        let raw = read_proc_file(handle).unwrap();

        match parse_softnet_stats(CompleteByteSlice(&raw)) {
            Ok((_, _)) => {},
            Err(Err::Incomplete(needed)) => panic!("{} is in an unsupported format. Needed: {:?}", file, needed),
            Err(Err::Error(e)) | Err(Err::Failure(e)) => panic!("Error while parsing {}: {:?}", file, e),
        }
    }
}
