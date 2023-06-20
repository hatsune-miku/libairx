use std::io;

pub trait SendDataWithRetry {
    fn send_data_with_retry(&mut self, data: &Vec<u8>) -> Result<usize, io::Error>;
}

pub trait ReadDataWithRetry {
    fn read_data_with_retry(&mut self) -> Result<Vec<u8>, io::Error>;
}
