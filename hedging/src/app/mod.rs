use opentelemetry::{propagation::TextMapPropagator, sdk::propagation::TraceContextPropagator};
use tracing::{info_span, instrument, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use futures::stream::StreamExt;

use shared::{
    payload::SynthUsdExposurePayload,
    pubsub::{PubSubConfig, Subscriber},
};

use crate::error::*;

pub struct HedgingApp {}

impl HedgingApp {
    pub async fn run(config: PubSubConfig) -> Result<Self, HedgingError> {
        let subscriber = Subscriber::new(config).await?;
        let mut stream = subscriber.subscribe::<SynthUsdExposurePayload>().await?;
        let app = HedgingApp {};
        // balance updator {
        //   tokio::spawn(async move {
        //     poll or listen  to WS the exchange and write the current exposure
        //   })
        // }
        // msg receiver
        let _ = tokio::spawn(async move {
            let propagator = TraceContextPropagator::new();

            while let Some(msg) = stream.next().await {
                let span = info_span!(
                    "synth_usd_exposure_received",
                    message_type = %msg.payload_type,
                    correlation_id = %msg.meta.correlation_id
                );
                let context = propagator.extract(&msg.meta.tracing_data);
                span.set_parent(context);

                let exposure = msg.payload.exposure;
                // load last known exposure
                // diff with new exposure
                //
                //
                // exposue changed => trigger check
                // balance changed => trigger check
                //
                // if need action?
                //   create job
                // else
                //   ignore
            }
        });
        Ok(app)
    }
}
