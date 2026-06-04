pub struct PGReplicationState {
    pub standby_message_timeout: std::time::Duration,
    pub last_status: std::time::SystemTime,
    pub output_written_lsn: i64,
}

impl PGReplicationState {
    pub fn new() -> Self {
        Self {
            standby_message_timeout: std::time::Duration::from_secs(10),
            last_status: std::time::SystemTime::now(),
            output_written_lsn: 0,
        }
    }

    pub fn should_send_keepalive(&self, now: std::time::SystemTime) -> bool {
        let duration_since = now.duration_since(self.last_status);
        duration_since.unwrap_or(std::time::Duration::ZERO)
            >= self.standby_message_timeout
    }

    pub fn on_primary_keepalive_message(
        &mut self,
        message: &super::events::PrimaryKeepaliveMessage,
    ) {
        self.output_written_lsn =
            std::cmp::max(self.output_written_lsn, message.server_wal_end);
        if message.reply_requested {
            self.last_status = std::time::UNIX_EPOCH;
        }
    }

    pub fn on_xlog_data(&mut self, message: &super::events::XLogData) {
        self.output_written_lsn =
            std::cmp::max(self.output_written_lsn, message.message_wal_start);
    }
}
