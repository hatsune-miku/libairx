use std::fmt;
use std::fmt::{Debug, Formatter};

pub enum FileSendingStatus {
    Requested,
    Rejected,
    Accepted,
    InProgress,
    CancelledBySender,
    CancelledByReceiver,
    Completed,
    Error,
}

impl Debug for FileSendingStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FileSendingStatus::Requested => write!(f, "Requested"),
            FileSendingStatus::Rejected => write!(f, "Rejected"),
            FileSendingStatus::Accepted => write!(f, "Accepted"),
            FileSendingStatus::InProgress => write!(f, "InProgress"),
            FileSendingStatus::CancelledBySender => write!(f, "CancelledBySender"),
            FileSendingStatus::CancelledByReceiver => write!(f, "CancelledByReceiver"),
            FileSendingStatus::Completed => write!(f, "Completed"),
            FileSendingStatus::Error => write!(f, "Error"),
        }
    }
}

pub enum FileSendingStatusError {
    InvalidStatus,
}

impl Debug for FileSendingStatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid status.")
    }
}

impl FileSendingStatus {
    pub fn to_u8(&self) -> u8 {
        match self {
            FileSendingStatus::Requested => 1,
            FileSendingStatus::Rejected => 2,
            FileSendingStatus::Accepted => 3,
            FileSendingStatus::InProgress => 4,
            FileSendingStatus::CancelledBySender => 5,
            FileSendingStatus::CancelledByReceiver => 6,
            FileSendingStatus::Completed => 7,
            FileSendingStatus::Error => 8,
        }
    }

    pub fn from_u8(value: u8) -> Result<FileSendingStatus, FileSendingStatusError> {
        match value {
            1 => Ok(FileSendingStatus::Requested),
            2 => Ok(FileSendingStatus::Rejected),
            3 => Ok(FileSendingStatus::Accepted),
            4 => Ok(FileSendingStatus::InProgress),
            5 => Ok(FileSendingStatus::CancelledBySender),
            6 => Ok(FileSendingStatus::CancelledByReceiver),
            7 => Ok(FileSendingStatus::Completed),
            8 => Ok(FileSendingStatus::Error),
            _ => Err(FileSendingStatusError::InvalidStatus),
        }
    }
}

pub struct FileSendingPacket {
    file_id: u8,
    progress: u64,
    total: u64,
    status: FileSendingStatus,
}

impl FileSendingPacket {
    pub fn new(
        file_id: u8,
        progress: u64,
        total: u64,
        status: FileSendingStatus,
    ) -> FileSendingPacket {
        FileSendingPacket {
            file_id,
            progress,
            total,
            status,
        }
    }

    pub fn file_id(&self) -> u8 {
        self.file_id
    }

    pub fn progress(&self) -> u64 {
        self.progress
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn status(&self) -> &FileSendingStatus {
        &self.status
    }

    pub fn set_status(&mut self, status: FileSendingStatus) {
        self.status = status;
    }
}
