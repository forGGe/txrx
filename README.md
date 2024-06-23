# txrx

Sometimes all you need is send bunch of bytes into the serial using specific settings and print
the response after some time. The rxtx utility does exactly that.

## Building

Rust freaks know what to do :)

For rest & uninitiated:

 * `git clone` this repo
 * `cargo build` in the project dir
 * check `target/debug` dir for the output binary.

## Usage

```
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
```

## Typical Scenarios

### Basic Usage

Send/receive data using default settings:

```sh
echo "Hello, serial world!" | txrx /dev/ttyUSB0 -b 9600`
```

### Saving Data to File

You can send the serial port incoming data somewhere else:

```sh
echo "Hello, serial world!" | txrx /dev/ttyUSB0 -b 9600 -s > /tmp/response.bin
```

### Piping Responses

Note the `-q` flag to supress unnecessary chatter:

```sh
echo "Hello, serial world!" | txrx /dev/ttyUSB0 -b 9600 -q -s | grep "needle"
```

## License

This project is licensed under the MIT License.