use fixedstr;
use pico_args;
use serial2;
use anyhow::Result;
use anyhow::anyhow;
use std::io::Read;
use std::io::Write;
use rhexdump::rhexdumps;

// useful helper
macro_rules! quiet_eprintln {
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            eprintln!($($arg)*);
        }
    };
}

//--------------------------------------------------------------------------------------------------

const HELP: &str = "\
txrx - small utility to send and receive data on a serial port

txrx will send the data from the standard input to a serial port using specified
settings, will listen to reply, and hexdump the communication to stderr.

USAGE:
    txrx TTYDEV -b BAUDRATE [OPTIONS] [FLAGS]

    FLAGS:
        -h, --help              Prints help and exits

    OPTIONS:
        -b, --baud BAUDRATE     Baudrate of the port. Mandatory.

        -c, --cfg CONFIG        Three characters representing the configuration
                                of the port. Examples are: 8N1, 7E1, etc.
                                Optional. Default value is 8N1.

                                First character: number of data bits.
                                Permitted values are: 5, 6, 7, 8

                                Second character: a letter represendting parity:
                                Permitted values are:
                                    N - no parity
                                    E - even parity

                                Third character: number of stop bits.
                                Permitted values are: 1, 2

        -t, --timeout TIMEOUT   Wait for reply for specified amount of millseconds.
                                Optional. Default value is 1000 ms.

        -q, --quiet             Don't output anything to stderr.
                                Optional. Default behaviour is to print logs
                                to stderr.

        -s, --stdout            Output received data into the stdout. This way
                                incoming data can be piped into other processes
                                or saved to the file.

    ARGS:
        <TTYDEV>                Path to tty serial port device.
                                E.g.: /dev/ttyUSB0, /dev/ttyACM0 and such.
";

struct ToolArgs {
    port: fixedstr::str64,
    baud: u32,
    cfg: fixedstr::str8,
    quiet: bool,
    stdout: bool,
    timeout: u64,
}

fn parse_args() -> Result<ToolArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        eprintln!("{}", HELP);
        std::process::exit(0);
    }

    let args = ToolArgs {
        port: pargs.free_from_str()?,
        baud: pargs.value_from_str(["-b", "--baud" ])?,
        cfg: pargs.opt_value_from_str([ "-c", "--config" ])?.unwrap_or("8E1".into()),
        quiet: pargs.contains(["-q", "--quiet"]),
        timeout: pargs.opt_value_from_str([ "-t", "--timeout" ])?.unwrap_or(1000),
        stdout: pargs.contains(["-s", "--stdout"]),
    };

    Ok(args)
}

//--------------------------------------------------------------------------------------------------

fn main() -> Result<()> {
    let Ok(args) = parse_args() else { return Err(anyhow!("failed to parse args, aborting...")); };

    quiet_eprintln!(args.quiet, "tty: {}", args.port);
    quiet_eprintln!(args.quiet, "baud: {}", args.baud);
    quiet_eprintln!(args.quiet, "cfg: {}", args.cfg);

    let mut port = serial2::SerialPort::open(args.port.as_str(), args.baud)?;
    port.set_read_timeout(std::time::Duration::from_millis(args.timeout))?;

    let mut buf = Vec::new();
    buf.try_reserve(2048)?;

    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_end(&mut buf)?;

    if !args.quiet {
        quiet_eprintln!(args.quiet, "sending data:");
        let hexdump = rhexdumps!(&buf);
        quiet_eprintln!(args.quiet, "{}", hexdump);
    }

    port.write(&buf)?;
    buf.clear();

    quiet_eprintln!(args.quiet, "waiting for {} ms to complete read...", args.timeout);

    // TODO: error check: it should be always "Timeout", otherwise something is wrong
    let _ = port.read_to_end(&mut buf);

    let hexdump = rhexdumps!(&buf);
    quiet_eprintln!(args.quiet, "{}", hexdump);

    if args.stdout {
        std::io::stdout().write_all(&buf)?;
    }

    Ok(())
}
