use Module;
use socket::Socket;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Unit {
    RAW,
    VOLTS,
}

impl ::std::convert::Into<String> for Unit {
    fn into(self) -> String {
        let s = match self {
            Unit::RAW => "RAW",
            Unit::VOLTS => "VOLTS",
        };

        String::from(s)
    }
}

impl ::std::str::FromStr for Unit  {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RAW" => Ok(Unit::RAW),
            "VOLTS" => Ok(Unit::VOLTS),
            unit => Err(format!("Unknow unit '{}'", unit)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Format {
    FLOAT,
    ASCII,
}

impl ::std::convert::Into<String> for Format {
    fn into(self) -> String {
        let s = match self {
            Format::FLOAT => "FLOAT",
            Format::ASCII => "ASCII",
        };

        String::from(s)
    }
}

#[derive(Clone)]
pub struct Data {
    socket: ::std::cell::RefCell<Socket>,
}

impl ::Module for Data {
    fn get_socket<'a>(&'a self) -> ::std::cell::RefMut<'a, ::socket::Socket> {
        self.socket.borrow_mut()
    }
}

impl Data {
    pub fn new(socket: Socket) -> Self {
        Data {
            socket: ::std::cell::RefCell::new(socket),
        }
    }

    /**
     * Returns current position of write pointer.
     */
    pub fn get_write_pointer(&self) -> Result<u32, <u32 as ::std::str::FromStr>::Err> {
        self.send("ACQ:WPOS?");

        self.receive()
            .parse()
    }

    /**
     * Returns position where trigger event appeared.
     */
    pub fn get_trigger_position(&self) -> Result<u32, <u32 as ::std::str::FromStr>::Err> {
        self.send("ACQ:TPOS?");

        self.receive()
            .parse()
    }

    /**
     * Selects units in which acquired data will be returned.
     */
    pub fn set_units(&self, unit: Unit) {
        self.send(format!("ACQ:DATA:UNITS {}", Into::<String>::into(unit)));
    }

    /**
     * Get units in which acquired data will be returned.
     */
    pub fn get_units(&self) -> Result<Unit, String> {
        self.send("ACQ:DATA:UNITS?");

        self.receive()
            .parse()
    }

    /**
     * Selects format acquired data will be returned.
     */
    pub fn set_format(&self, format: Format) {
        self.send(format!("ACQ:DATA:FORMAT {}", Into::<String>::into(format)));
    }

    /**
     * Read samples from start to stop position.
     *
     * start = {0,1,...,16384}
     * stop_pos = {0,1,...16384}
     */
    pub fn read_slice(&self, source: ::acquire::Source, start: u16, end: u16) -> String {
        self.send(format!("ACQ:{}:DATA:STA:END? {},{}", Into::<String>::into(source), start, end));

        self.receive()
    }

    /**
     * Read `m` samples from start position on.
     */
    pub fn read(&self, source: ::acquire::Source, start: u16, len: u32) -> String {
        self.send(format!("ACQ:{}:DATA:STA:N? {},{}", Into::<String>::into(source), start, len));

        self.receive()
    }

    /**
     * Read full buf.
     *
     * Size starting from oldest sample in buffer (this is first sample after
     * trigger delay). Trigger delay by default is set to zero (in samples or
     * in seconds). If trigger delay is set to zero it will read full buf.
     * Size starting from trigger.
     */
    pub fn read_all(&self, source: ::acquire::Source) -> String {
        self.send(format!("ACQ:{}:DATA?", Into::<String>::into(source)));

        self.receive()
    }

    /**
     * Read m samples after trigger delay, starting from oldest sample in buffer
     * (this is first sample after trigger delay).
     *
     * Trigger delay by default is set to zero (in samples or in seconds). If
     * trigger delay is set to zero it will read m samples starting from trigger.
     */
    pub fn read_oldest(&self, source: ::acquire::Source, len: u32) -> String {
        self.send(format!("ACQ:{}:DATA:OLD:N? {}", Into::<String>::into(source), len));

        self.receive()
    }

    /**
     * Read ``m`` samples before trigger delay.
     *
     * Trigger delay by default is set to zero (in samples or in seconds). If
     * trigger delay is set to zero it will read m samples before trigger.
     */
    pub fn read_latest(&self, source: ::acquire::Source, len: u32) -> String {
        self.send(format!("ACQ:{}:DATA:LAT:N? {}", Into::<String>::into(source), len));

        self.receive()
    }

    /**
     * Returns buffer size.
     */
    pub fn buffer_size(&self) -> Result<u32, <u32 as ::std::str::FromStr>::Err> {
        self.send("ACQ:BUF:SIZE?");

        self.receive()
            .parse()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_get_write_pointer() {
        let (_, data) = create_data();

        assert_eq!(data.get_write_pointer(), Ok(1024));
    }

    #[test]
    fn test_get_write_pointer_at_trigger() {
        let (_, data) = create_data();

        assert_eq!(data.get_trigger_position(), Ok(512));
    }

    #[test]
    fn test_set_units() {
        let (rx, data) = create_data();

        data.set_units(::data::Unit::VOLTS);
        assert_eq!("ACQ:DATA:UNITS VOLTS\r\n", rx.recv().unwrap());
    }

    #[test]
    fn test_get_units() {
        let (_, data) = create_data();

        assert_eq!(data.get_units(), Ok(::data::Unit::RAW));
    }

    #[test]
    fn test_set_format() {
        let (rx, data) = create_data();

        data.set_format(::data::Format::FLOAT);
        assert_eq!("ACQ:DATA:FORMAT FLOAT\r\n", rx.recv().unwrap());
    }

    #[test]
    fn test_read_slice() {
        let (_, data) = create_data();

        assert_eq!(data.read_slice(::acquire::Source::IN1, 10, 13), "{123,231,-231}");
    }

    #[test]
    fn test_read() {
        let (_, data) = create_data();

        assert_eq!(data.read(::acquire::Source::IN1, 10, 3), "{1.2,3.2,-1.2}");
    }

    #[test]
    fn test_read_all() {
        let (_, data) = create_data();

        assert_eq!(data.read_all(::acquire::Source::IN1), "{1.2,3.2,-1.2}");
    }

    #[test]
    fn test_read_oldest() {
        let (_, data) = create_data();

        assert_eq!(data.read_oldest(::acquire::Source::IN1, 2), "{3.2,-1.2}");
    }

    #[test]
    fn test_read_latest() {
        let (_, data) = create_data();

        assert_eq!(data.read_latest(::acquire::Source::IN1, 2), "{1.2,3.2}");
    }

    #[test]
    fn test_buffer_size() {
        let (_, data) = create_data();

        assert_eq!(data.buffer_size(), Ok(16384));
    }

    fn create_data() -> (::std::sync::mpsc::Receiver<String>, ::data::Data) {
        let (addr, rx) = ::test::launch_server();
        let socket = ::socket::Socket::new(addr);

        (rx, ::data::Data::new(socket))
    }
}
