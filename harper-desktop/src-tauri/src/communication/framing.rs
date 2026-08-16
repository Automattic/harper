use serde::Serialize;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::error::ProtocolError;

pub(super) async fn write_message<W, T>(writer: &mut W, message: &T) -> Result<(), ProtocolError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let encoded = serde_json::to_vec(message)?;
    writer.write_all(&encoded).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}
