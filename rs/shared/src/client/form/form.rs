use bytes::Bytes;
use reqwest::{
    multipart::{Form, Part},
    Body,
};
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::error::FormDataError;

pub fn create_form_data(
    metadata: serde_json::Value,
    stream: UnboundedReceiverStream<Result<Bytes, hyper::Error>>,
) -> Result<Form, FormDataError> {
    let metadata = Part::text(metadata.to_string()).mime_str("application/json")?;
    let stream = Part::stream(Body::wrap_stream(stream)).mime_str("application/octet-stream")?;

    let form_data = Form::new()
        .part("metadata", metadata)
        .part("stream", stream);
    Ok(form_data)
}
