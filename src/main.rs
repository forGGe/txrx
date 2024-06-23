use anyhow::anyhow;
use anyhow::Result;
use fixedstr;
use pico_args;
use rhexdump::rhexdumps;
use serial2;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

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

#[derive(Clone)]
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
        baud: pargs.value_from_str(["-b", "--baud"])?,
        cfg: pargs
            .opt_value_from_str(["-c", "--config"])?
            .unwrap_or("8E1".into()),
        quiet: pargs.contains(["-q", "--quiet"]),
        timeout: pargs
            .opt_value_from_str(["-t", "--timeout"])?
            .unwrap_or(1000),
        stdout: pargs.contains(["-s", "--stdout"]),
    };

    if args.cfg.len() != 3 {
        return Err(pico_args::Error::ArgumentParsingFailed {
            cause: "invalid port config string".to_string(),
        });
    }

    Ok(args)
}

impl serial2::IntoSettings for ToolArgs {
    fn apply_to_settings(self, settings: &mut serial2::Settings) -> std::io::Result<()> {
        // panic is justified - parse_args() suppose to check for the string size

        match self.cfg.chars().nth(0).unwrap() {
            '5' => settings.set_char_size(serial2::CharSize::Bits5),
            '6' => settings.set_char_size(serial2::CharSize::Bits6),
            '7' => settings.set_char_size(serial2::CharSize::Bits7),
            '8' => settings.set_char_size(serial2::CharSize::Bits8),
            _ => return Err(std::io::Error::from(ErrorKind::InvalidInput)),
        }

        match self.cfg.chars().nth(1).unwrap() {
            'E' => settings.set_parity(serial2::Parity::Even),
            'N' => settings.set_parity(serial2::Parity::None),
            'O' => settings.set_parity(serial2::Parity::Odd),
            _ => return Err(std::io::Error::from(ErrorKind::InvalidInput)),
        }

        match self.cfg.chars().nth(2).unwrap() {
            '1' => settings.set_stop_bits(serial2::StopBits::One),
            '2' => settings.set_stop_bits(serial2::StopBits::Two),
            _ => return Err(std::io::Error::from(ErrorKind::InvalidInput)),
        }

        settings.set_baud_rate(self.baud)?;
        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------

fn main() -> Result<()> {
    let Ok(args) = parse_args() else {
        return Err(anyhow!("failed to parse args, aborting..."));
    };

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

    quiet_eprintln!(
        args.quiet,
        "waiting for {} ms to complete read...",
        args.timeout
    );

    // TODO: error check: it should be always "Timeout", otherwise something is wrong
    let _ = port.read_to_end(&mut buf);

    let hexdump = rhexdumps!(&buf);
    quiet_eprintln!(args.quiet, "{}", hexdump);

    if args.stdout {
        std::io::stdout().write_all(&buf)?;
    }

    Ok(())
}
