/// Wrapper for u8 error code that implements the [coap_message::error::RenderableOnMinimal]
/// trait so that we can hook up the error codes from the past version of the
/// API to the new way of handling errors in coap_message.
#[derive(Debug)]
pub struct GenericRequestError(pub u8);

impl From<u8> for GenericRequestError {
    fn from(value: u8) -> Self {
        GenericRequestError(value)
    }
}

impl coap_message::error::RenderableOnMinimal for GenericRequestError {
    type Error<IE: coap_message::error::RenderableOnMinimal + core::fmt::Debug> =
        core::convert::Infallible;

    fn render<M: coap_message::MinimalWritableMessage>(
        self,
        message: &mut M,
    ) -> Result<(), Self::Error<M::UnionError>> {
        let _ = message.set_payload(&[self.0]);
        Ok(())
    }
}

