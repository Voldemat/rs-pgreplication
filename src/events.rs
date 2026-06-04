#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum PrimaryEventType {
    XLogData = b'w',
    PrimaryKeepaliveMessage = b'k',
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum StandbyEventType {
    StandbyStatusUpdate = b'r',
    HotStandbyFeedbackMessage = b'h',
}

impl StandbyEventType {
    pub fn from_char(c: u8) -> Option<StandbyEventType> {
        if c == StandbyEventType::StandbyStatusUpdate as u8 {
            Some(StandbyEventType::StandbyStatusUpdate)
        } else if c == StandbyEventType::HotStandbyFeedbackMessage as u8 {
            Some(StandbyEventType::HotStandbyFeedbackMessage)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct XLogData<'a> {
    pub message_wal_start: i64,
    pub server_wal_end: i64,
    pub sent_at_unix_timestamp: i64,
    pub wal_data: &'a [u8],
}

impl<'a> XLogData<'a> {
    pub const MIN_SIZE: usize = std::mem::size_of::<i64>() * 3;

    pub fn get_network_buffer_size(self: &Self) -> usize {
        Self::MIN_SIZE + self.wal_data.len()
    }

    pub fn to_network_buffer(self: &Self, buffer: &'a mut [u8]) {
        assert!(buffer.len() == self.get_network_buffer_size());
        buffer[0..8].copy_from_slice(&self.message_wal_start.to_be_bytes());
        buffer[8..16].copy_from_slice(&self.server_wal_end.to_be_bytes());
        buffer[16..24]
            .copy_from_slice(&self.sent_at_unix_timestamp.to_be_bytes());

        // Append the wal_data payload
        buffer[24..24 + self.wal_data.len()].copy_from_slice(self.wal_data);
    }

    pub fn from_network_buffer(
        buffer: &'a [u8],
    ) -> Result<XLogData<'a>, String> {
        if buffer.len() < Self::MIN_SIZE {
            return Err(format!(
                "Buffer too small. Expected at least {} bytes, got {}",
                Self::MIN_SIZE,
                buffer.len()
            ));
        }

        Ok(XLogData {
            message_wal_start: i64::from_be_bytes(
                buffer[0..8].try_into().unwrap(),
            ),
            server_wal_end: i64::from_be_bytes(
                buffer[8..16].try_into().unwrap(),
            ),
            sent_at_unix_timestamp: i64::from_be_bytes(
                buffer[16..24].try_into().unwrap(),
            ),
            wal_data: &buffer[24..],
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrimaryKeepaliveMessage {
    pub server_wal_end: i64,
    pub sent_at_unix_timestamp: i64,
    pub reply_requested: bool,
}

impl PrimaryKeepaliveMessage {
    pub const SIZE: usize =
        std::mem::size_of::<i64>() * 2 + std::mem::size_of::<bool>();

    pub fn to_network_buffer(self: &Self, buffer: &mut [u8; Self::SIZE]) {
        buffer[0..8].copy_from_slice(&self.server_wal_end.to_be_bytes());
        buffer[8..16]
            .copy_from_slice(&self.sent_at_unix_timestamp.to_be_bytes());
        buffer[16] = self.reply_requested as u8;
    }

    pub fn from_network_buffer(
        buffer: &[u8; Self::SIZE],
    ) -> Result<PrimaryKeepaliveMessage, String> {
        Ok(PrimaryKeepaliveMessage {
            server_wal_end: i64::from_be_bytes(
                buffer[0..8].try_into().unwrap(),
            ),
            sent_at_unix_timestamp: i64::from_be_bytes(
                buffer[8..16].try_into().unwrap(),
            ),
            reply_requested: buffer[16] != 0,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StandbyStatusUpdate {
    pub written_wal_position: i64,
    pub flushed_wal_position: i64,
    pub applied_wal_position: i64,
    pub sent_at_unix_timestamp: i64,
    pub reply_requested: bool,
}

impl StandbyStatusUpdate {
    pub const SIZE: usize =
        std::mem::size_of::<i64>() * 4 + std::mem::size_of::<u8>();

    pub fn from_network_buffer(buffer: &[u8; Self::SIZE]) -> Self {
        StandbyStatusUpdate {
            written_wal_position: i64::from_be_bytes(
                buffer[0..8].try_into().unwrap(),
            ),
            flushed_wal_position: i64::from_be_bytes(
                buffer[8..16].try_into().unwrap(),
            ),
            applied_wal_position: i64::from_be_bytes(
                buffer[16..24].try_into().unwrap(),
            ),
            sent_at_unix_timestamp: i64::from_be_bytes(
                buffer[24..32].try_into().unwrap(),
            ),
            reply_requested: buffer[32] != 0,
        }
    }

    pub fn to_network_buffer(&self, buffer: &mut [u8; Self::SIZE]) {
        buffer[0..8].copy_from_slice(&self.written_wal_position.to_be_bytes());
        buffer[8..16].copy_from_slice(&self.flushed_wal_position.to_be_bytes());
        buffer[16..24]
            .copy_from_slice(&self.applied_wal_position.to_be_bytes());
        buffer[24..32]
            .copy_from_slice(&self.sent_at_unix_timestamp.to_be_bytes());
        buffer[32] = if self.reply_requested { 1 } else { 0 };
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HotStandbyFeedbackMessage {
    pub sent_at_unix_timestamp: i64,
    pub xmin: i32,
    pub xmin_epoch: i32,
    pub lowest_replication_slot_catalog_xmin: i32,
    pub catalog_xmin_epoch: i32,
}

impl HotStandbyFeedbackMessage {
    pub const SIZE: usize =
        std::mem::size_of::<i64>() + std::mem::size_of::<i32>() * 4;

    pub fn from_network_buffer(buffer: &[u8; Self::SIZE]) -> Self {
        HotStandbyFeedbackMessage {
            sent_at_unix_timestamp: i64::from_be_bytes(
                buffer[0..8].try_into().unwrap(),
            ),
            xmin: i32::from_be_bytes(buffer[8..12].try_into().unwrap()),
            xmin_epoch: i32::from_be_bytes(buffer[12..16].try_into().unwrap()),
            lowest_replication_slot_catalog_xmin: i32::from_be_bytes(
                buffer[16..20].try_into().unwrap(),
            ),
            catalog_xmin_epoch: i32::from_be_bytes(
                buffer[20..24].try_into().unwrap(),
            ),
        }
    }

    pub fn to_network_buffer(&self, buffer: &mut [u8; Self::SIZE]) {
        buffer[0..8]
            .copy_from_slice(&self.sent_at_unix_timestamp.to_be_bytes());
        buffer[8..12].copy_from_slice(&self.xmin.to_be_bytes());
        buffer[12..16].copy_from_slice(&self.xmin_epoch.to_be_bytes());
        buffer[16..20].copy_from_slice(
            &self.lowest_replication_slot_catalog_xmin.to_be_bytes(),
        );
        buffer[20..24].copy_from_slice(&self.catalog_xmin_epoch.to_be_bytes());
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrimaryEvent<'a> {
    XLogData(XLogData<'a>),
    PrimaryKeepaliveMessage(PrimaryKeepaliveMessage),
}

#[derive(Debug, PartialEq, Eq)]
pub enum StandbyEvent {
    StandbyStatusUpdate(StandbyStatusUpdate),
    HotStandbyFeedbackMessage(HotStandbyFeedbackMessage),
}

impl StandbyEvent {
    pub fn to_network_buffer(event: &StandbyEvent) -> Vec<u8> {
        match event {
            StandbyEvent::StandbyStatusUpdate(msg) => {
                standby_status_update_to_network_buffer(msg).to_vec()
            }
            StandbyEvent::HotStandbyFeedbackMessage(msg) => {
                hot_standby_feedback_message_to_network_buffer(msg).to_vec()
            }
        }
    }
}

pub fn primary_keepalive_message_to_network_buffer(
    message: &PrimaryKeepaliveMessage,
) -> [u8; 1 + PrimaryKeepaliveMessage::SIZE] {
    let mut buffer = [0u8; 1 + PrimaryKeepaliveMessage::SIZE];
    buffer[0] = PrimaryEventType::PrimaryKeepaliveMessage as u8;

    // Safely slice the array starting at index 1 to pass to serialization logic
    let payload_slice: &mut [u8; PrimaryKeepaliveMessage::SIZE] =
        (&mut buffer[1..]).try_into().unwrap();
    message.to_network_buffer(payload_slice);

    buffer
}

pub fn x_log_data_to_network_buffer(data: &XLogData) -> Vec<u8> {
    let mut buffer = vec![0u8; 1 + data.get_network_buffer_size()];
    buffer[0] = PrimaryEventType::XLogData as u8;
    data.to_network_buffer(&mut buffer[1..]);
    buffer
}

pub fn primary_event_to_network_buffer(event: &PrimaryEvent) -> Vec<u8> {
    match event {
        PrimaryEvent::PrimaryKeepaliveMessage(msg) => {
            primary_keepalive_message_to_network_buffer(msg).to_vec()
        }
        PrimaryEvent::XLogData(data) => x_log_data_to_network_buffer(data),
    }
}

pub fn standby_event_type_from_char(c: u8) -> Option<StandbyEventType> {
    if c == StandbyEventType::StandbyStatusUpdate as u8 {
        Some(StandbyEventType::StandbyStatusUpdate)
    } else if c == StandbyEventType::HotStandbyFeedbackMessage as u8 {
        Some(StandbyEventType::HotStandbyFeedbackMessage)
    } else {
        None
    }
}

pub fn standby_event_from_network_buffer(
    buffer: &[u8],
) -> Result<StandbyEvent, String> {
    if buffer.is_empty() {
        return Err("Empty buffer".to_string());
    }

    let type_char = buffer[0];
    let event_type = standby_event_type_from_char(type_char)
        .ok_or_else(|| format!("Unexpected type: {}", type_char))?;

    let event_buffer = &buffer[1..];

    match event_type {
        StandbyEventType::StandbyStatusUpdate => {
            if event_buffer.len() != StandbyStatusUpdate::SIZE {
                return Err(format!(
                    "StandbyStatusUpdate buffer size must be equal to {}",
                    StandbyStatusUpdate::SIZE
                ));
            }
            let fixed_buf: &[u8; StandbyStatusUpdate::SIZE] =
                event_buffer.try_into().unwrap();
            Ok(StandbyEvent::StandbyStatusUpdate(
                StandbyStatusUpdate::from_network_buffer(fixed_buf),
            ))
        }
        StandbyEventType::HotStandbyFeedbackMessage => {
            if event_buffer.len() != HotStandbyFeedbackMessage::SIZE {
                return Err(format!(
                    "HotStandbyFeedbackMessage buffer size must be equal to {}",
                    HotStandbyFeedbackMessage::SIZE
                ));
            }
            let fixed_buf: &[u8; HotStandbyFeedbackMessage::SIZE] =
                event_buffer.try_into().unwrap();
            Ok(StandbyEvent::HotStandbyFeedbackMessage(
                HotStandbyFeedbackMessage::from_network_buffer(fixed_buf),
            ))
        }
    }
}

pub fn standby_status_update_to_network_buffer(
    message: &StandbyStatusUpdate,
) -> [u8; 1 + StandbyStatusUpdate::SIZE] {
    let mut buffer = [0u8; 1 + StandbyStatusUpdate::SIZE];
    buffer[0] = StandbyEventType::StandbyStatusUpdate as u8;

    let payload_slice: &mut [u8; StandbyStatusUpdate::SIZE] =
        (&mut buffer[1..]).try_into().unwrap();
    message.to_network_buffer(payload_slice);

    buffer
}

pub fn hot_standby_feedback_message_to_network_buffer(
    message: &HotStandbyFeedbackMessage,
) -> [u8; 1 + HotStandbyFeedbackMessage::SIZE] {
    let mut buffer = [0u8; 1 + HotStandbyFeedbackMessage::SIZE];
    buffer[0] = StandbyEventType::HotStandbyFeedbackMessage as u8;

    let payload_slice: &mut [u8; HotStandbyFeedbackMessage::SIZE] =
        (&mut buffer[1..]).try_into().unwrap();
    message.to_network_buffer(payload_slice);

    buffer
}
